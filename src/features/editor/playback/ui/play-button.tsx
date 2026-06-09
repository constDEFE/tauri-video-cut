import { useHotkey } from "@tanstack/react-hotkeys";
import { memo } from "preact/compat";

import { useVideoStore } from "@/entities/video";
import { pause, play } from "@/shared/lib/mpv";
import { PauseIcon, PlayIcon } from "@/shared/ui/icons";

type Props = {
	isDisabled?: boolean;
};

export const PlayButton = memo(({ isDisabled }: Props) => {
	const isPlaying = useVideoStore((s) => s.state.player.isPlaying);

	const handlePlayPause = () => {
		const fn = isPlaying ? pause : play;
		fn().catch(console.error);
	};

	useHotkey("Space", handlePlayPause, { ignoreInputs: false, enabled: !isDisabled });

	return (
		<button
			onClick={handlePlayPause}
			disabled={isDisabled}
			class="button secondary icon size-12 rounded-full"
			title="Play/Pause (Space)"
		>
			{isPlaying ? <PauseIcon class="size-6" /> : <PlayIcon class="size-6" />}
		</button>
	);
});
