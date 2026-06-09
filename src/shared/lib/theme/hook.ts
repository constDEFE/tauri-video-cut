import { invoke } from "@tauri-apps/api/core";
import { useCallback, useSyncExternalStore } from "preact/compat";

const subscribe = (cb: () => void) => {
	window.addEventListener("themeUpdate", cb);

	return () => {
		window.removeEventListener("themeUpdate", cb);
	};
};

const getSnap = () => window.__CONFIG__.theme;

export const useTheme = () => {
	const theme = useSyncExternalStore(subscribe, getSnap);

	const toggleTheme = useCallback(() => {
		const isDark = window.__CONFIG__.theme === "dark";

		if (isDark) {
			document.documentElement.classList.remove("dark");
			document.documentElement.classList.add("light");
		} else {
			document.documentElement.classList.remove("light");
			document.documentElement.classList.add("dark");
		}

		window.__CONFIG__.theme = isDark ? "light" : "dark";
		window.dispatchEvent(new CustomEvent("themeUpdate", { detail: window.__CONFIG__.theme }));

		invoke("set_app_config_var", { key: "theme", value: window.__CONFIG__.theme });
	}, []);

	return { theme, toggleTheme };
};
