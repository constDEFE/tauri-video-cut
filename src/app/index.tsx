import { getCurrentWindow } from "@tauri-apps/api/window";
import { render } from "preact";

import { App } from "./app";

const initTheme = () => {
	const config = window.__CONFIG__;

	if (config.theme === "dark") {
		document.documentElement.classList.add("dark");
	} else {
		document.documentElement.classList.add("light");
	}
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
