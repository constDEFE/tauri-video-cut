import "./styles/index.css";

import { useHotkeys } from "@tanstack/react-hotkeys";
import { RouterProvider } from "react-router";

import { Toaster } from "@/shared/ui";
import { exitFullscreen, toggleFullscreen } from "@/shared/utils";

import { usePreventContextMenu } from "./lib";
import { AppErrorBoundary } from "./root-error-boundary";
import { router } from "./routes";

export const App = () => {
	usePreventContextMenu();

	useHotkeys(
		[
			{ hotkey: "F11", callback: toggleFullscreen },
			{ hotkey: "F", callback: toggleFullscreen },
			{ hotkey: "Escape", callback: exitFullscreen }
		],
		{ ignoreInputs: false }
	);

	return (
		<AppErrorBoundary>
			<Toaster />
			<RouterProvider router={router} />
		</AppErrorBoundary>
	);
};
