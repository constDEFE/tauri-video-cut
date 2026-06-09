import { useHotkeys } from "@tanstack/react-hotkeys";
import { nanoid } from "nanoid";

import { useSegmentsStore } from "@/entities/segments";
import { calculateEstimatedSize, useVideoStore } from "@/entities/video";
import { PlusIcon } from "@/shared/ui/icons";
import { invariant } from "@/shared/utils";

import type { Segment } from "@/entities/segments";

export const AddSegmentButton = () => {
	const add = useSegmentsStore((s) => s.actions.add);
	const metadata = useVideoStore((s) => s.state.metadata);

	const handleAdd = () => {
		invariant(metadata);

		const newSegment: Segment = {
			id: nanoid(),
			start: 0,
			end: metadata.duration,
			duration: metadata.duration,
			estimatedSize: calculateEstimatedSize(metadata.bitrate, metadata.duration)
		};

		add(newSegment);
	};

	useHotkeys([
		{ hotkey: "A", callback: handleAdd },
		{ hotkey: "=", callback: handleAdd }
	]);

	return (
		<button onClick={handleAdd} class="button icon size-9 rounded" title="Add Segment ( A )">
			<PlusIcon class="size-5" />
		</button>
	);
};
