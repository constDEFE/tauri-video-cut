import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "preact/hooks";
import { toast } from "sonner";

import { useSegmentsStore } from "@/entities/segments";
import { useSessionStore } from "@/entities/session";
import { useVideoStore } from "@/entities/video";
import { useNavigate } from "@/shared/lib/router";

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
	const [progress, setProgress] = useState(initialState);
	const filePath = useVideoStore((s) => s.state.filePath);
	const segments = useSegmentsStore((s) => s.state.segments);
	const blank = useSessionStore((s) => s.actions.blank);

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
					blank();
					navigate("/complete", { state: { outputFiles: result.output_files } });
				} else {
					toast.error("Export failed");
					navigate("/export");
				}
			} catch (error) {
				console.error("Export error:", error);
				toast.error(`Export failed: ${error}`);

				navigate("/export");
			}
		};

		startExport();

		return () => void unsub?.();
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [settings, filePath, segments]);

	return progress;
};
