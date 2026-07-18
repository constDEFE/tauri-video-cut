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

export class WaveformController {
	private static _activeJobIds = new Map<number, string>();
	private static _startGenerations = new Map<number, number>();

	static get activeJobIds() {
		return this._activeJobIds;
	}

	static handleChunk({ payload }: TauriEvent<WaveformChunkPayload>) {
		useWaveformStore
			.getState()
			.actions.appendChunk(
				payload.trackIndex,
				payload.jobId,
				payload.pointOffset,
				new Uint8Array(payload.leftRms),
				new Uint8Array(payload.rightRms),
				new Uint8Array(payload.leftPeak),
				new Uint8Array(payload.rightPeak),
				payload.totalPoints
			);
	}

	static handleFinished({ payload }: TauriEvent<WaveformFinishedPayload>) {
		useWaveformStore.getState().actions.setFinished(payload.trackIndex, payload.jobId);
		if (this._activeJobIds.get(payload.trackIndex) === payload.jobId) {
			this._activeJobIds.delete(payload.trackIndex);
		}
	}

	static handleError({ payload }: TauriEvent<WaveformErrorPayload>) {
		const store = useWaveformStore.getState();
		store.actions.setError(payload.trackIndex, payload.jobId, payload.message);
		store.actions.clearTrack(payload.trackIndex);
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

		// Superseded by a newer startWaveform call while awaiting
		if (this._startGenerations.get(trackIndex) !== generation) {
			invoke("cancel_waveform", { jobId: response.jobId }).catch(() => {});
			return response;
		}

		if (response.cachedData) {
			const { leftRms, rightRms, leftPeak, rightPeak } = response.cachedData;
			const clamp = (arr: number[]) => (arr.length > response.totalPoints ? arr.slice(0, response.totalPoints) : arr);
			useWaveformStore.getState().actions.setCachedData(
				trackIndex,
				response.jobId,
				{
					left: new Uint8Array(clamp(leftRms)),
					right: new Uint8Array(clamp(rightRms)),
					peakLeft: new Uint8Array(clamp(leftPeak)),
					peakRight: new Uint8Array(clamp(rightPeak))
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
