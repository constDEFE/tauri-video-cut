import { useCallback, useEffect, useLayoutEffect, useRef } from "preact/hooks";
import { useShallow } from "zustand/shallow";

import { useSegmentsStore } from "@/entities/segments";
import { formatTime, useVideoStore } from "@/entities/video";
import { useTheme } from "@/shared/lib/theme";
import { debounce, invariant } from "@/shared/utils";

import { drawTimeline, useTimeline } from "./lib";

import type { SegmentsStore } from "@/entities/segments";
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

const SELECT_SEGMENTS = (s: SegmentsStore) => ({
	selectedId: s.state.selectedId,
	segments: s.state.segments,
	getById: s.actions.getById
});

export const Timeline = () => {
	const containerRef = useRef<HTMLDivElement>(null);
	const canvasRef = useRef<HTMLCanvasElement>(null);

	const { isPlaying, metadata, isDisabled } = useVideoStore(useShallow(SELECT_VIDEO));
	const { cursor, setCursor } = useVideoStore(useShallow(SELECT_TIMELINE));
	const { getById, segments, selectedId } = useSegmentsStore(useShallow(SELECT_SEGMENTS));
	const { theme } = useTheme();

	const { handlers, hoveredPosition } = useTimeline({
		canvasRef,
		duration: metadata?.duration ?? 0,
		isDisabled,
		isPlaying,
		setCursor
	});

	const drawCanvas = useCallback(
		(canvas: HTMLCanvasElement) => {
			const ctx = canvas.getContext("2d");

			if (!ctx) return;

			invariant(metadata);

			drawTimeline(ctx, {
				height: canvas.height,
				width: canvas.width,
				selectedSegment: getById(selectedId),
				duration: metadata.duration,
				cursor,
				theme
			});
		},
		// eslint-disable-next-line react-hooks/exhaustive-deps
		[cursor, metadata?.duration, segments, selectedId, theme]
	);

	useLayoutEffect(() => {
		if (!canvasRef.current || !containerRef.current) {
			return;
		}

		canvasRef.current.width = containerRef.current.getBoundingClientRect().width - 16;
	}, []);

	useEffect(() => {
		if (!canvasRef.current) return;

		drawCanvas(canvasRef.current);

		const handleResize = debounce(() => {
			if (!canvasRef.current || !containerRef.current) {
				return;
			}

			canvasRef.current.width = containerRef.current.getBoundingClientRect().width - 16;
			drawCanvas(canvasRef.current);
		}, 200);

		window.addEventListener("resize", handleResize);

		return () => {
			window.removeEventListener("resize", handleResize);
		};
	}, [drawCanvas]);

	const displayTime = hoveredPosition ?? cursor;
	const timeText = formatTime(displayTime, metadata?.fps);

	return (
		<div ref={containerRef} class="group relative isolate w-full px-2 pb-2">
			<span class="pointer-events-none absolute top-1/2 left-1/2 z-10 -translate-x-1/2 -translate-y-[calc(50%+0.25rem)] font-mono text-sm text-white mix-blend-difference select-none">
				{timeText}
			</span>
			<canvas
				ref={canvasRef}
				width={1200}
				height={40}
				class="border-secondary h-[40px] w-full cursor-pointer rounded-lg border opacity-0 duration-100 ease-out group-hover:opacity-100"
				{...handlers}
			/>
		</div>
	);
};
