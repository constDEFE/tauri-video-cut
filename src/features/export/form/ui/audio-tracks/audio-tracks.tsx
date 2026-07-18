import { useState } from "preact/hooks";

import { GlobalTracks } from "./global-tracks";
import { ModeSelect } from "./mode-select";
import { SegmentTracks } from "./segment-tracks";

import type { ExportSettings } from "@/features/export/execution";

type Props = {
	defaultValue: ExportSettings["audioMode"];
};

export const AudioTracks = ({ defaultValue }: Props) => {
	const [mode, setMode] = useState(defaultValue);

	return (
		<div class="surface rounded-lg p-4">
			<h2 class="text-accent mb-3 text-lg font-semibold select-none">Audio Tracks</h2>
			<ModeSelect onChange={setMode} value={mode} />
			{mode === "global" && <GlobalTracks />}
			{mode === "per-segment" && <SegmentTracks />}
		</div>
	);
};
