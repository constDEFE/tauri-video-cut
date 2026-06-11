import { getCurrentWindow } from "@tauri-apps/api/window";
import { render } from "preact";

import { App } from "./app";

const initTheme = () => {
	document.documentElement.classList.add(window.__CONFIG__.theme);
};

const init = async () => {
	try {
		initTheme();
	} catch (error) {
		console.error("Failed to initialize app: ", error);
	}

	render(<App />, document.getElementById("root")!);

	await getCurrentWindow().show();
};

export { init };
