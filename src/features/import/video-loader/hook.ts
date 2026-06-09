import { invoke } from "@tauri-apps/api/core";
import { nanoid } from "nanoid";
import { useCallback, useState } from "preact/hooks";
import { useNavigate } from "react-router";
import { toast } from "sonner";

import { useSegmentsStore } from "@/entities/segments";
import { calculateEstimatedSize, useVideoStore, type VideoMetadata } from "@/entities/video";

import type { Segment } from "@/entities/segments";

const getVideoMeta = async (filePath: string) => {
	const metaPromise = invoke<VideoMetadata>("get_video_metadata", {
		videoPath: filePath
	});

	toast.promise(metaPromise, {
		loading: "Analyzing video...",
		error: `Failed to load video`
	});

	return await metaPromise;
};

export const useVideoLoader = () => {
	const [isLoading, setIsLoading] = useState(false);
	const setVideo = useVideoStore((s) => s.actions.setVideo);
	const setSegments = useSegmentsStore((s) => s.actions.set);
	const navigate = useNavigate();

	const loadVideo = useCallback(
		async (filePath: string) => {
			try {
				setIsLoading(true);

				const meta = await getVideoMeta(filePath);

				const defaultSegment: Segment = {
					id: nanoid(),
					start: 0,
					end: meta.duration,
					duration: meta.duration,
					estimatedSize: calculateEstimatedSize(meta.bitrate, meta.duration)
				};

				setVideo(filePath, meta);
				setSegments([defaultSegment]);

				navigate("/editor", { replace: true });
			} catch (error) {
				toast.error(`Failed to load video: ${error}`);
				console.error(error);
			} finally {
				setIsLoading(false);
			}
		},
		// eslint-disable-next-line react-hooks/exhaustive-deps
		[]
	);

	return {
		isLoading,
		loadVideo
	};
};
