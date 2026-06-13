import { openPath } from "@tauri-apps/plugin-opener";
import { useLocation, useNavigate } from "react-router";
import { toast } from "sonner";

import { useVideoStore } from "@/entities/video";
import { invariant } from "@/shared/utils";

export const CompleteOutput = () => {
	const resetVideo = useVideoStore((s) => s.actions.reset);
	const location = useLocation();
	const navigate = useNavigate();

	const outputFiles: [string] = location.state.outputFiles;

	invariant(outputFiles);

	const handleClearState = () => {
		resetVideo();
		navigate("/", { replace: true });
	};

	const handleOpenFolder = async () => {
		const firstFile = outputFiles[0];
		const lastBackslash = firstFile.lastIndexOf("\\");
		const lastSlash = firstFile.lastIndexOf("/");
		const lastSep = Math.max(lastBackslash, lastSlash);

		if (lastSep === -1) {
			toast.error("Invalid file path");
			return;
		}

		const folderPath = firstFile.substring(0, lastSep);

		try {
			await openPath(folderPath);
		} catch (error) {
			console.error("Failed to open folder:", error);
			toast.error(`Failed to open folder: ${error}`);
		}
	};

	return (
		<div class="w-full max-w-xl text-center">
			<h1 class="text-accent mb-4 text-4xl font-bold select-none">Export Complete</h1>
			<p class="text-text mb-4 select-none">Your video segments have been successfully exported</p>
			<div class="surface text-accent mb-6 rounded-lg p-4 text-left">
				<p class="font-medium select-none">Output Files:</p>
				<ul class="mt-2 space-y-1 text-sm">
					{outputFiles.map((file: string) => (
						<li key={file} class="truncate">
							{file.split(/[\\/]/).pop()}
						</li>
					))}
				</ul>
			</div>
			<div class="mx-auto grid max-w-80 grid-cols-2 gap-3 *:rounded *:px-4 *:py-2">
				<button onClick={handleOpenFolder} class="button">
					Open Folder
				</button>
				<button onClick={handleClearState} class="button secondary">
					New Project
				</button>
			</div>
		</div>
	);
};
