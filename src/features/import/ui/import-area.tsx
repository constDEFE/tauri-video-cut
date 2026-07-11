import { useFileDrop, useVideoLoader, VIDEO_FORMATS } from "../lib";
import { ImportButton } from "./import-button";
import { SessionModal } from "./session-modal";

export const ImportArea = () => {
	const { isLoading, loadVideo } = useVideoLoader();

	useFileDrop(loadVideo, VIDEO_FORMATS, isLoading);

	return (
		<>
			<div class="bg-accent-inverted grid min-h-screen place-items-center">
				<ImportButton isLoading={isLoading} loader={loadVideo} />
			</div>
			<SessionModal isLoading={isLoading} loader={loadVideo} />
		</>
	);
};
