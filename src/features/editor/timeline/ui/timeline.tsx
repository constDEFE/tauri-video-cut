import { useRef } from "preact/hooks";
import { useShallow } from "zustand/shallow";

import { formatTime, useVideoStore } from "@/entities/video";

import { useCursor } from "../lib/use-cursor";
import { useTimelineRendering } from "../lib/use-timeline-rendering";

import type { VideoStore } from "@/entities/video";

const SELECT_VIDEO = (s: VideoStore) => ({
	metadata: s.state.metadata,
	isPlaying: s.state.player.isPlaying,
	isDisabled: !s.state.player.isFileLoaded || !s.state.player.isInitialized
});

const SELECT_TIMELINE = (s: VideoStore) => ({
	cursor: s.state.timeline.cursor,
	setCursor: s.actions.timeline.setCursor
});

export const Timeline = () => {
	const containerRef = useRef<HTMLDivElement>(null);
	const baseCanvasRef = useRef<HTMLCanvasElement>(null);
	const cursorCanvasRef = useRef<HTMLCanvasElement>(null);

	const { metadata, isDisabled, isPlaying } = useVideoStore(useShallow(SELECT_VIDEO));
	const { cursor, setCursor } = useVideoStore(useShallow(SELECT_TIMELINE));
	const duration = metadata?.duration ?? 0;

	const { handlers, hoveredPosition } = useCursor({
		canvasRef: cursorCanvasRef,
		duration,
		isDisabled,
		isPlaying,
		setCursor
	});

	useTimelineRendering({ baseCanvasRef, cursorCanvasRef, containerRef, duration });

	const displayTime = hoveredPosition ?? cursor;
	const timeText = formatTime(displayTime, metadata?.fps);

	return (
		<div class="group relative isolate w-full px-2 pb-2">
			<span class="timeline-text">{timeText}</span>
			<div
				ref={containerRef}
				class="border-secondary relative h-10 w-full cursor-pointer overflow-hidden rounded-lg border opacity-0 duration-100 ease-out group-hover:opacity-100"
				{...handlers}
			>
				<canvas ref={baseCanvasRef} class="absolute inset-0 h-full w-full" />
				<canvas ref={cursorCanvasRef} class="pointer-events-none absolute inset-0 h-full w-full" />
			</div>
		</div>
	);
};
