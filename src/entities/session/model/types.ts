export type PrunedSegment = {
	id: string;
	start: number;
	end: number;
};

export type Session = {
	file_path: string | null;
	segments: PrunedSegment[] | null;
	audio_tracks: number[] | null;
};
