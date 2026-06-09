import { getCurrentWindow } from "@tauri-apps/api/window";
import { toast } from "sonner";

export const toggleFullscreen = async () => {
	try {
		const app = getCurrentWindow();
		const isFullscreen = await app.isFullscreen();

		await app.setFullscreen(!isFullscreen);
	} catch (error) {
		toast.error(`Failed to toggle fullscreen: ${error}`);
	}
};

export const exitFullscreen = async () => {
	try {
		const app = getCurrentWindow();

		await app.setFullscreen(false);
	} catch (error) {
		toast.error(`Failed to exit fullscreen: ${error}`);
	}
};
