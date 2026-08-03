import { useCallback, useLayoutEffect, useRef } from "preact/hooks";
import { useShallow } from "zustand/react/shallow";

import { useSegmentsStore } from "@/entities/segments";
import { useVideoStore } from "@/entities/video";
import { useWaveformStore } from "@/entities/waveform";
import { useAppTheme } from "@/shared/lib/theme";

import { drawCursorOverlay, drawTimelineBase } from "./canvas-renderer";
import { useCanvasResolution } from "./use-canvas-resolution";

import type { VideoStore } from "@/entities/video";
import type { RefObject } from "preact";

type Props = {
	baseCanvasRef: RefObject<HTMLCanvasElement>;
	cursorCanvasRef: RefObject<HTMLCanvasElement>;
	containerRef: RefObject<HTMLElement>;
	duration: number;
};

const SELECT_VIDEO = (s: VideoStore) => ({
	cursor: s.state.timeline.cursor,
	selectedAudio: s.state.player.selectedAudio
});

const useBatchedRedraw = () => {
	const pendingRef = useRef({ base: false, cursor: false });
	const rafRef = useRef(0);

	const schedule = useCallback(
		(drawBase: () => void, drawCursor: () => void, options?: { base?: boolean; cursor?: boolean }) => {
			const { base = false, cursor = false } = options ?? {};
			if (base) pendingRef.current.base = true;
			if (cursor) pendingRef.current.cursor = true;

			if (rafRef.current) return;

			rafRef.current = requestAnimationFrame(() => {
				rafRef.current = 0;
				const { base: needsBase, cursor: needsCursor } = pendingRef.current;
				pendingRef.current = { base: false, cursor: false };

				if (needsBase) drawBase();
				if (needsCursor) drawCursor();
			});
		},
		[]
	);

	const cancel = useCallback(() => {
		if (rafRef.current) {
			cancelAnimationFrame(rafRef.current);
			rafRef.current = 0;
		}
	}, []);

	return { schedule, cancel };
};

export const useTimelineRendering = ({ baseCanvasRef, cursorCanvasRef, containerRef, duration }: Props) => {
	const { cursor, selectedAudio } = useVideoStore(useShallow(SELECT_VIDEO));
	const getAudioById = useVideoStore((s) => s.actions.player.getAudioById);
	const selectedSegment = useSegmentsStore((s) => s.state.selectedSegment);
	const { theme } = useAppTheme();
	const lastRenderedCursorRef = useRef<number | null>(null);
	const selectedTrackIdx = selectedAudio != null ? (getAudioById(selectedAudio)?.index ?? null) : null;

	const { schedule, cancel } = useBatchedRedraw();

	const drawBase = useCallback(() => {
		const ctx = baseCtxRef.current;
		if (!ctx || duration <= 0) return;

		const wf =
			selectedTrackIdx != null ? useWaveformStore.getState().getters.waveformByTrackIdx(selectedTrackIdx) : undefined;

		drawTimelineBase(ctx, {
			width: sizeRef.current.cssWidth,
			height: sizeRef.current.cssHeight,
			duration,
			selectedSegment,
			theme,
			waveform: wf && wf.rmsLeft.length > 0 ? wf : undefined,
			totalPoints: wf?.totalPoints
		});
	}, [duration, selectedSegment, theme, selectedTrackIdx]);

	const drawCursor = useCallback(() => {
		const ctx = overlayCtxRef.current;
		if (!ctx || duration <= 0 || sizeRef.current.cssWidth <= 0) return;

		const { cssWidth, cssHeight } = sizeRef.current;
		const x = cursor * (cssWidth / duration);

		ctx.clearRect(0, 0, cssWidth, cssHeight);
		drawCursorOverlay(ctx, x, cssHeight, theme);
		lastRenderedCursorRef.current = cursor;
	}, [cursor, duration, theme]);

	const handleResize = useCallback(
		() => schedule(drawBase, drawCursor, { base: true, cursor: true }),
		[schedule, drawBase, drawCursor]
	);

	const { sizeRef, baseCtxRef, overlayCtxRef } = useCanvasResolution({
		baseCanvasRef,
		cursorCanvasRef,
		containerRef,
		defaultHeight: 40,
		onResize: handleResize
	});

	useLayoutEffect(() => {
		drawBase();
		drawCursor();
	}, [drawBase, drawCursor]);

	useLayoutEffect(() => {
		return useWaveformStore.subscribe((store, prev) => {
			if (store.state.waveformUpdateCounter !== prev.state.waveformUpdateCounter) {
				schedule(drawBase, drawCursor, { base: true });
			}
		});
	}, [schedule, drawBase, drawCursor]);

	useLayoutEffect(() => {
		if (lastRenderedCursorRef.current !== cursor) {
			schedule(drawBase, drawCursor, { cursor: true });
		}
	}, [cursor, schedule, drawBase, drawCursor]);

	useLayoutEffect(() => cancel, [cancel]);
};
