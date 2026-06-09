import { useHotkeys } from "@tanstack/react-hotkeys";
import { memo } from "preact/compat";
import { useShallow } from "zustand/shallow";

import { useVideoStore } from "@/entities/video";
import { setVolume as mpvSetVolume, mute } from "@/shared/lib/mpv";
import { Slider } from "@/shared/ui";
import { VolumeFullIcon, VolumeLowIcon, VolumeMutedIcon, VolumeNoneIcon } from "@/shared/ui/icons";

import type { VideoStore } from "@/entities/video";
import type { EventFor } from "@/shared/types/react";

type Props = {
	isDisabled?: boolean;
};

const SELECT_VIDEO = (s: VideoStore) => ({
	volume: s.state.player.volume,
	isMuted: s.state.player.isMuted
});

export const VolumeSlider = memo(({ isDisabled }: Props) => {
	const { isMuted, volume } = useVideoStore(useShallow(SELECT_VIDEO));

	const handleVolumeChange = (newVolume: number) => {
		mpvSetVolume(newVolume);
	};

	const handleMuteToggle = () => {
		mute(!isMuted);
	};

	const handleSliderChange = (e: EventFor<"input", "input">) => {
		handleVolumeChange(e.currentTarget.valueAsNumber);
	};

	useHotkeys(
		[
			{ hotkey: "ArrowUp", callback: () => handleVolumeChange(Math.min(volume + 10, 100)) },
			{ hotkey: "ArrowDown", callback: () => handleVolumeChange(Math.max(volume - 10, 0)) },
			{ hotkey: "M", callback: handleMuteToggle, options: { ignoreInputs: false } }
		],
		{ enabled: !isDisabled }
	);

	return (
		<div class="group flex flex-col-reverse items-center">
			<button
				disabled={isDisabled}
				onClick={handleMuteToggle}
				class="button secondary icon size-9 rounded-lg"
				title={isMuted ? "Unmute (M)" : "Mute (M)"}
			>
				{isMuted ? (
					<VolumeMutedIcon class="size-6" />
				) : volume > 50 ? (
					<VolumeFullIcon class="size-6" />
				) : volume > 0 ? (
					<VolumeLowIcon class="size-6" />
				) : (
					<VolumeNoneIcon class="size-6" />
				)}
			</button>
			<div class="flex w-full justify-center pb-2 opacity-0 duration-100 ease-out group-hover:opacity-100">
				<Slider
					disabled={isDisabled}
					type="range"
					min="0"
					max="100"
					step="1"
					value={volume}
					onChange={handleSliderChange}
					class="vertical h-24"
					title="Volume (Up, Down)"
				/>
			</div>
		</div>
	);
});
