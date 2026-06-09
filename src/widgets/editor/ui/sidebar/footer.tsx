import { useHotkeys } from "@tanstack/react-hotkeys";
import { useNavigate } from "react-router";
import { useShallow } from "zustand/shallow";

import { useVideoStore } from "@/entities/video";
import { AddSegmentButton, RemoveSegmentButton, SplitSegmentButton } from "@/features/editor/segment-edit";
import { SheetFooter } from "@/shared/ui";

import type { VideoStore } from "@/entities/video";

const SELECT_VIDEO = (s: VideoStore) => ({
	reset: s.actions.reset,
	resetPlayer: s.actions.player.reset
});

export const Footer = () => {
	const { reset, resetPlayer } = useVideoStore(useShallow(SELECT_VIDEO));
	const navigate = useNavigate();

	const handleExport = () => {
		navigate("/export", { replace: true });
		resetPlayer();
	};

	useHotkeys([
		{ hotkey: "Control+N", callback: reset },
		{ hotkey: "Control+S", callback: handleExport }
	]);

	return (
		<SheetFooter class="border-secondary flex flex-col gap-1.5 border-t px-2 pt-2 pb-2.5">
			<div class="flex gap-1 *:flex-1">
				<AddSegmentButton />
				<RemoveSegmentButton />
				<SplitSegmentButton />
			</div>
			<div class="grid grid-cols-2 gap-1">
				<button class="button h-9 rounded font-medium" onClick={reset} title="Import New Video ( Ctrl+N )">
					Import New
				</button>
				<button
					class="button h-9 rounded font-medium"
					onClick={handleExport}
					title="Export Created Segments ( Ctrl+S )"
				>
					Export
				</button>
			</div>
		</SheetFooter>
	);
};
