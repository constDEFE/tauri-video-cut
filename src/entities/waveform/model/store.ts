import { create } from "zustand";

import { jobSeq } from "../lib";

type RawTrackData = {
	rmsLeft: Uint8Array;
	rmsRight: Uint8Array;
	leftUp: Uint8Array;
	leftDown: Uint8Array;
	rightUp: Uint8Array;
	rightDown: Uint8Array;
};

type TrackState = {
	jobId: string;
	data: RawTrackData;
	totalPoints: number;
	filledPoints: number;
	finished: boolean;
	error: string | null;
	maxPeak: number;
	displayGain: number;
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
	waveformByTrackIdx: (trackIndex: number) =>
		| {
				rmsLeft: Uint8Array;
				rmsRight: Uint8Array;
				leftUp: Uint8Array;
				leftDown: Uint8Array;
				rightUp: Uint8Array;
				rightDown: Uint8Array;
				totalPoints: number;
				maxPeak: number;
				displayGain: number;
		  }
		| undefined;
	emittedPointsByTrackIdx: (trackIndex: number) => number;
};

type WaveformChunkArrays = {
	rmsLeft: Uint8Array;
	rmsRight: Uint8Array;
	leftUp: Uint8Array;
	leftDown: Uint8Array;
	rightUp: Uint8Array;
	rightDown: Uint8Array;
};

type Actions = {
	beginJob: (trackIndex: number, jobId: string, totalPoints: number) => void;
	appendChunk: (
		trackIndex: number,
		jobId: string,
		pointOffset: number,
		arrays: WaveformChunkArrays,
		totalPoints: number,
		chunkMaxPeak?: number
	) => void;
	clearTrack: (trackIndex: number) => void;
	setCachedData: (trackIndex: number, jobId: string, data: RawTrackData, totalPoints: number) => void;
	setFinished: (
		trackIndex: number,
		jobId: string,
		maxLeftPeak?: number,
		maxRightPeak?: number,
		displayGain?: number
	) => void;
	setError: (trackIndex: number, jobId: string, message: string) => void;
	reset: () => void;
};

export type WaveformStore = {
	state: State;
	private: Private;
	getters: Getters;
	actions: Actions;
};

const WAVEFORM_TARGET_AMP = 0.9;
const WAVEFORM_MAX_GAIN = 4;

const computeDisplayGain = (maxPeak: number): number => {
	if (!maxPeak || maxPeak <= 0) return 1;
	return Math.min((WAVEFORM_TARGET_AMP * 255) / maxPeak, WAVEFORM_MAX_GAIN);
};

const computeMaxPeak = (up: Uint8Array, down: Uint8Array, up2?: Uint8Array, down2?: Uint8Array): number => {
	let max = 0;
	for (let i = 0; i < up.length; i++) {
		const u = up[i]!;
		const d = down[i]!;
		if (u > max) max = u;
		if (d > max) max = d;
	}
	if (up2 && down2) {
		for (let i = 0; i < up2.length; i++) {
			const u = up2[i]!;
			const d = down2[i]!;
			if (u > max) max = u;
			if (d > max) max = d;
		}
	}
	return max;
};

