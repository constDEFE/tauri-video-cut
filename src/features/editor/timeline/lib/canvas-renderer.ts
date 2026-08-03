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

const U8_MAX = 255;
const NOISE_GATE_THRESHOLD = 2;
const RMS_VISUAL_SCALE = 0.8;
const PEAK_FILL_ALPHA = 0.33;
const CURSOR_LINE_WIDTH = 1.5;
const SEGMENT_BORDER_WIDTH = 1;
const MIN_WAVEFORM_COLUMNS = 2;
const WAVEFORM_COLUMN_DENSITY = 2; // columns per CSS pixel

type ThemeColors = (typeof CANVAS_THEME)[keyof typeof CANVAS_THEME];

type WaveformColumn = {
	x: number;
	peakUp: number;
	peakDown: number;
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

const aggregateWaveformColumns = (
	upSamples: Uint8Array,
	downSamples: Uint8Array,
	rmsSamples: Uint8Array | undefined,
	width: number,
	totalPoints: number
): WaveformColumn[] => {
	const sampleCount = upSamples.length;
	if (sampleCount === 0 || totalPoints <= 1 || width <= 0) return [];

	const stepX = width / (totalPoints - 1);
	const maxColumns = Math.max(MIN_WAVEFORM_COLUMNS, Math.floor(width * WAVEFORM_COLUMN_DENSITY));
	const needsAggregation = sampleCount > maxColumns;
	const columnCount = needsAggregation ? maxColumns : sampleCount;

	const columns: WaveformColumn[] = [];

	for (let c = 0; c < columnCount; c++) {
		const i0 = needsAggregation ? Math.floor((c * sampleCount) / columnCount) : c;
		const i1 = needsAggregation ? Math.max(i0 + 1, Math.floor(((c + 1) * sampleCount) / columnCount)) : i0 + 1;

		let peakUp = 0;
		let peakDown = 0;
		let rms = 0;

		for (let i = i0; i < i1; i++) {
			const u = upSamples[i] ?? 0;
			const d = downSamples[i] ?? 0;
			if (u > peakUp) peakUp = u;
			if (d > peakDown) peakDown = d;
			if (rmsSamples) {
				const r = rmsSamples[i] ?? 0;
				if (r > rms) rms = r;
			}
		}

		const midIndex = (i0 + i1 - 1) / 2;
		columns.push({ x: midIndex * stepX, peakUp, peakDown, rms });
	}

	return columns;
};

const computePeakAmp = (col: WaveformColumn, gain: number): number => {
	const raw = Math.max(col.peakUp, col.peakDown);
	if (raw <= NOISE_GATE_THRESHOLD) return 0;
	return Math.min(1, (raw / U8_MAX) * gain);
};

const computeRmsAmp = (col: WaveformColumn, gain: number, peakAmp: number): number => {
	if (col.rms <= NOISE_GATE_THRESHOLD) return 0;
	const scaled = Math.min(1, (col.rms / U8_MAX) * gain * RMS_VISUAL_SCALE);
	return Math.min(scaled, peakAmp);
};

const buildWaveformPath = (
	columns: WaveformColumn[],
	centerY: number,
	waveHeight: number,
	direction: number,
	amplitudeFn: (col: WaveformColumn) => number
): Path2D => {
	const path = new Path2D();
	if (columns.length === 0) return path;

	columns.forEach((col, index) => {
		const y = centerY + direction * amplitudeFn(col) * waveHeight;
		if (index === 0) path.moveTo(col.x, y);
		else path.lineTo(col.x, y);
	});

	const last = columns[columns.length - 1]!;
	path.lineTo(last.x, centerY);
	path.lineTo(columns[0]!.x, centerY);
	path.closePath();

	return path;
};

const drawWaveformSide = (
	ctx: CanvasRenderingContext2D,
	upSamples: Uint8Array,
	downSamples: Uint8Array,
	rmsSamples: Uint8Array | undefined,
	width: number,
	centerY: number,
	waveHeight: number,
	isTop: boolean,
	colors: ThemeColors,
	totalPoints: number,
	gain: number
) => {
	const columns = aggregateWaveformColumns(upSamples, downSamples, rmsSamples, width, totalPoints);
	if (columns.length === 0) return;

	const direction = isTop ? -1 : 1;

	const peakPath = buildWaveformPath(columns, centerY, waveHeight, direction, (col) => computePeakAmp(col, gain));
	ctx.save();
	ctx.fillStyle = colors.waveform;
	ctx.globalAlpha = PEAK_FILL_ALPHA;
	ctx.fill(peakPath);
	ctx.restore();

	if (rmsSamples?.length) {
		const rmsPath = buildWaveformPath(columns, centerY, waveHeight, direction, (col) =>
			computeRmsAmp(col, gain, computePeakAmp(col, gain))
		);
		ctx.fillStyle = colors.waveform;
		ctx.fill(rmsPath);
	}

	ctx.strokeStyle = colors.waveformEdge;
	ctx.lineWidth = 1;
	ctx.stroke(peakPath);
};

const hasValidWaveformData = (wf: AudioWaveform): boolean => !!(wf.leftUp?.length && wf.leftDown?.length);

const drawWaveform = (
	ctx: CanvasRenderingContext2D,
	waveform: AudioWaveform,
	width: number,
	height: number,
	colors: ThemeColors,
	totalPoints: number
) => {
	if (!hasValidWaveformData(waveform)) return;

	const centerY = height / 2;
	const waveHeight = Math.max(1, height / 2 - 1);
	const gain = typeof waveform.displayGain === "number" && waveform.displayGain > 0 ? waveform.displayGain : 1;

	drawWaveformSide(
		ctx,
		waveform.leftUp!,
		waveform.leftDown!,
		waveform.rmsLeft,
		width,
		centerY,
		waveHeight,
		true,
		colors,
		totalPoints,
		gain
	);

	if (waveform.rightUp?.length && waveform.rightDown?.length) {
		drawWaveformSide(
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

const drawSelectedSegment = (
	ctx: CanvasRenderingContext2D,
	segment: { start: number; duration: number },
	pixelsPerSecond: number,
	height: number,
	colors: ThemeColors
) => {
	const startX = segment.start * pixelsPerSecond;
	const segmentWidth = segment.duration * pixelsPerSecond;

	ctx.fillStyle = colors.segmentArea;
	ctx.fillRect(startX, 0, segmentWidth, height);

	ctx.strokeStyle = colors.segmentBorder;
	ctx.lineWidth = SEGMENT_BORDER_WIDTH;
	ctx.lineCap = "round";

	ctx.beginPath();
	ctx.moveTo(startX, 0);
	ctx.lineTo(startX, height);
	ctx.stroke();

	ctx.beginPath();
	ctx.moveTo(startX + segmentWidth, 0);
	ctx.lineTo(startX + segmentWidth, height);
	ctx.stroke();
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
		drawSelectedSegment(ctx, selectedSegment, pixelsPerSecond, height, colors);
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
	ctx.lineWidth = CURSOR_LINE_WIDTH;
	ctx.beginPath();
	ctx.moveTo(x, 0);
	ctx.lineTo(x, height);
	ctx.stroke();
	ctx.restore();
};

export const newCursor = (offsetX: number, canvas: HTMLCanvasElement, duration: number): number => {
	const rect = canvas.getBoundingClientRect();
	if (rect.width <= 0 || duration <= 0) return 0;
	const position = ((offsetX - rect.left) / rect.width) * duration;

	return Math.max(0, Math.min(position, duration));
};
