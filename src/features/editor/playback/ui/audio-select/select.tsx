import { useShallow } from "zustand/shallow";

import { useVideoStore, type VideoStore } from "@/entities/video";
import { WaveformController } from "@/entities/waveform";
import { setAudioTrack } from "@/shared/lib/mpv";
import {
	SelectContent,
	SelectGroup,
	SelectItem,
	SelectLabel,
	SelectTrigger,
	SelectValue,
	Select as UiSelect
} from "@/shared/ui";

const TypedSelect = UiSelect<number>;

const SELECT_VIDEO = (s: VideoStore) => ({
	tracks: s.state.metadata?.audio_tracks,
	selected: s.state.player.selectedAudio,
	getAudioById: s.actions.player.getAudioById,
	filePath: s.state.filePath,
	duration: s.state.metadata?.duration
});

type Props = {
	onSelect: () => void;
};

export const Select = ({ onSelect }: Props) => {
	const { getAudioById, selected, tracks, filePath, duration } = useVideoStore(useShallow(SELECT_VIDEO));

	const handleSelect = (value: number | null) => {
		if (!value) return;
		setAudioTrack(value);
		onSelect();

		if (filePath && duration) {
			WaveformController.startWaveform(value, {
				videoPath: filePath,
				duration
			});
		}
	};

	return (
		<TypedSelect onValueChange={handleSelect} value={selected} items={tracks}>
			<SelectTrigger className="w-48">
				<SelectValue placeholder="Loading...">
					{(value: number) => (value ? getAudioById(value)?.name : "None")}
				</SelectValue>
			</SelectTrigger>
			<SelectContent alignItemWithTrigger={false} side="top" align="start">
				<SelectGroup>
					<SelectLabel>Audio Tracks</SelectLabel>
					{tracks?.map((t) => (
						<SelectItem key={t.index} value={t.index}>
							{t.name ?? `Track #${t.index}`}
						</SelectItem>
					))}
				</SelectGroup>
			</SelectContent>
		</TypedSelect>
	);
};
