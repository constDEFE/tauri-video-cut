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

const drawVerticalLine = (ctx: CanvasRenderingContext2D, x: number, height: number) => {
	ctx.beginPath();
	ctx.moveTo(x, 0);
	ctx.lineTo(x, height);
	ctx.stroke();
};

const buildDrawableIndices = (count: number, width: number): number[] => {
	const maxDrawablePoints = Math.max(2, Math.ceil(width * 2));
	const step = Math.max(1, Math.floor(count / maxDrawablePoints));

	const indices: number[] = [];

	for (let i = 0; i < count; i += step) {
		indices.push(i);
	}

	if (indices.length === 0 || indices[indices.length - 1] !== count - 1) {
		indices.push(count - 1);
	}

	return indices;
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

	const indices = buildDrawableIndices(count, width);
	const stepX = width / (totalPoints - 1);
	const direction = isTop ? -1 : 1;

	const peakAmpAt = (index: number): number => {
		const up = upSamples[index] ?? 0;
		const down = downSamples[index] ?? 0;
		const raw = Math.max(up, down);
		if (raw <= WAVEFORM_NOISE_GATE) return 0;
		return Math.min(1, raw * U8_NORMALIZE * gain);
	};

	const rmsAmpAt = (index: number): number => {
		if (!rmsSamples) return 0;
		const raw = rmsSamples[index] ?? 0;
		if (raw <= WAVEFORM_NOISE_GATE) return 0;
		let rmsAmp = Math.min(1, raw * U8_NORMALIZE * gain * RMS_VISUAL_SCALE);
		const peak = peakAmpAt(index);
		if (rmsAmp > peak) rmsAmp = peak;
		return rmsAmp;
	};

	const peakPath = new Path2D();

	let lastX = 0;
	indices.forEach((index, n) => {
		const amp = peakAmpAt(index);
		const x = index * stepX;
		const y = centerY + direction * amp * waveHeight;
		if (n === 0) peakPath.moveTo(x, y);
		else peakPath.lineTo(x, y);
		lastX = x;
	});

	if ((count - 1) % (indices[1] ?? 1) !== 0 && count > 0) {
		const lastIndex = count - 1;
		const x = lastIndex * stepX;
		const y = centerY + direction * peakAmpAt(lastIndex) * waveHeight;
		peakPath.lineTo(x, y);
		lastX = x;
	}

	peakPath.lineTo(lastX, centerY);
	peakPath.lineTo(0, centerY);
	peakPath.closePath();

	// 1. Peak Fill
	ctx.save();
	ctx.fillStyle = colors.waveform;
	ctx.globalAlpha = 0.33;
	ctx.fill(peakPath);
	ctx.restore();

	// 2. RMS Fill
	if (rmsSamples?.length) {
		const rmsPath = new Path2D();
		let rmsLastX = 0;

		indices.forEach((index, n) => {
			const amp = rmsAmpAt(index);
			const x = index * stepX;
			const y = centerY + direction * amp * waveHeight;
			if (n === 0) rmsPath.moveTo(x, y);
			else rmsPath.lineTo(x, y);
			rmsLastX = x;
		});

		if ((count - 1) % (indices[1] ?? 1) !== 0 && count > 0) {
			const lastIndex = count - 1;
			const x = lastIndex * stepX;
			const y = centerY + direction * rmsAmpAt(lastIndex) * waveHeight;
			rmsPath.lineTo(x, y);
			rmsLastX = x;
		}

		rmsPath.lineTo(rmsLastX, centerY);
		rmsPath.lineTo(0, centerY);
		rmsPath.closePath();

		ctx.fillStyle = colors.waveform;
		ctx.fill(rmsPath);
	}

	// 3. Peak Outline
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
