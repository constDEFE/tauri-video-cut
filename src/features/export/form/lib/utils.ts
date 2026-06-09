import { isValidPrefix } from "@/shared/utils/is";

import type { ExportSettings } from "../../execution";
import type { Segment } from "@/entities/segments";
import type { VideoMetadata } from "@/entities/video";

const extractCheckedTracks = (form: HTMLFormElement, audioTracks: VideoMetadata["audio_tracks"], prefix: string) => {
	return audioTracks.reduce<number[]>((checked, track) => {
		const el = form.elements.namedItem(`${prefix}-${track.index}`) as HTMLInputElement;

		if (el.checked) {
			checked.push(track.index);
		}

		return checked;
	}, []);
};

const extractPerSegmentTracks = (
	form: HTMLFormElement,
	audioTracks: VideoMetadata["audio_tracks"],
	segments: Segment[]
) => {
	return segments.reduce<Record<Segment["id"], number[]>>((map, s) => {
		const selectedTracks = extractCheckedTracks(form, audioTracks, `per-segment-${s.id}`);

		map[s.id] = selectedTracks;

		return map;
	}, {});
};

export const parseExportFields = (
	form: HTMLFormElement,
	audioTracks: VideoMetadata["audio_tracks"],
	segments: Segment[]
) => {
	const outputField = form.elements.namedItem("output-folder") as HTMLInputElement;
	const output = outputField.value;

	if (!output) {
		return "Select output folder";
	}

	const prefixField = form.elements.namedItem("prefix") as HTMLInputElement;
	const prefix = prefixField.value;

	if (!isValidPrefix(prefix)) {
		return "Prefix must be a valid Windows file name";
	}

	const smartCutField = form.elements.namedItem("smart-cut") as HTMLInputElement;
	const smartCut = smartCutField.checked;

	const audioModeField = form.elements.namedItem("audio-mode") as HTMLInputElement;
	const audioMode = audioModeField.value as ExportSettings["audioMode"];

	const globalTracks = audioMode === "global" ? extractCheckedTracks(form, audioTracks, "global") : null;
	const perSegmentTracks = audioMode === "per-segment" ? extractPerSegmentTracks(form, audioTracks, segments) : null;

	return {
		output,
		prefix,
		smartCut,
		audioMode,
		globalTracks,
		perSegmentTracks
	} as ExportSettings;
};
