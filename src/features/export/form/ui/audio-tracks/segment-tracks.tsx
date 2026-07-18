import { useSegmentsStore } from "@/entities/segments";
import { useVideoStore } from "@/entities/video";

import { SegmentCard } from "./segment-card";

export const SegmentTracks = () => {
	const segments = useSegmentsStore((s) => s.state.segments);
	const metadata = useVideoStore((s) => s.state.metadata);

	return (
		<div class="space-y-4">
			{!!metadata && segments.map((s, idx) => <SegmentCard segment={s} metadata={metadata} idx={idx} key={s.id} />)}
		</div>
	);
};
