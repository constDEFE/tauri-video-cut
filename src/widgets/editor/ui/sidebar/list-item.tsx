import { useSegmentsStore } from "@/entities/segments";
import { formatFileSize, formatTime, useVideoStore } from "@/entities/video";
import { cn } from "@/shared/utils";

import type { Segment } from "@/entities/segments";

type Props = {
	segment: Segment;
	idx: number;
	isSelected: boolean;
};

export const ListItem = ({ segment, idx, isSelected }: Props) => {
	const metadata = useVideoStore((s) => s.state.metadata);
	const select = useSegmentsStore((s) => s.actions.select);

	const handleClick = () => {
		select(segment.id);
	};

	const startTime = formatTime(segment.start, metadata?.fps);
	const endTime = formatTime(segment.end, metadata?.fps);
	const durationTime = formatTime(segment.duration, metadata?.fps);
	const sizeText = formatFileSize(segment.estimatedSize);

	const wrapCn = cn(
		"rounded border border-secondary p-3 duration-100 ease-out hover:bg-surface",
		isSelected && "bg-surface",
		!isSelected && "cursor-pointer"
	);

	return (
		<div onClick={handleClick} class={wrapCn}>
			<div class="text-accent mb-1 font-semibold">Segment {idx + 1}</div>
			<div class="text-text text-sm">
				<div>Duration: {durationTime}</div>
				<div>Size: ~{sizeText}</div>
				<div class="mt-1 font-mono text-xs">
					{startTime}
					<br />
					{endTime}
				</div>
			</div>
		</div>
	);
};
