import { Playback } from "@/features/editor/playback";
import { Timeline } from "@/features/editor/timeline";

import { Preview, Sidebar } from "./ui";

export const VideoEditor = () => (
	<div class="relative h-screen gap-1.5">
		<Preview />
		<div class="absolute inset-x-0 bottom-0 flex flex-col gap-1.5">
			<Playback />
			<Timeline />
		</div>
		<Sidebar />
	</div>
);
