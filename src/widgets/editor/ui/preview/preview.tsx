import { usePlayer } from "@/features/editor/player";
import { toggleFullscreen } from "@/shared/utils";

import { StatusView } from "./status-view";

const STATUS = {
	NOT_INITIALIZED: "Initializing video player",
	PENDING_FILE: "Loading video file..."
};

export const Preview = () => {
	const { isFileLoaded, isInitialized } = usePlayer();

	if (!isInitialized || !isFileLoaded) {
		return <StatusView isLoading>{isInitialized ? STATUS.PENDING_FILE : STATUS.NOT_INITIALIZED}</StatusView>;
	}

	return <div class="absolute inset-0" onDblClick={toggleFullscreen} />;
};
