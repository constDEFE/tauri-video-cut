export type AudioTrack = {
	track_id: number;
	index: number,
	codec: string;
	name: string;
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
