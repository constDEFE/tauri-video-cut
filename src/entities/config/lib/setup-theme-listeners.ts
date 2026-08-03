import { invoke } from "@tauri-apps/api/core";

const handler = (e: Event) => {
	const theme = (e as CustomEvent<string>).detail;
	invoke("set_app_config_var", { key: "theme", value: theme }).catch(console.error);
};

export const setupThemeListeners = () => {
	window.addEventListener("themeUpdate", handler);
	return () => window.removeEventListener("themeUpdate", handler);
};
