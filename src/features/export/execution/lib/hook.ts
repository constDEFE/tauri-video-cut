import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "preact/hooks";
import { useNavigate } from "react-router";
import { toast } from "sonner";

import { useSegmentsStore } from "@/entities/segments";
import { useVideoStore } from "@/entities/video";

import type { ExportSettings } from "./types";
import type { Event } from "@tauri-apps/api/event";

type ExportProgress = {
	current_segment: number;
	total_segments: number;
	current_segment_progress: number;
	eta_seconds: number;
};

type SegmentExportRequest = {
	start: number;
	end: number;
	audio_tracks: number[];
};

type ExportRequest = {
	video_path: string;
	segments: SegmentExportRequest[];
	output_folder: string;
	file_prefix: string;
	smart_cut: boolean;
};

type ExportResult = {
	success: boolean;
	output_files: string[];
};

type ProgressState = {
	currentSegment: number;
	totalSegments: number;
	completionPercent: number;
	etaSeconds: number;
};

const initialState: ProgressState = {
	currentSegment: 0,
	totalSegments: 0,
	completionPercent: 0,
	etaSeconds: 0
};

export const useExport = (settings: ExportSettings) => {
	const filePath = useVideoStore((s) => s.state.filePath);
	const segments = useSegmentsStore((s) => s.state.segments);
	const [progress, setProgress] = useState(initialState);

	const navigate = useNavigate();

	const sub = (e: Event<ExportProgress>) => {
		setProgress({
			currentSegment: e.payload.current_segment,
			totalSegments: e.payload.total_segments,
			completionPercent: e.payload.current_segment_progress,
			etaSeconds: e.payload.eta_seconds
		});
	};

	useEffect(() => {
		if (!settings || !filePath) {
			return;
		}

		let unsub: () => void;

		const startExport = async () => {
			try {
				unsub = await listen<ExportProgress>("export-progress", sub);

				const request: ExportRequest = {
					video_path: filePath,
					output_folder: settings.output,
					file_prefix: settings.prefix,
					smart_cut: settings.smartCut,
					segments: segments.map((seg) => ({
						start: seg.start,
						end: seg.end,
						audio_tracks: settings.audioMode === "global" ? settings.globalTracks : settings.perSegmentTracks[seg.id]!
					}))
				};

				setProgress({
					currentSegment: 0,
					totalSegments: segments.length,
					completionPercent: 0,
					etaSeconds: 0
				});

				const result = await invoke<ExportResult>("export_segments", { request });

				if (result.success) {
					navigate("/complete", { replace: true, state: { outputFiles: result.output_files } });
				} else {
					toast.error("Export failed");
					navigate("/export", { replace: true });
				}
			} catch (error) {
				console.error("Export error:", error);
				toast.error(`Export failed: ${error}`);

				navigate("/export", { replace: true });
			}
		};

		startExport();

		return () => void unsub?.();
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [settings, filePath, segments]);

	return progress;
};
