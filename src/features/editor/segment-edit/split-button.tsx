import { useHotkeys } from "@tanstack/react-hotkeys";
import { useShallow } from "zustand/shallow";

import { useSegmentsStore } from "@/entities/segments";
import { useVideoStore } from "@/entities/video";
import { SplitIcon } from "@/shared/ui/icons";

import type { SegmentsStore } from "@/entities/segments";

const SELECT_SEGMENTS_ACTIONS = (s: SegmentsStore) => ({
	split: s.actions.split,
	getSegment: s.actions.getById
});

export const SplitSegmentButton = () => {
	const { split, getSegment } = useSegmentsStore(useShallow(SELECT_SEGMENTS_ACTIONS));
	const selectedId = useSegmentsStore((s) => s.state.selectedId);
	const cursor = useVideoStore((s) => s.state.timeline.cursor);

	const segment = getSegment(selectedId);
	const canSplit = segment && cursor > segment.start && cursor < segment.end;

	const handleSplit = () => {
		if (!canSplit) {
			return;
		}

		split(selectedId, cursor);
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
