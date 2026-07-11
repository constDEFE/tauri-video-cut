import type { AppConfig } from "./types";

declare global {
	interface Window {
		__CONFIG__: AppConfig;
	}

	interface WindowEventMap {
		configUpdate: CustomEvent<AppConfig>;
		themeUpdate: CustomEvent<AppConfig["theme"]>;
	}
}
