import type { AudioWaveform } from "@/entities/video";

export const CANVAS_THEME = {
	dark: {
		timelineArea: "#000000",
		segmentArea: "#303030",
		segmentBorder: "#ffffff",
		cursor: "#ffffff",
		waveform: "#ffffff",
		waveformEdge: "#ffffff"
	},
	light: {
		timelineArea: "#ffffff",
		segmentArea: "#d0d0d0",
		segmentBorder: "#000000",
		cursor: "#000000",
		waveform: "#000000",
		waveformEdge: "#000000"
	}
} as const;

type Column = {
	i0: number;
	i1: number;
	x: number;
	up: number;
	down: number;
	rms: number;
};

export type BaseDrawOptions = {
	width: number;
	height: number;
	duration: number;
	selectedSegment: { start: number; duration: number } | null;
	theme: keyof typeof CANVAS_THEME;
	waveform?: AudioWaveform | undefined;
	totalPoints?: number | undefined;
};

const U8_NORMALIZE = 1 / 255.0;

const WAVEFORM_NOISE_GATE = 2;
const RMS_VISUAL_SCALE = 0.8;

const buildColumns = (
	count: number,
	width: number,
	totalPoints: number,
	upSamples: Uint8Array,
	downSamples: Uint8Array,
	rmsSamples: Uint8Array | undefined
): Column[] => {
	const stepX = width / (totalPoints - 1);
	const needsAggregation = count > Math.max(2, Math.floor(width * 2));
	const cols = needsAggregation ? Math.max(2, Math.floor(width * 2)) : count;
	const out: Column[] = [];
	for (let c = 0; c < cols; c++) {
		const i0 = needsAggregation ? Math.floor((c * count) / cols) : c;
		const i1 = needsAggregation ? Math.max(i0 + 1, Math.floor(((c + 1) * count) / cols)) : i0 + 1;

		let up = 0;
		let down = 0;
		let rms = 0;
		for (let i = i0; i < i1; i++) {
			const u = upSamples[i] ?? 0;
			const d = downSamples[i] ?? 0;
			if (u > up) up = u;
			if (d > down) down = d;
			if (rmsSamples) {
				const r = rmsSamples[i] ?? 0;
				if (r > rms) rms = r;
			}
		}
		const midIndex = (i0 + i1 - 1) / 2;
		out.push({ i0, i1, x: midIndex * stepX, up, down, rms });
	}

	return out;
};

const drawVerticalLine = (ctx: CanvasRenderingContext2D, x: number, height: number) => {
	ctx.beginPath();
	ctx.moveTo(x, 0);
	ctx.lineTo(x, height);
	ctx.stroke();
};

const drawWaveformSideHybrid = (
	ctx: CanvasRenderingContext2D,
	upSamples: Uint8Array,
	downSamples: Uint8Array,
	rmsSamples: Uint8Array | undefined,
	width: number,
	centerY: number,
	waveHeight: number,
	isTop: boolean,
	colors: (typeof CANVAS_THEME)[keyof typeof CANVAS_THEME],
	totalPoints: number,
	gain: number
) => {
	const count = upSamples.length;
	if (count === 0 || totalPoints <= 1 || width <= 0) return;

	const columns = buildColumns(count, width, totalPoints, upSamples, downSamples, rmsSamples);
	const direction = isTop ? -1 : 1;

	const peakAmp = (col: Column) => {
		const raw = Math.max(col.up, col.down);
		if (raw <= WAVEFORM_NOISE_GATE) return 0;
		return Math.min(1, raw * U8_NORMALIZE * gain);
	};

	const rmsAmp = (col: Column) => {
		if (!rmsSamples) return 0;
		if (col.rms <= WAVEFORM_NOISE_GATE) return 0;
		let a = Math.min(1, col.rms * U8_NORMALIZE * gain * RMS_VISUAL_SCALE);
		const p = peakAmp(col);
		if (a > p) a = p;
		return a;
	};

	const peakPath = new Path2D();

	columns.forEach((col, n) => {
		const y = centerY + direction * peakAmp(col) * waveHeight;
		if (n === 0) peakPath.moveTo(col.x, y);
		else peakPath.lineTo(col.x, y);
	});

	const last = columns[columns.length - 1];

	peakPath.lineTo(last!.x, centerY);
	peakPath.lineTo(columns[0]!.x, centerY);
	peakPath.closePath();

	ctx.save();
	ctx.fillStyle = colors.waveform;
	ctx.globalAlpha = 0.33;
	ctx.fill(peakPath);
	ctx.restore();

	if (rmsSamples?.length) {
		const rmsPath = new Path2D();

		columns.forEach((col, n) => {
			const y = centerY + direction * rmsAmp(col) * waveHeight;
			if (n === 0) rmsPath.moveTo(col.x, y);
			else rmsPath.lineTo(col.x, y);
		});

		rmsPath.lineTo(last!.x, centerY);
		rmsPath.lineTo(columns[0]!.x, centerY);
		rmsPath.closePath();
		ctx.fillStyle = colors.waveform;
		ctx.fill(rmsPath);
	}

	ctx.strokeStyle = colors.waveformEdge;
	ctx.lineWidth = 1;
	ctx.stroke(peakPath);
};

