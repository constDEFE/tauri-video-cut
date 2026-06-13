import { useCallback, useEffect } from "preact/hooks";
import { toast } from "sonner";
import { useShallow } from "zustand/shallow";

import { usePlayerEvents, usePlayerProps, useVideoStore } from "@/entities/video";
import { destroyMpv, initializeMpv, loadFile, setEndOfFile } from "@/shared/lib/mpv";

import type { VideoStore } from "@/entities/video";
import type { MpvPropertyEvent } from "@/shared/lib/mpv";
import type { MpvEvent } from "tauri-plugin-libmpv-api";

const SELECT_VIDEO_STATE = (s: VideoStore) => ({
	filePath: s.state.filePath,
	isInitialized: s.state.player.isInitialized,
	isFileLoaded: s.state.player.isFileLoaded
});

const SELECT_VIDEO_ACTIONS = (s: VideoStore) => ({
	setPlayerInitialized: s.actions.player.setInitialized,
	setPlayerFileLoaded: s.actions.player.setFileLoaded,
	setPlaying: s.actions.player.setPlaying,
	setAudioTrack: s.actions.player.setSelectedAudio,
	toggleMuted: s.actions.player.toggleMuted,
	setVolume: s.actions.player.setVolume,
	setCursor: s.actions.timeline.setCursor
});

export const usePlayer = () => {
	const { filePath, isInitialized, isFileLoaded } = useVideoStore(useShallow(SELECT_VIDEO_STATE));
	const { setPlayerInitialized, setPlayerFileLoaded, setAudioTrack, setPlaying, toggleMuted, setVolume, setCursor } =
		useVideoStore(useShallow(SELECT_VIDEO_ACTIONS));

	/* init player */
	useEffect(() => {
		if (isInitialized) {
			return;
		}

		initializeMpv()
			.then(() => setPlayerInitialized(true))
			.catch((err) => toast.error(`Player init failed: ${err?.message ?? err}`));

		return () => {
			destroyMpv()
				.then(() => {
					setPlayerInitialized(false);
					setPlayerFileLoaded(false);
				})
				.catch(() => toast.error(`Failed to destroy player`));
		};
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);

	/* load file */
	useEffect(() => {
		if (!filePath || !isInitialized) {
			return;
		}

		loadFile(filePath).catch((error) => {
			/* trigger error boundary */
			throw Error("Failed to load video", { cause: error });
		});
	}, [filePath, isInitialized]);

	const eventsSub = useCallback((e: MpvEvent) => {
		switch (e.event) {
			case "start-file":
				return setPlayerFileLoaded(false);
			case "file-loaded":
				setPlayerFileLoaded(true);
				break;
		}
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);

	/* sub to file state */
	usePlayerEvents(eventsSub);

	const propsSub = useCallback(({ name, data }: MpvPropertyEvent) => {
		switch (name) {
			case "pause":
				if (typeof data !== "boolean") return;
				return setPlaying(!data);
			case "time-pos":
				if (typeof data !== "number") return;
				return setCursor(data);
			case "mute":
				if (typeof data !== "boolean") return;
				return toggleMuted(data);
			case "volume":
				if (typeof data !== "number") return;
				return setVolume(data);
			case "aid":
				if (typeof data !== "number") return;
				return setAudioTrack(data);
			case "eof-reached":
				if (typeof data !== "boolean") return;
				return setEndOfFile(data);
		}
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);

	/* sub to props */
	usePlayerProps(propsSub);

	return {
		isInitialized,
		isFileLoaded
	};
};