const allocBuffers = (totalPoints: number): RawTrackData => ({
	rmsLeft: new Uint8Array(totalPoints),
	rmsRight: new Uint8Array(totalPoints),
	leftUp: new Uint8Array(totalPoints),
	leftDown: new Uint8Array(totalPoints),
	rightUp: new Uint8Array(totalPoints),
	rightDown: new Uint8Array(totalPoints)
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
				rmsLeft: track.data.rmsLeft.subarray(0, track.filledPoints),
				rmsRight: track.data.rmsRight.subarray(0, track.filledPoints),
				leftUp: track.data.leftUp.subarray(0, track.filledPoints),
				leftDown: track.data.leftDown.subarray(0, track.filledPoints),
				rightUp: track.data.rightUp.subarray(0, track.filledPoints),
				rightDown: track.data.rightDown.subarray(0, track.filledPoints),
				totalPoints: track.totalPoints,
				maxPeak: track.maxPeak,
				displayGain: track.displayGain
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

			const trackBody: TrackState = track
				? {
						...track,
						jobId,
						maxPeak: track.maxPeak ?? 0,
						displayGain: track.displayGain ?? 1
					}
				: {
						jobId,
						data: allocBuffers(totalPoints),
						totalPoints,
						filledPoints: 0,
						finished: false,
						error: null,
						maxPeak: 0,
						displayGain: 1
					};

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
		appendChunk: (trackIndex, jobId, pointOffset, arrays, totalPoints, chunkMaxPeak) => {
			const store = get();
			let track = store.private._tracks.get(trackIndex);

			if (!track) {
				track = {
					jobId,
					data: allocBuffers(totalPoints),
					totalPoints,
					filledPoints: 0,
					finished: false,
					error: null,
					maxPeak: 0,
					displayGain: 1
				};
			} else if (track.jobId !== jobId) {
				if (jobSeq(track.jobId) > jobSeq(jobId)) {
					return;
				}

				track = { ...track, jobId, finished: false, error: null };
			}

			if (pointOffset < 0 || pointOffset + arrays.leftUp.length > track.totalPoints) {
				return;
			}

			track.data.rmsLeft.set(arrays.rmsLeft, pointOffset);
			track.data.rmsRight.set(arrays.rmsRight, pointOffset);
			track.data.leftUp.set(arrays.leftUp, pointOffset);
			track.data.leftDown.set(arrays.leftDown, pointOffset);
			track.data.rightUp.set(arrays.rightUp, pointOffset);
			track.data.rightDown.set(arrays.rightDown, pointOffset);

			const filledPoints = Math.max(track.filledPoints, pointOffset + arrays.leftUp.length);

			let maxPeak = track.maxPeak ?? 0;
			if (typeof chunkMaxPeak === "number" && chunkMaxPeak > maxPeak) {
				maxPeak = chunkMaxPeak;
			}

			store.private._tracks.set(trackIndex, {
				...track,
				filledPoints,
				maxPeak,
				displayGain: computeDisplayGain(maxPeak)
			});

			store.private._emittedPoints.set(trackIndex, filledPoints);

			set({ state: { waveformUpdateCounter: store.state.waveformUpdateCounter + 1 } });
		},
		setCachedData: (trackIndex, jobId, data, totalPoints) => {
			const store = get();
			const track = store.private._tracks.get(trackIndex);

			if (track && jobSeq(track.jobId) > jobSeq(jobId)) {
				return;
			}

			const maxPeak = computeMaxPeak(data.leftUp, data.leftDown, data.rightUp, data.rightDown);

			store.private._emittedPoints.set(trackIndex, data.leftUp.length);
			store.private._tracks.set(trackIndex, {
				jobId,
				data,
				totalPoints,
				filledPoints: data.leftUp.length,
				finished: true,
				error: null,
				maxPeak,
				displayGain: computeDisplayGain(maxPeak)
			});

			set({ state: { waveformUpdateCounter: store.state.waveformUpdateCounter + 1 } });
		},
		setFinished: (trackIndex, jobId, maxLeftPeak, maxRightPeak, backendDisplayGain) => {
			const store = get();
			const track = store.private._tracks.get(trackIndex);

			if (!track || track.jobId !== jobId) {
				return;
			}

			const backendMaxPeak = Math.max(maxLeftPeak ?? 0, maxRightPeak ?? 0);
			const maxPeak = Math.max(track.maxPeak ?? 0, backendMaxPeak);

			const displayGain = maxPeak > 0 ? computeDisplayGain(maxPeak) : (backendDisplayGain ?? 1);

			store.private._tracks.set(trackIndex, {
				...track,
				finished: true,
				maxPeak,
				displayGain
			});

			set({ state: { waveformUpdateCounter: store.state.waveformUpdateCounter + 1 } });
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
