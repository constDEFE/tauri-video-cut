import type { AudioTrack } from "@/shared/types/common";

export type AudioWaveform = {
	left: Uint8Array;
	right?: Uint8Array | undefined;
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
