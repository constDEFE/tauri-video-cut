import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

import { useExport } from "@/features/export/execution";
import { useNavigate, useRoute } from "@/shared/lib/router";

import { formatETA } from "../lib";

export const ExportProgress = () => {
	const route = useRoute();
	const navigate = useNavigate();
	const { abort, progress } = useExport(route.state.settings);

	const isProbing = progress.currentSegment === 0 && progress.completionPercent === 0;

	const handleCancel = async () => {
		try {
			abort();
			await invoke("cancel_all_tasks");
			navigate("/export");
		} catch (error) {
			let msg;

			if (error instanceof Error) msg = error.message;
			else if (typeof error === "string") msg = error;
			else msg = "Unknown";

			toast.error(`Failed to interrupt: ${msg}`);
		}
	};

	return (
		<>
			<h1 class="text-accent mb-6 text-center text-4xl font-bold select-none">Export in progress</h1>
			<div class="border-secondary bg-surface w-full max-w-md space-y-4 rounded-lg border p-6">
				<div class="text-accent">
					<div class="flex justify-between text-sm select-none">
						<p class="mb-2">Current Segment</p>
						<p class="text-accent">{Math.round(progress.completionPercent)}%</p>
					</div>
					<div class="bg-accent-inverted relative h-2 w-full overflow-hidden rounded-full">
						<div class="bg-accent h-full duration-300 ease-out" style={{ width: `${progress.completionPercent}%` }} />
					</div>
				</div>
				<div class="text-accent flex items-baseline justify-between select-none">
					<p class="text-sm">Segment Progress</p>
					<p class="text-2xl font-semibold">
						{progress.currentSegment} / {progress.totalSegments}
					</p>
				</div>
				<div class="text-accent flex items-baseline justify-between select-none">
					<p class="text-sm">Estimated Time</p>
					<p class="text-xl font-medium">{formatETA(progress.etaSeconds)}</p>
				</div>
			</div>
			<p class="text-text mt-4 text-center text-sm select-none">
				{isProbing ? "Extracting keyframes..." : "Exporting your segments..."}
			</p>
			<button class="button mx-auto mt-2 rounded-md px-6 py-2 font-medium" onClick={handleCancel}>
				Cancel
			</button>
		</>
	);
};
