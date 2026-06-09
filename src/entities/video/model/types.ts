export type AudioTrack = {
	index: number;
	codec: string;
	language?: string;
	channels: number;
};

export type VideoMetadata = {
	duration: number;
	width: number;
	height: number;
	video_codec: string;
	bitrate: number;
	fps: number;
	audio_tracks: AudioTrack[];
};

export type MpvAudioTrack = {
	id: number;
	name: string;
};