const drawWaveform = (
	ctx: CanvasRenderingContext2D,
	waveform: AudioWaveform,
	width: number,
	height: number,
	colors: (typeof CANVAS_THEME)[keyof typeof CANVAS_THEME],
	totalPoints: number
) => {
	if (!waveform.leftUp?.length || !waveform.leftDown?.length) return;

	const centerY = height / 2;
	const waveHeight = Math.max(1, height / 2 - 1);

	const gain = typeof waveform.displayGain === "number" && waveform.displayGain > 0 ? waveform.displayGain : 1;

	// Top half: left channel (upward from center)
	drawWaveformSideHybrid(
		ctx,
		waveform.leftUp,
		waveform.leftDown,
		waveform.rmsLeft,
		width,
		centerY,
		waveHeight,
		true,
		colors,
		totalPoints,
		gain
	);

	// Bottom half: right channel (downward from center)
	if (waveform.rightUp?.length && waveform.rightDown?.length) {
		drawWaveformSideHybrid(
			ctx,
			waveform.rightUp,
			waveform.rightDown,
			waveform.rmsRight,
			width,
			centerY,
			waveHeight,
			false,
			colors,
			totalPoints,
			gain
		);
	}
};

export const drawTimelineBase = (ctx: CanvasRenderingContext2D, options: BaseDrawOptions) => {
	const { width, height, duration, selectedSegment, theme, waveform, totalPoints } = options;
	const colors = CANVAS_THEME[theme];

	ctx.clearRect(0, 0, width, height);
	ctx.fillStyle = colors.timelineArea;
	ctx.fillRect(0, 0, width, height);

	if (duration <= 0) return;

	const pixelsPerSecond = width / duration;

	if (selectedSegment) {
		const startX = selectedSegment.start * pixelsPerSecond;
		const segmentWidth = selectedSegment.duration * pixelsPerSecond;
		ctx.fillStyle = colors.segmentArea;
		ctx.fillRect(startX, 0, segmentWidth, height);
		ctx.strokeStyle = colors.segmentBorder;
		ctx.lineWidth = 1;
		ctx.lineCap = "round";
		drawVerticalLine(ctx, startX, height);
		drawVerticalLine(ctx, startX + segmentWidth, height);
	}

	if (waveform && totalPoints) {
		drawWaveform(ctx, waveform, width, height, colors, totalPoints);
	}
};

export const drawCursorOverlay = (
	ctx: CanvasRenderingContext2D,
	x: number,
	height: number,
	theme: keyof typeof CANVAS_THEME
) => {
	if (!Number.isFinite(x) || height <= 0) return;

	const colors = CANVAS_THEME[theme];

	ctx.save();
	ctx.strokeStyle = colors.cursor;
	ctx.lineWidth = 1.5;
	ctx.beginPath();
	ctx.moveTo(x, 0);
	ctx.lineTo(x, height);
	ctx.stroke();
	ctx.restore();
};

export const newCursor = (offsetX: number, canvas: HTMLCanvasElement, duration: number) => {
	const rect = canvas.getBoundingClientRect();
	if (rect.width <= 0 || duration <= 0) return 0;
	const position = ((offsetX - rect.left) / rect.width) * duration;

	return Math.max(0, Math.min(position, duration));
};
