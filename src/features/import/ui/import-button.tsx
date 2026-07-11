import { open } from "@tauri-apps/plugin-dialog";

import { FileVideoIcon } from "@/shared/ui/icons";
import { cn } from "@/shared/utils";

import { VIDEO_FORMATS } from "../lib";

const CN = {
	container:
		"border-secondary hover:border-accent hover:bg-surface focus-visible:outline-none group flex cursor-pointer flex-col items-center gap-4 rounded-lg border-2 border-dashed p-12 duration-100 ease-out",
	icon: "text-text group-hover:text-accent size-12 duration-100 ease-out"
};

type Props = {
	isLoading?: boolean;
	loader: (path: string, fromSession?: boolean) => Promise<void>;
};

export const ImportButton = ({ isLoading, loader }: Props) => {
	const handleFileSelect = async () => {
		const selected = await open({
			multiple: false,
			filters: [{ name: "Video", extensions: VIDEO_FORMATS }]
		});

		if (!selected) {
			return;
		}

		await loader(selected);
	};

	return (
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
	);
};
