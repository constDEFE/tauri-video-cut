import { usePlayer } from "@/features/editor/player";
import { toggleFullscreen } from "@/shared/utils";

import { StatusView } from "./status-view";

export const Preview = () => {
	const { isFileLoaded, isInitialized } = usePlayer();

	if (!isInitialized || !isFileLoaded) {
		return (
			<StatusView isLoading>{isInitialized ? "Loading video file..." : "Initializing video player..."}</StatusView>
		);
	}

	return <div class="absolute inset-0" onDblClick={toggleFullscreen} />;
};
