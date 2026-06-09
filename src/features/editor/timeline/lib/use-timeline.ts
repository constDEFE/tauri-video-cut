import { useCallback, useState } from "preact/hooks";

import { pause, play, seek } from "@/shared/lib/mpv";
import { debounce } from "@/shared/utils";

import { newCursor } from "./canvas-renderer";

type Props = {
	canvasRef: React.RefObject<HTMLCanvasElement>;
	duration: number;
	isPlaying: boolean;
	isDisabled: boolean;
	setCursor: (cursor: number) => void;
};

export const useTimeline = ({ canvasRef, duration, isPlaying, isDisabled, setCursor }: Props) => {
	const [hoveredPosition, setHoveredPosition] = useState<number | null>(null);

	const handleMouseMove = useCallback(
		(e: MouseEvent) => {
			if (!canvasRef.current) return;

			const position = newCursor(e.clientX, canvasRef.current, duration);

			setHoveredPosition(position);
		},
		[canvasRef, duration]
	);

	const handleMouseLeave = useCallback(() => {
		setHoveredPosition(null);
	}, []);

	const handleMouseDown = useCallback(
		async (e: MouseEvent) => {
			if (isDisabled || !canvasRef.current) return;

			if (isPlaying) {
				await pause().catch(console.error);
			}

			let isSeekCancelled = false;
			const debouncedSeek = debounce(async (cursor: number) => {
				if (isSeekCancelled) return;
				await seek(cursor).catch(console.error);
			}, 150);

			const handleDrag = (e: MouseEvent) => {
				const cursor = newCursor(e.clientX, canvasRef.current!, duration);

				setCursor(cursor);
				debouncedSeek(cursor);
			};

			const handleMouseUp = async (e: MouseEvent) => {
				document.removeEventListener("mousemove", handleDrag);
				document.removeEventListener("mouseup", handleMouseUp);

				const cursor = newCursor(e.clientX, canvasRef.current!, duration);

				isSeekCancelled = true;
				await seek(cursor).catch(console.error);

				if (isPlaying) {
					await play().catch(console.error);
				}
			};

			document.addEventListener("mousemove", handleDrag);
			document.addEventListener("mouseup", handleMouseUp);

			const cursor = newCursor(e.clientX, canvasRef.current, duration);
			setCursor(cursor);
		},
		[canvasRef, duration, isPlaying, isDisabled, setCursor]
	);

	return {
		hoveredPosition,
		handlers: {
			onMouseMove: handleMouseMove,
			onMouseLeave: handleMouseLeave,
			onMouseDown: handleMouseDown
		}
	};
};
