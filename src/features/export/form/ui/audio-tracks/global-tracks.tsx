import { useVideoStore } from "@/entities/video";
import { Checkbox } from "@/shared/ui";

export const GlobalTracks = () => {
	const metadata = useVideoStore((s) => s.state.metadata);

	return (
		<div class="space-y-2">
			<p class="text-accent text-sm font-medium">Select tracks to include:</p>
			{metadata?.audio_tracks.map((track) => (
				<Checkbox key={track.index} defaultChecked name={`global-${track.index}`}>
					{track.name} ({track.codec})
				</Checkbox>
			))}
		</div>
	);
};
