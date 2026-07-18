import { create } from "zustand";

import { jobSeq } from "../lib";

type RawTrackData = {
	left: Uint8Array;
	right: Uint8Array;
	peakLeft: Uint8Array;
	peakRight: Uint8Array;
};

type TrackState = {
	jobId: string;
	data: RawTrackData;
	totalPoints: number;
	filledPoints: number;
	finished: boolean;
	error: string | null;
};

type State = {
	waveformUpdateCounter: number;
};

type Private = {
	_tracks: Map<number, TrackState>;
	_emittedPoints: Map<number, number>;
};

type Getters = {
	trackByIdx: (trackIndex: number) => TrackState | undefined;
	waveformByTrackIdx: (trackIndex: number) => { left: Uint8Array; right: Uint8Array; totalPoints: number } | undefined;
	emittedPointsByTrackIdx: (trackIndex: number) => number;
};

type Actions = {
	beginJob: (trackIndex: number, jobId: string, totalPoints: number) => void;
	appendChunk: (
		trackIndex: number,
		jobId: string,
		pointOffset: number,
		left: Uint8Array,
		right: Uint8Array,
		peakLeft: Uint8Array,
		peakRight: Uint8Array,
		totalPoints: number
	) => void;
	clearTrack: (trackIndex: number) => void;
	setCachedData: (trackIndex: number, jobId: string, data: RawTrackData, totalPoints: number) => void;
	setFinished: (trackIndex: number, jobId: string) => void;
	setError: (trackIndex: number, jobId: string, message: string) => void;
	reset: () => void;
};

export type WaveformStore = {
	state: State;
	private: Private;
	getters: Getters;
	actions: Actions;
};

const allocBuffers = (totalPoints: number): RawTrackData => ({
	left: new Uint8Array(totalPoints),
	right: new Uint8Array(totalPoints),
	peakLeft: new Uint8Array(totalPoints),
	peakRight: new Uint8Array(totalPoints)
});

export const useWaveformStore = create<WaveformStore>((set, get) => ({
	state: {
		waveformUpdateCounter: 0
	},
	private: {
		_tracks: new Map(),
		_emittedPoints: new Map()
	},
	getters: {
		trackByIdx: (trackIdx) => {
			return get().private._tracks.get(trackIdx);
		},
		waveformByTrackIdx: (trackIdx) => {
			const track = get().private._tracks.get(trackIdx);

			if (!track || track.filledPoints === 0) {
				return undefined;
			}

			return {
				left: track.data.left.subarray(0, track.filledPoints),
				right: track.data.right.subarray(0, track.filledPoints),
				totalPoints: track.totalPoints
			};
		},
		emittedPointsByTrackIdx: (trackIdx) => {
			return get().private._emittedPoints.get(trackIdx) ?? 0;
		}
	},
	actions: {
		beginJob: (trackIndex, jobId, totalPoints) => {
			const store = get();
			const track = store.private._tracks.get(trackIndex);

			if (track && (track.jobId === jobId || jobSeq(track.jobId) > jobSeq(jobId))) {
				return;
			}

			const trackBody = track
				? { ...track, jobId }
				: { jobId, data: allocBuffers(totalPoints), totalPoints, filledPoints: 0, finished: false, error: null };

			if (!track) {
				store.private._emittedPoints.set(trackIndex, 0);
			}

			store.private._tracks.set(trackIndex, trackBody);
		},
		clearTrack: (trackIndex) => {
			const store = get();

			store.private._tracks.delete(trackIndex);
			store.private._emittedPoints.delete(trackIndex);

			set({ state: { waveformUpdateCounter: store.state.waveformUpdateCounter + 1 } });
		},
		appendChunk: (trackIndex, jobId, pointOffset, left, right, peakLeft, peakRight, totalPoints) => {
			const store = get();
			let track = store.private._tracks.get(trackIndex);

			if (!track) {
				track = {
					jobId,
					data: allocBuffers(totalPoints),
					totalPoints,
					filledPoints: 0,
					finished: false,
					error: null
				};
			} else if (track.jobId !== jobId) {
				if (jobSeq(track.jobId) > jobSeq(jobId)) {
					return;
				}

				track = { ...track, jobId, finished: false, error: null };
			}

			if (pointOffset < 0 || pointOffset + left.length > track.totalPoints) {
				return;
			}

			track.data.left.set(left, pointOffset);
			track.data.right.set(right, pointOffset);
			track.data.peakLeft.set(peakLeft, pointOffset);
			track.data.peakRight.set(peakRight, pointOffset);

			const filledPoints = Math.max(track.filledPoints, pointOffset + left.length);

			store.private._tracks.set(trackIndex, { ...track, filledPoints });
			store.private._emittedPoints.set(trackIndex, filledPoints);

			set({ state: { waveformUpdateCounter: store.state.waveformUpdateCounter + 1 } });
		},
		setCachedData: (trackIndex, jobId, data, totalPoints) => {
			const store = get();
			const track = store.private._tracks.get(trackIndex);

			if (track && jobSeq(track.jobId) > jobSeq(jobId)) {
				return;
			}

			store.private._emittedPoints.set(trackIndex, data.left.length);
			store.private._tracks.set(trackIndex, {
				jobId,
				data,
				totalPoints,
				filledPoints: data.left.length,
				finished: true,
				error: null
			});

			set({ state: { waveformUpdateCounter: store.state.waveformUpdateCounter + 1 } });
		},
		setFinished: (trackIndex, jobId) => {
			const store = get();
			const track = store.private._tracks.get(trackIndex);

			if (!track || track.jobId !== jobId) {
				return;
			}

			store.private._tracks.set(trackIndex, { ...track, finished: true });
		},
		setError: (trackIndex, jobId, message) => {
			const store = get();
			const track = store.private._tracks.get(trackIndex);

			if (!track || track.jobId !== jobId) {
				return;
			}

			store.private._tracks.set(trackIndex, { ...track, error: message });
		},
		reset: () => {
			const store = get();

			store.private._tracks.clear();
			store.private._emittedPoints.clear();

			set({ state: { waveformUpdateCounter: 0 } });
		}
	}
}));
