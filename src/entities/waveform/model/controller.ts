import { invoke } from "@tauri-apps/api/core";

import { useWaveformStore } from "./store";

import type {
	StartWaveformResponse,
	WaveformCancelledPayload,
	WaveformChunkPayload,
	WaveformErrorPayload,
	WaveformFinishedPayload
} from "./types";
import type { Event as TauriEvent } from "@tauri-apps/api/event";

const toU8 = (data: number[] | Uint8Array | null | undefined): Uint8Array => {
	if (!data) return new Uint8Array(0);

	return data instanceof Uint8Array ? data : new Uint8Array(data);
};

export class WaveformController {
	private static _activeJobIds = new Map<number, string>();
	private static _startGenerations = new Map<number, number>();

	static get activeJobIds() {
		return this._activeJobIds;
	}

	static handleChunk({ payload }: TauriEvent<WaveformChunkPayload>) {
		useWaveformStore.getState().actions.appendChunk(
			payload.trackIndex,
			payload.jobId,
			payload.pointOffset,
			{
				rmsLeft: toU8(payload.leftRms),
				rmsRight: toU8(payload.rightRms),
				leftUp: toU8(payload.leftPeakUp),
				leftDown: toU8(payload.leftPeakDown),
				rightUp: toU8(payload.rightPeakUp),
				rightDown: toU8(payload.rightPeakDown)
			},
			payload.totalPoints,
			payload.chunkMaxPeak,
			payload.displayGain
		);
	}

	static handleFinished({ payload }: TauriEvent<WaveformFinishedPayload>) {
		useWaveformStore
			.getState()
			.actions.setFinished(
				payload.trackIndex,
				payload.jobId,
				payload.maxLeftPeak,
				payload.maxRightPeak,
				payload.displayGain
			);

		if (this._activeJobIds.get(payload.trackIndex) === payload.jobId) {
			this._activeJobIds.delete(payload.trackIndex);
		}
	}

	static handleError({ payload }: TauriEvent<WaveformErrorPayload>) {
		const store = useWaveformStore.getState();
		const track = store.getters.trackByIdx(payload.trackIndex);

		if (track?.jobId !== payload.jobId) {
			return;
		}

		store.actions.setError(payload.trackIndex, payload.jobId, payload.message);

		if (this._activeJobIds.get(payload.trackIndex) === payload.jobId) {
			this._activeJobIds.delete(payload.trackIndex);
		}
	}

	static handleCancelled({ payload }: TauriEvent<WaveformCancelledPayload>) {
		if (this._activeJobIds.get(payload.trackIndex) === payload.jobId) {
			this._activeJobIds.delete(payload.trackIndex);
		}
	}

	static async startWaveform(
		trackIndex: number,
		request: {
			videoPath: string;
			duration: number;
			targetRate?: number;
			audioTracksSampleRate?: number;
		}
	) {
		const existing = useWaveformStore.getState().getters.trackByIdx(trackIndex);

		if (existing?.finished && existing.filledPoints >= existing.totalPoints) {
			return;
		}

		const generation = (this._startGenerations.get(trackIndex) ?? 0) + 1;
		this._startGenerations.set(trackIndex, generation);
		this.cancelTrack(trackIndex);

		const emitted = useWaveformStore.getState().getters.emittedPointsByTrackIdx(trackIndex);
		const response = await invoke<StartWaveformResponse>("stream_waveform", {
			request: {
				videoPath: request.videoPath,
				trackIndex,
				duration: request.duration,
				targetRate: request.targetRate,
				audioTracksSampleRate: request.audioTracksSampleRate,
				resumeFromPoint: emitted > 0 ? emitted : undefined
			}
		});

		if (this._startGenerations.get(trackIndex) !== generation) {
			invoke("cancel_waveform", { jobId: response.jobId }).catch(() => {});
			return response;
		}

		if (response.cachedData) {
			const clamp = (arr: number[]) => (arr.length > response.totalPoints ? arr.slice(0, response.totalPoints) : arr);
			useWaveformStore.getState().actions.setCachedData(
				trackIndex,
				response.jobId,
				{
					rmsLeft: toU8(clamp(response.cachedData.leftRms)),
					rmsRight: toU8(clamp(response.cachedData.rightRms)),
					leftUp: toU8(clamp(response.cachedData.leftPeakUp)),
					leftDown: toU8(clamp(response.cachedData.leftPeakDown)),
					rightUp: toU8(clamp(response.cachedData.rightPeakUp)),
					rightDown: toU8(clamp(response.cachedData.rightPeakDown))
				},
				response.totalPoints
			);
		} else {
			useWaveformStore.getState().actions.beginJob(trackIndex, response.jobId, response.totalPoints);
			this._activeJobIds.set(trackIndex, response.jobId);
		}
		return response;
	}

	static cancelTrack(trackIndex: number) {
		const jobId = this._activeJobIds.get(trackIndex);
		if (jobId) {
			invoke("cancel_waveform", { jobId }).catch(() => {});
		}
		this._activeJobIds.delete(trackIndex);
	}

	static reset() {
		for (const [, jobId] of this._activeJobIds.entries()) {
			invoke("cancel_waveform", { jobId }).catch(() => {});
		}
		this._activeJobIds.clear();
		this._startGenerations.clear();
	}
}
