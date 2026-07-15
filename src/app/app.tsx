import "./styles/index.css";

import { useHotkeys } from "@tanstack/react-hotkeys";

import { Toaster } from "@/shared/ui";
import { exitFullscreen, toggleFullscreen } from "@/shared/utils";

import { AppErrorBoundary } from "./error-boundary";
import { usePreventContextMenu } from "./lib";
import { AppRouter } from "./router";

export const App = () => {
	usePreventContextMenu();

	useHotkeys([
		{ hotkey: "F11", callback: toggleFullscreen, options: { ignoreInputs: false } },
		{ hotkey: "F", callback: toggleFullscreen },
		{ hotkey: "Escape", callback: exitFullscreen, options: { ignoreInputs: false } }
	]);

	return (
		<AppErrorBoundary>
			<Toaster />
			<AppRouter />
		</AppErrorBoundary>
	);
};
