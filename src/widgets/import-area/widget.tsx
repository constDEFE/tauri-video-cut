import { open } from "@tauri-apps/plugin-dialog";

import { useFileDrop } from "@/features/import/file-drop";
import { useVideoLoader } from "@/features/import/video-loader";
import { FileVideoIcon } from "@/shared/ui/icons";
import { cn } from "@/shared/utils/cn";

const VIDEO_FORMATS = ["mp4", "mkv", "mov", "avi", "webm"];

const CN = {
	container:
		"border-secondary hover:border-accent hover:bg-surface focus-visible:outline-none group flex cursor-pointer flex-col items-center gap-4 rounded-lg border-2 border-dashed p-12 duration-100 ease-out",
	icon: "text-text group-hover:text-accent size-12 duration-100 ease-out"
};

export const ImportArea = () => {
	const { isLoading, loadVideo } = useVideoLoader();

	useFileDrop(loadVideo, VIDEO_FORMATS);

	const handleFileSelect = async () => {
		const selected = await open({
			multiple: false,
			filters: [{ name: "Video", extensions: VIDEO_FORMATS }]
		});

		if (!selected) {
			return;
		}

		await loadVideo(selected);
	};

	return (
		<div class="bg-accent-inverted grid min-h-screen place-items-center">
			<button
				onClick={handleFileSelect}
				disabled={isLoading}
				class={cn(CN.container, isLoading && "pointer-events-none opacity-50")}
			>
				<FileVideoIcon class={cn(CN.icon, isLoading && "text-accent animate-pulse")} />
				<div class="text-center">
					<p class="text-accent text-lg font-semibold">Click to select video</p>
					<p class="text-text text-sm">{VIDEO_FORMATS.join(", ").toUpperCase()}</p>
				</div>
			</button>
		</div>
	);
};
