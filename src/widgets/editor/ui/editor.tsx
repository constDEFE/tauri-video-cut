import { Playback } from "@/features/editor/playback";
import { Timeline } from "@/features/editor/timeline";
import { useWaveformController } from "@/features/editor/waveform";

import { Preview } from "./preview";
import { Sidebar } from "./sidebar";

export const VideoEditor = () => {
	useWaveformController();

	return (
		<div class="relative h-screen gap-1.5">
			<Preview />
			<div class="absolute inset-x-0 bottom-0 flex flex-col gap-1.5">
				<Playback />
				<Timeline />
			</div>
			<Sidebar />
		</div>
	);
};
