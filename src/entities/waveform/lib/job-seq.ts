// `wf-{millis}-{counter}`
export const jobSeq = (jobId: string): number => {
	const parts = jobId.split("-");
	return Number(parts[parts.length - 1]) || 0;
};
