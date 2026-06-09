import type { AppConfig } from "@/entities/config";

declare global {
	interface Window {
		__CONFIG__: AppConfig;
	}

	interface WindowEventMap {
		configUpdate: CustomEvent<AppConfig>;
		themeUpdate: CustomEvent<AppConfig["theme"]>;
	}
}
