import { invoke } from "@tauri-apps/api/core";
import { nanoid } from "nanoid";
import { useCallback, useState } from "preact/hooks";
import { useNavigate } from "react-router";
import { toast } from "sonner";

import { useSegmentsStore } from "@/entities/segments";
import { useSessionStore } from "@/entities/session";
import { calculateEstimatedSize, useVideoStore, type VideoMetadata } from "@/entities/video";
import { setAudioTrack } from "@/shared/lib/mpv";

import type { Segment } from "@/entities/segments";
import type { Session } from "@/entities/session";

const parsePersistedData = (session: Session, meta: VideoMetadata) => {
	const segments = session.segments!.reduce<Segment[]>((acc, seg) => {
		const isValid = seg.end > seg.start && seg.end <= meta.duration;

		if (!isValid) {
			return acc;
		}

		const duration = seg.end - seg.start;
		const estimatedSize = calculateEstimatedSize(meta.bitrate, duration);

		acc.push({ ...seg, duration, estimatedSize });

		return acc;
	}, []);

	const audioTracks = [meta.audio_tracks[0]?.index ?? 0];

	return { segments, audioTracks };
};

const getPlayerDefaults = (meta: VideoMetadata) => {
	const segments = [
		{
			id: nanoid(),
			start: 0,
			end: meta.duration,
			duration: meta.duration,
			estimatedSize: calculateEstimatedSize(meta.bitrate, meta.duration)
		}
	];

	const audioTracks = [meta.audio_tracks[0]?.index ?? 0];

	return { segments, audioTracks };
};

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
	const updateSession = useSessionStore((s) => s.actions.updateSession);
	const session = useSessionStore((s) => s.state.session);
	const navigate = useNavigate();

	const loadVideo = useCallback(
		async (filePath: string, fromSession?: boolean) => {
			try {
				setIsLoading(true);

				const meta = await getVideoMeta(filePath);
				const { segments, audioTracks } = fromSession ? parsePersistedData(session, meta) : getPlayerDefaults(meta);

				// @todo
				const audioTrackCb = () => {
					setAudioTrack(audioTracks[0]!);
				};

				setSegments(segments);
				setVideo(filePath, meta, audioTrackCb);
				updateSession({ file_path: filePath, segments: segments, audio_tracks: audioTracks });

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
