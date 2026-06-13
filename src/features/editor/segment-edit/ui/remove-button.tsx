import { useHotkeys } from "@tanstack/react-hotkeys";
import { useShallow } from "zustand/shallow";

import { useSegmentsStore } from "@/entities/segments";
import { MinusIcon } from "@/shared/ui/icons";

import type { SegmentsStore } from "@/entities/segments";

const SELECT_SEGMENTS_STATE = (s: SegmentsStore) => ({
	isOnlySegment: s.state.segments.length === 1,
	selectedId: s.state.selectedId
});

export const RemoveSegmentButton = () => {
	const { isOnlySegment, selectedId } = useSegmentsStore(useShallow(SELECT_SEGMENTS_STATE));
	const remove = useSegmentsStore((s) => s.actions.remove);

	const handleRemove = () => {
		if (isOnlySegment) {
			return;
		}

		remove(selectedId);
	};

	useHotkeys([
		{ hotkey: "R", callback: handleRemove },
		{ hotkey: "-", callback: handleRemove }
	]);

	return (
		<button
			onClick={handleRemove}
			disabled={isOnlySegment}
			class="button icon size-9 rounded"
			title="Remove Selected Segment ( R )"
		>
			<MinusIcon class="size-5" />
		</button>
	);
};
