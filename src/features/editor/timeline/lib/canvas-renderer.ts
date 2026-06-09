export const CANVAS_THEME = {
	dark: {
		/* surface */
		timelineArea: "#151515",
		/* accent */
		segmentArea: "#000000",
		/* border */
		segmentBorder: "#808080",
		cursor: "#FFFFFF"
	},
	light: {
		/* surface */
		timelineArea: "#dfdfdf",
		/* accent */
		segmentArea: "#FFFFFF",
		/* border */
		segmentBorder: "#808080",
		cursor: "#FFFFFF"
	}
};

export type DrawOptions = {
	width: number;
	height: number;
	duration: number;
	cursor: number;
	selectedSegment: { start: number; duration: number } | null;
	theme: "light" | "dark";
};

export const drawTimeline = (ctx: CanvasRenderingContext2D, options: DrawOptions) => {
	const { width, height, duration, cursor, selectedSegment, theme } = options;
	const colors = CANVAS_THEME[theme];
	const pixelsPerSecond = width / duration;

	ctx.clearRect(0, 0, width, height);
	ctx.fillStyle = colors.timelineArea;
	ctx.fillRect(0, 0, width, height);

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

	drawCursor(ctx, cursor * pixelsPerSecond, height, colors.cursor);
};

const drawVerticalLine = (ctx: CanvasRenderingContext2D, x: number, height: number) => {
	ctx.beginPath();
	ctx.moveTo(x, 0);
	ctx.lineTo(x, height);
	ctx.stroke();
};

const drawCursor = (ctx: CanvasRenderingContext2D, x: number, height: number, color: string) => {
	ctx.save();
	ctx.globalCompositeOperation = "difference";
	ctx.strokeStyle = color;
	ctx.lineWidth = 1.5;
	ctx.beginPath();
	ctx.moveTo(x, 0);
	ctx.lineTo(x, height);
	ctx.stroke();
	ctx.restore();
};

export const newCursor = (offsetX: number, canvas: HTMLCanvasElement, duration: number) => {
	const rect = canvas.getBoundingClientRect();
	const x = offsetX - rect.left;
	const position = (x / rect.width) * duration;

	return Math.max(0, Math.min(position, duration));
};
