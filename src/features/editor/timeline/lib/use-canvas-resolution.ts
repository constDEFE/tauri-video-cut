import { useCallback, useLayoutEffect, useRef } from "preact/hooks";

import { useAppZoom } from "@/shared/lib/zoom";

import type { RefObject } from "preact";

type CanvasSize = {
	cssWidth: number;
	cssHeight: number;
	dpr: number;
};

type Props = {
	baseCanvasRef: RefObject<HTMLCanvasElement>;
	cursorCanvasRef: RefObject<HTMLCanvasElement>;
	containerRef: RefObject<HTMLElement>;
	defaultHeight: number;
	onResize?: () => void;
};

const applyCanvasBuffer = (
	canvas: HTMLCanvasElement,
	cssWidth: number,
	cssHeight: number,
	dpr: number,
	contextOptions?: CanvasRenderingContext2DSettings
): CanvasRenderingContext2D | null => {
	const physW = Math.round(cssWidth * dpr);
	const physH = Math.round(cssHeight * dpr);

	canvas.width = physW;
	canvas.height = physH;

	const ctx = canvas.getContext("2d", contextOptions);
	ctx?.setTransform(dpr, 0, 0, dpr, 0, 0);

	return ctx;
};

export const useCanvasResolution = ({
	baseCanvasRef,
	cursorCanvasRef,
	containerRef,
	defaultHeight,
	onResize
}: Props) => {
	const sizeRef = useRef<CanvasSize>({ cssWidth: 0, cssHeight: 0, dpr: 1 });
	const baseCtxRef = useRef<CanvasRenderingContext2D | null>(null);
	const overlayCtxRef = useRef<CanvasRenderingContext2D | null>(null);
	const zoomScale = useAppZoom();

	const sync = useCallback((): boolean => {
		const container = containerRef.current;
		const base = baseCanvasRef.current;
		const overlay = cursorCanvasRef.current;

		if (!container || !base || !overlay) return false;

		const cssWidth = Math.max(1, Math.floor(container.getBoundingClientRect().width));
		const cssHeight = base.clientHeight || defaultHeight;
		const dpr = zoomScale;

		const prev = sizeRef.current;
		const changed = prev.cssWidth !== cssWidth || prev.cssHeight !== cssHeight || prev.dpr !== dpr;

		if (!changed) return false;

		sizeRef.current = { cssWidth, cssHeight, dpr };

		baseCtxRef.current = applyCanvasBuffer(base, cssWidth, cssHeight, dpr, {
			alpha: false,
			desynchronized: true
		});
		overlayCtxRef.current = applyCanvasBuffer(overlay, cssWidth, cssHeight, dpr);

		return true;
	}, [defaultHeight, zoomScale]);

	useLayoutEffect(() => {
		sync();
		onResize?.();

		const listener = () => {
			if (sync()) onResize?.();
		};

		const ro = new ResizeObserver(listener);
		window.addEventListener("zoomUpdate", listener);

		if (containerRef.current) {
			ro.observe(containerRef.current);
		}

		return () => {
			ro.disconnect();
			window.removeEventListener("zoomUpdate", listener);
		};
	}, [sync, onResize, containerRef]);

	return { sizeRef, baseCtxRef, overlayCtxRef };
};
