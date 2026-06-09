import { useShallow } from "zustand/shallow";

import { useSegmentsStore } from "@/entities/segments";

import { ListItem } from "./list-item";

import type { SegmentsStore } from "@/entities/segments";

const SELECT_SEGMENTS = (s: SegmentsStore) => ({
	segments: s.state.segments,
	selectedId: s.state.selectedId
});

export const List = () => {
	const { segments, selectedId } = useSegmentsStore(useShallow(SELECT_SEGMENTS));

	return (
		<div class="scrollbar-track-surface scrollbar-thumb-accent flex flex-1 flex-col gap-2 overflow-y-auto px-4 pb-4">
			{segments.map((s, index) => (
				<ListItem key={s.id} segment={s} idx={index} isSelected={s.id === selectedId} />
			))}
		</div>
	);
};
