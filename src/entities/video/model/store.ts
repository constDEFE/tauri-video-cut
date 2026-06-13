import { create } from "zustand";

import type { AudioTrack, VideoMetadata } from "./types";

type State = {
	player: {
		isInitialized: boolean;
		isFileLoaded: boolean;
		selectedAudio: number;
		isPlaying: boolean;
		isMuted: boolean;
		volume: number;
	};
	timeline: {
		cursor: number;
	};
	filePath: string | null;
	metadata: VideoMetadata | null;
	isLoaded: boolean;
};

type Private = {
	_audioTracksMap: Map<AudioTrack["index"], AudioTrack>;
};

type Actions = {
	player: {
		setSelectedAudio: (id: number) => void;
		setInitialized: (initialized: boolean) => void;
		setFileLoaded: (loaded: boolean) => void;
		getAudioById: (id: number) => AudioTrack;
		setPlaying: (playing: boolean) => void;
		setPaused: (paused: boolean) => void;
		setVolume: (volume: number) => void;
		toggleMuted: (muted: boolean) => void;
		reset: () => void;
	};
	timeline: {
		setCursor: (position: number) => void;
	};
	setVideo: (filePath: string, metadata: VideoMetadata) => void;
	reset: () => void;
};

export type VideoStore = {
	state: State;
	private: Private;
	actions: Actions;
};

const INITIAL_STATE: State = {
	player: {
		selectedAudio: 1,
		isPlaying: false,
		isMuted: false,
		volume: 100,
		isInitialized: false,
		isFileLoaded: false
	},
	timeline: {
		cursor: 0
	},
	filePath: null,
	metadata: null,
	isLoaded: false
};

export const useVideoStore = create<VideoStore>((set, get) => ({
	state: INITIAL_STATE,
	private: {
		_audioTracksMap: new Map()
	},
	actions: {
		player: {
			reset: () => {
				set((s) => ({ state: { ...s.state, player: INITIAL_STATE.player, timeline: { cursor: 0 } } }));
			},
			getAudioById: (id) => {
				const store = get();
				return store.private._audioTracksMap.get(id)!;
			},
			setSelectedAudio: (id) => {
				set((s) => ({ state: { ...s.state, player: { ...s.state.player, selectedAudio: id } } }));
			},
			setInitialized: (initialized) => {
				set((s) => ({ state: { ...s.state, player: { ...s.state.player, isInitialized: initialized } } }));
			},
			setFileLoaded: (loaded) => {
				set((s) => ({ state: { ...s.state, player: { ...s.state.player, isFileLoaded: loaded } } }));
			},
			setPlaying: (playing) => {
				set((s) => ({ state: { ...s.state, player: { ...s.state.player, isPlaying: playing } } }));
			},
			setPaused: (paused) => {
				set((s) => ({ state: { ...s.state, player: { ...s.state.player, isPlaying: !paused } } }));
			},
			setVolume: (volume) => {
				set((s) => ({ state: { ...s.state, player: { ...s.state.player, volume } } }));
			},
			toggleMuted: (muted) => {
				set((s) => ({
					state: { ...s.state, player: { ...s.state.player, isMuted: muted ?? !s.state.player.isMuted } }
				}));
			}
		},
		timeline: {
			setCursor: (position) => {
				set((s) => ({ state: { ...s.state, timeline: { cursor: position } } }));
			}
		},
		setVideo: (filePath, metadata) => {
			const store = get();

			store.private._audioTracksMap.clear();

			metadata.audio_tracks.forEach((t) => {
				store.private._audioTracksMap.set(t.index, t);
			});

			console.debug({ metadata })

			set((s) => ({
				state: {
					...s.state,
					filePath,
					metadata,
					isLoaded: true,
					player: { ...s.state.player, selectedAudio: metadata.audio_tracks[0]?.index ?? 0 }
				}
			}));
		},
		reset: () => {
			set({ state: INITIAL_STATE });
		}
	}
}));
