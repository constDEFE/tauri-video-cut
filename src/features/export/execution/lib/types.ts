import type { Segment } from "@/entities/segments";

type BaseSettings = {
	output: string;
	prefix: string;
	smartCut: boolean;
};

type GlobalSettings = {
	audioMode: "global";
	globalTracks: number[];
	perSegmentTracks: null;
};

type PerSegmentSettings = {
	audioMode: "per-segment";
	globalTracks: null;
	perSegmentTracks: Record<Segment["id"], number[]>;
};

export type ExportSettings = BaseSettings & (GlobalSettings | PerSegmentSettings);
