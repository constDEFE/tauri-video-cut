import { useExport } from "@/features/export/execution";

import { formatETA } from "./lib";

import type { ExportSettings } from "@/features/export/execution";

type Props = {
	settings: ExportSettings;
};

export const ExportProgress = ({ settings }: Props) => {
	const progress = useExport(settings);

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
			<p class="text-text mt-4 text-center text-sm select-none">Please wait while your segments are being exported</p>
		</>
	);
};
