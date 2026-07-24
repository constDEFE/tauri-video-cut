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

	leftPeakUp: number[];
	leftPeakDown: number[];
	rightPeakUp: number[];
	rightPeakDown: number[];

	chunkMaxPeak: number;
};

export type WaveformFinishedPayload = {
	jobId: string;
	trackIndex: number;
	totalPoints: number;
	decodedFrames: number;
	expectedFrames: number;
	targetRate: number;
	maxLeftPeak: number;
	maxRightPeak: number;
	displayGain: number;
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
	cachedData: WaveformChunkPayload | null;
};
