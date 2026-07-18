import { useHotkey } from "@tanstack/react-hotkeys";
import { invoke } from "@tauri-apps/api/core";
import { useShallow } from "zustand/shallow";

import { useVideoStore } from "@/entities/video";
import { useWaveformStore } from "@/entities/waveform";
import { useNavigate } from "@/shared/lib/router";

import type { VideoStore } from "@/entities/video";

const SELECT_STATE = (s: VideoStore) => ({
	resetVideoStore: s.actions.reset,
	resetPlayer: s.actions.player.reset
});

export const ImportButton = () => {
	const { resetVideoStore, resetPlayer } = useVideoStore(useShallow(SELECT_STATE));
	const resetWaveformStore = useWaveformStore((s) => s.actions.reset);
	const navigate = useNavigate();

	const handleClick = () => {
		navigate("/");
		invoke("cancel_all_tasks");
		resetWaveformStore();
		resetVideoStore();
		resetPlayer();
	};

	useHotkey("Control+N", handleClick);

	return (
		<button class="button h-9 rounded font-medium" onClick={handleClick} title="Import New Video ( Ctrl+N )">
			Import New
		</button>
	);
};
