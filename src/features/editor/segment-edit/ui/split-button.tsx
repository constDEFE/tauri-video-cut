import { useHotkeys } from "@tanstack/react-hotkeys";

import { useSegmentsStore } from "@/entities/segments";
import { useSessionStore } from "@/entities/session";
import { useVideoStore } from "@/entities/video";
import { SplitIcon } from "@/shared/ui/icons";

export const SplitSegmentButton = () => {
	const split = useSegmentsStore((s) => s.actions.split);
	const segment = useSegmentsStore((s) => s.state.selectedSegment);
	const cursor = useVideoStore((s) => s.state.timeline.cursor);
	const updateSession = useSessionStore((s) => s.actions.updateSession);

	const canSplit = segment && cursor > segment.start && cursor < segment.end;

	const handleSplit = () => {
		if (!canSplit) {
			return;
		}

		split(segment.id, cursor);
		updateSession({ segments: useSegmentsStore.getState().state.segments });
	};

	useHotkeys([
		{ hotkey: "X", callback: handleSplit },
		{ hotkey: "\\", callback: handleSplit }
	]);

	return (
		<button
			onClick={handleSplit}
			disabled={!canSplit}
			class="button icon size-9 rounded"
			title="Split Selected Segment ( X )"
		>
			<SplitIcon class="size-5" />
		</button>
	);
};
