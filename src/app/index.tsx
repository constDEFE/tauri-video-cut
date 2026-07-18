import { getCurrentWindow } from "@tauri-apps/api/window";
import { render } from "preact";

import { persistTheme } from "@/entities/config";
import { useSessionStore } from "@/entities/session";

import { App } from "./app";

const initTheme = () => {
	document.documentElement.classList.add(window.__CONFIG__.theme);
};

const init = async () => {
	try {
		initTheme();
		persistTheme();
		useSessionStore.getState().actions.init(window.__SESSION__);
	} catch (error) {
		console.error("Failed to initialize app: ", error);
	}

	render(<App />, document.getElementById("root")!);

	await getCurrentWindow().show();
};

export { init };
