import { listen, type EventCallback } from "@tauri-apps/api/event";
import { useEffect } from "preact/hooks";
import { useShallow } from "zustand/shallow";

import { useVideoStore } from "@/entities/video";
import { useWaveformStore, WaveformController } from "@/entities/waveform";

import type { VideoStore } from "@/entities/video";
import type {
	WaveformCancelledPayload,
	WaveformChunkPayload,
	WaveformErrorPayload,
	WaveformFinishedPayload
} from "@/entities/waveform";

const SELECT_STATE = (s: VideoStore) => ({
	selectedTrackIndex: s.state.player.selectedAudio,
	filePath: s.state.filePath,
	metadata: s.state.metadata
});

export const useWaveformController = () => {
	const { filePath, metadata, selectedTrackIndex } = useVideoStore(useShallow(SELECT_STATE));

	useEffect(() => {
		if (!filePath || !metadata || metadata.audio_tracks.length === 0) {
			WaveformController.reset();
			useWaveformStore.getState().actions.reset();
			return;
		}

		const subs: Array<() => void> = [];

		const setupListener = async <T>(channel: string, handler: EventCallback<T>) => {
			try {
				const unsub = await listen(channel, handler);
				subs.push(unsub);
			} catch (err) {
				console.error(`Failed to listen to ${channel}:`, err);
			}
		};

		setupListener<WaveformChunkPayload>("waveform://chunk", WaveformController.handleChunk);
		setupListener<WaveformFinishedPayload>("waveform://finished", WaveformController.handleFinished);
		setupListener<WaveformErrorPayload>("waveform://error", WaveformController.handleError);
		setupListener<WaveformCancelledPayload>("waveform://cancelled", WaveformController.handleCancelled);

		return () => subs.forEach((unsub) => unsub());
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [filePath]);

	useEffect(() => {
		if (!filePath || !metadata || selectedTrackIndex === null) return;
		const track = metadata.audio_tracks.find((t) => t.index === selectedTrackIndex);
		if (!track) return;

		for (const trackIdx of WaveformController.activeJobIds.keys()) {
			if (trackIdx !== selectedTrackIndex) {
				WaveformController.cancelTrack(trackIdx);
			}
		}

		WaveformController.startWaveform(selectedTrackIndex, {
			videoPath: filePath,
			duration: metadata.duration
		});
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [selectedTrackIndex, filePath, metadata?.duration]);
};
