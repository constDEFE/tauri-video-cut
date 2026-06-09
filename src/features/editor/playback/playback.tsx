import { useHotkeys } from "@tanstack/react-hotkeys";
import { useShallow } from "zustand/shallow";

import { useSegmentsStore } from "@/entities/segments";
import { useVideoStore, type VideoStore } from "@/entities/video";
import { frameBackStep, frameStep, seekBackward, seekForward } from "@/shared/lib/mpv";
import { NextIcon, PrevIcon } from "@/shared/ui/icons";
import { invariant } from "@/shared/utils";

import { AudioSelector, PlayButton, VolumeSlider } from "./ui";

import type { Segment, SegmentsStore } from "@/entities/segments";

const newDuration = (type: "start" | "end", segment: Segment, cursor: number) => {
	if (type === "start") {
		return segment.end - cursor;
	}

	return cursor - segment.start;
};

const isInBounds = (type: "start" | "end", cursor: number, segment: Segment) => {
	return (type === "end" && cursor > segment.start) || (type === "start" && cursor < segment.end);
};

const SELECT_SEGMENT = (s: SegmentsStore) => ({
	selectedId: s.state.selectedId,
	getSegment: s.actions.getById,
	updateSegment: s.actions.update
});

const SELECT_VIDEO = (s: VideoStore) => ({
	metadata: s.state.metadata,
	isDisabled: !s.state.player.isFileLoaded || !s.state.player.isInitialized,
	cursor: s.state.timeline.cursor
});

export const Playback = () => {
	const { getSegment, selectedId, updateSegment } = useSegmentsStore(useShallow(SELECT_SEGMENT));
	const { metadata, isDisabled, cursor } = useVideoStore(useShallow(SELECT_VIDEO));

	const handleSegment = (type: Parameters<typeof newDuration>[0]) => {
		invariant(metadata);

		const selected = getSegment(selectedId);
		const computedType = isInBounds(type, cursor, selected) ? type : type === "start" ? "end" : "start";
		const duration = newDuration(computedType, selected, cursor);
		const newEstimatedSize = (metadata.bitrate / 8) * duration;

		updateSegment(selectedId, {
			[computedType]: cursor,
			duration: duration,
			estimatedSize: newEstimatedSize
		});
	};

	useHotkeys(
		[
			{ hotkey: "[", callback: () => handleSegment("start") },
			{ hotkey: "]", callback: () => handleSegment("end") },
			{ hotkey: ",", callback: () => frameBackStep().catch(console.error) },
			{ hotkey: "ArrowLeft", callback: () => seekBackward().catch(console.error) },
			{ hotkey: ".", callback: () => frameStep().catch(console.error) },
			{ hotkey: "ArrowRight", callback: () => seekForward().catch(console.error) }
		],
		{ enabled: !isDisabled, ignoreInputs: false }
	);

	return (
		<div class="relative">
			<div class="mx-auto flex w-fit items-center justify-center gap-1 opacity-70 duration-100 ease-out hover:opacity-100">
				<button
					disabled={isDisabled}
					onClick={frameBackStep}
					class="button secondary icon size-9 rounded-lg"
					title="Prev Frame ( < )"
				>
					<PrevIcon class="size-5" />
				</button>
				<button
					disabled={isDisabled}
					onClick={() => handleSegment("start")}
					class="button secondary icon size-9 rounded-lg font-medium"
					title="Set Segment Start ( [ )"
				>
					S
				</button>
				<PlayButton isDisabled={isDisabled} />
				<button
					disabled={isDisabled}
					onClick={() => handleSegment("end")}
					class="button secondary icon size-9 rounded-lg font-medium"
					title="Set Segment End ( ] )"
				>
					E
				</button>
				<button
					disabled={isDisabled}
					onClick={frameStep}
					class="button secondary icon size-9 rounded-lg"
					title="Next Frame ( > )"
				>
					<NextIcon class="size-5" />
				</button>
			</div>
			<div class="absolute right-2 bottom-1.5 flex items-end gap-2 opacity-70 duration-100 ease-out hover:opacity-100">
				<AudioSelector isDisabled={isDisabled} />
				<VolumeSlider isDisabled={isDisabled} />
			</div>
		</div>
	);
};
