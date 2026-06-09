import { Navigate } from "react-router";

import { useVideoStore } from "@/entities/video";
import { VideoEditor } from "@/widgets/editor";

export const EditorPage = () => {
	const isLoaded = useVideoStore((s) => s.state.isLoaded);

	if (!isLoaded) {
		return <Navigate replace to="/import" />;
	}

	return <VideoEditor />;
};
