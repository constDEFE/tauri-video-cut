import { formatTime } from "@/entities/video";
import { Checkbox } from "@/shared/ui";

import type { Segment } from "@/entities/segments";
import type { VideoStore } from "@/entities/video";

type Props = {
	idx: number;
	segment: Segment;
	metadata: NonNullable<VideoStore["state"]["metadata"]>;
};

export const SegmentCard = ({ metadata, idx, segment }: Props) => {
	const start = formatTime(segment.start, metadata.fps, false);
	const end = formatTime(segment.end, metadata.fps, false);

	return (
		<div key={segment.id} class="border-secondary bg-accent-inverted rounded border p-3">
			<p class="text-accent mb-2 font-medium">
				Segment {idx + 1} ({start} - {end})
			</p>
			<div class="space-y-2">
				{metadata.audio_tracks.map((track) => (
					<Checkbox name={`per-segment-${segment.id}-${track.index}`} defaultChecked key={track.index}>
						Track {track.index}: {track.codec}
					</Checkbox>
				))}
			</div>
		</div>
	);
};
