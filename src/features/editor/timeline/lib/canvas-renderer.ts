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

const drawVerticalLine = (ctx: CanvasRenderingContext2D, x: number, height: number) => {
	ctx.beginPath();
	ctx.moveTo(x, 0);
	ctx.lineTo(x, height);
	ctx.stroke();
};

const drawWaveformSide = (
	ctx: CanvasRenderingContext2D,
	samples: Uint8Array,
	width: number,
	centerY: number,
	waveHeight: number,
	isTop: boolean,
	colors: (typeof CANVAS_THEME)[keyof typeof CANVAS_THEME],
	totalPoints: number
) => {
	const count = samples.length;
	if (count === 0 || totalPoints <= 1 || width <= 0) return;

	// ✅ Basic LOD: Don't iterate more points than we have horizontal pixels
	const maxDrawablePoints = Math.max(2, Math.ceil(width * 2));
	const step = Math.max(1, Math.floor(count / maxDrawablePoints));

	const stepX = width / (totalPoints - 1);
	const direction = isTop ? -1 : 1;

	ctx.beginPath();
	let lastX = 0;

	for (let i = 0; i < count; i += step) {
		const amp = samples[i]! * U8_NORMALIZE;
		const x = i * stepX;
		const y = centerY + direction * amp * waveHeight;

		if (i === 0) ctx.moveTo(x, y);
		else ctx.lineTo(x, y);

		lastX = x;
	}

	// Ensure we connect to the exact end point
	if ((count - 1) % step !== 0 && count > 0) {
		const lastIndex = count - 1;
		const amp = samples[lastIndex]! * U8_NORMALIZE;
		const x = lastIndex * stepX;
		const y = centerY + direction * amp * waveHeight;
		ctx.lineTo(x, y);
		lastX = x;
	}

	ctx.strokeStyle = colors.waveformEdge;
	ctx.lineWidth = 1;
	ctx.stroke();

	ctx.lineTo(lastX, centerY);
	ctx.lineTo(0, centerY);
	ctx.closePath();
	ctx.fillStyle = colors.waveform;
	ctx.fill();
};

const drawWaveform = (
	ctx: CanvasRenderingContext2D,
	waveform: AudioWaveform,
	width: number,
	height: number,
	colors: (typeof CANVAS_THEME)[keyof typeof CANVAS_THEME],
	totalPoints: number
) => {
	if (!waveform.left?.length) return;

	const centerY = height / 2;
	const waveHeight = height * 0.475; // Split evenly for top/bottom

	drawWaveformSide(ctx, waveform.left, width, centerY, waveHeight, true, colors, totalPoints);
	if (waveform.right?.length) {
		drawWaveformSide(ctx, waveform.right, width, centerY, waveHeight, false, colors, totalPoints);
	}
};

// ✅ Pure base layer - NO cursor drawing here
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
