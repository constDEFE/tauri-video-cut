import type { AudioTrack } from "@/shared/types/common";

export type AudioWaveform = {
	rmsLeft: Uint8Array;
	rmsRight?: Uint8Array | undefined;

	leftUp?: Uint8Array | undefined;
	leftDown?: Uint8Array | undefined;
	rightUp?: Uint8Array | undefined;
	rightDown?: Uint8Array | undefined;

	totalPoints?: number | undefined;
	maxPeak?: number | undefined;
	displayGain?: number | undefined;
};

export type RawAudioWaveform = {
	left: number[];
	right?: number[];
};

export type VideoMetadata = {
	duration: number;
	width: number;
	height: number;
	video_codec: string;
	bitrate: number;
	fps: number;
	audio_tracks: AudioTrack[];
	waveforms?: RawAudioWaveform[];
};
