export type WaveformChunkPayload = {
	jobId: string;
	trackIndex: number;
	chunkIndex: number;
	pointOffset: number;
	pointCount: number;
	totalPoints: number;
	progress: number;
	pointsPerEvent: number;
	leftRms: number[];
	rightRms: number[];
	leftPeak: number[];
	rightPeak: number[];
};

export type WaveformFinishedPayload = {
	jobId: string;
	trackIndex: number;
	totalPoints: number;
	decodedFrames: number;
	expectedFrames: number;
	targetRate: number;
};

export type WaveformErrorPayload = {
	jobId: string;
	trackIndex: number;
	message: string;
};

export type WaveformCancelledPayload = {
	jobId: string;
	trackIndex: number;
};

export type StartWaveformResponse = {
	jobId: string;
	totalPoints: number;
	pointsPerEvent: number;
	eventCount: number;
	targetRate: number;
	cachedData: {
		leftRms: number[];
		rightRms: number[];
		leftPeak: number[];
		rightPeak: number[];
	} | null;
};
