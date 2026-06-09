import { create } from "zustand";

import type { MpvAudioTrack, VideoMetadata } from "./types";

type State = {
	player: {
		isInitialized: boolean;
		isFileLoaded: boolean;
		audioTracks: MpvAudioTrack[];
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
	_audioTracksMap: Map<MpvAudioTrack["id"], MpvAudioTrack>;
};

type Actions = {
	player: {
		setAudioTracks: (tracks: MpvAudioTrack[]) => void;
		setSelectedAudio: (id: number) => void;
		setInitialized: (initialized: boolean) => void;
		setFileLoaded: (loaded: boolean) => void;
		getAudioById: (id: number) => MpvAudioTrack;
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
		audioTracks: [],
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
			setAudioTracks: (tracks) => {
				const store = get();

				store.private._audioTracksMap.clear();

				tracks.forEach((t) => {
					store.private._audioTracksMap.set(t.id, t);
				});

				set((s) => ({ state: { ...s.state, player: { ...s.state.player, audioTracks: tracks, selectedAudio: 0 } } }));
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
			set((s) => ({ state: { ...s.state, filePath, metadata, isLoaded: true } }));
		},
		reset: () => {
			set({ state: INITIAL_STATE });
		}
	}
}));
