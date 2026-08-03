import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { render } from "preact";

import { setupThemeListeners, setupZoomListeners } from "@/entities/config";
import { useSessionStore } from "@/entities/session";

import { App } from "./app";

const initTheme = () => {
	document.documentElement.classList.add(window.__CONFIG__.theme);
};

const initZoom = async () => {
	try {
		await getCurrentWebview().setZoom(window.__CONFIG__.zoomScale);
	} catch (err) {
		await invoke("set_app_config_var", { key: "zoomScale", value: 1 });

		throw Error("Error initializing zoom", { cause: err });
	}
};

const init = async () => {
	try {
		initTheme();
		initZoom();
		useSessionStore.getState().actions.init(window.__SESSION__);

		setupThemeListeners();
		setupZoomListeners();
	} catch (error) {
		console.error("Failed to initialize app: ", error);
	}

	render(<App />, document.getElementById("root")!);

	await getCurrentWindow().show();
};

export { init };
