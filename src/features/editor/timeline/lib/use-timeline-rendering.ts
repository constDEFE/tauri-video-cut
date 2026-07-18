import { useCallback, useLayoutEffect, useRef } from "preact/hooks";
import { useShallow } from "zustand/react/shallow";

import { useSegmentsStore } from "@/entities/segments";
import { useVideoStore } from "@/entities/video";
import { useWaveformStore } from "@/entities/waveform";
import { useTheme } from "@/shared/lib/theme";

import { drawCursorOverlay, drawTimelineBase } from "./canvas-renderer";

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

export const useTimelineRendering = ({ baseCanvasRef, cursorCanvasRef, containerRef, duration }: Props) => {
	const { cursor, selectedAudio } = useVideoStore(useShallow(SELECT_VIDEO));
	const getAudioById = useVideoStore((s) => s.actions.player.getAudioById);
	const selectedSegment = useSegmentsStore((s) => s.state.selectedSegment);
	const { theme } = useTheme();

	const lastRenderedCursorRef = useRef<number | null>(null);
	const sizeRef = useRef({ width: 0, height: 0 });

	const selectedTrackIdx = selectedAudio != null ? (getAudioById(selectedAudio)?.index ?? null) : null;

	const syncCanvasSize = useCallback(() => {
		const container = containerRef.current;
		const base = baseCanvasRef.current;
		const overlay = cursorCanvasRef.current;

		if (!container || !base || !overlay) return;

		const width = Math.max(1, Math.floor(container.getBoundingClientRect().width));
		const height = base.clientHeight || 40;

		sizeRef.current = { width, height };

		if (base.width !== width || base.height !== height) {
			base.width = width;
			base.height = height;
			overlay.width = width;
			overlay.height = height;

			return true;
		}

		return false;
	}, [containerRef, baseCanvasRef, cursorCanvasRef]);

	const drawBase = useCallback(() => {
		const canvas = baseCanvasRef.current;
		const ctx = canvas?.getContext("2d");

		if (!canvas || !ctx || duration <= 0) return;

		const wf =
			selectedTrackIdx != null ? useWaveformStore.getState().getters.waveformByTrackIdx(selectedTrackIdx) : undefined;

		drawTimelineBase(ctx, {
			width: sizeRef.current.width,
			height: sizeRef.current.height,
			duration,
			selectedSegment,
			theme,
			waveform: wf && wf.left.length > 0 ? wf : undefined,
			totalPoints: wf?.totalPoints
		});
	}, [duration, selectedSegment, theme, selectedTrackIdx, baseCanvasRef]);

	const drawCursor = useCallback(() => {
		const canvas = cursorCanvasRef.current;
		const ctx = canvas?.getContext("2d");

		if (!canvas || !ctx || duration <= 0 || sizeRef.current.width <= 0) return;

		const { width, height } = sizeRef.current;
		const pixelsPerSecond = width / duration;
		const x = cursor * pixelsPerSecond;

		ctx.clearRect(0, 0, width, height);
		drawCursorOverlay(ctx, x, height, theme);

		lastRenderedCursorRef.current = cursor;
	}, [cursor, duration, cursorCanvasRef, theme]);

	useLayoutEffect(() => {
		syncCanvasSize();
		drawBase();
		drawCursor();

		const ro = new ResizeObserver(() => {
			syncCanvasSize();
			drawBase();
			drawCursor();
		});

		if (containerRef.current) {
			ro.observe(containerRef.current);
		}

		return () => ro.disconnect();
	}, [syncCanvasSize, drawBase, drawCursor, containerRef]);

	useLayoutEffect(() => {
		return useWaveformStore.subscribe((store, prev) => {
			if (store.state.waveformUpdateCounter !== prev.state.waveformUpdateCounter) {
				drawBase();
			}
		});
	}, [drawBase]);

	useLayoutEffect(() => {
		if (lastRenderedCursorRef.current !== cursor) {
			drawCursor();
		}
	}, [cursor, drawCursor]);
};
