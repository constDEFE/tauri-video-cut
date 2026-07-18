import { useHotkeys } from "@tanstack/react-hotkeys";
import { useShallow } from "zustand/shallow";

import { useSegmentsStore } from "@/entities/segments";
import { useSessionStore } from "@/entities/session";
import { MinusIcon } from "@/shared/ui/icons";

import type { SegmentsStore } from "@/entities/segments";

const SELECT_SEGMENTS_STATE = (s: SegmentsStore) => ({
	isOnlySegment: s.state.segments.length === 1,
	selectedId: s.state.selectedSegment?.id
});
export const RemoveSegmentButton = () => {
	const { isOnlySegment, selectedId } = useSegmentsStore(useShallow(SELECT_SEGMENTS_STATE));
	const remove = useSegmentsStore((s) => s.actions.remove);
	const updateSession = useSessionStore((s) => s.actions.updateSession);

	const handleRemove = () => {
		if (isOnlySegment || !selectedId) {
			return;
		}

		remove(selectedId);
		updateSession({ segments: useSegmentsStore.getState().state.segments });
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
