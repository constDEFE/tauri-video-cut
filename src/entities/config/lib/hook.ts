import { useSyncExternalStore } from "preact/compat";

const subscribe = (cb: () => void) => {
	window.addEventListener("configUpdate", cb);

	return () => {
		window.removeEventListener("configUpdate", cb);
	};
};

const getSnap = () => window.__CONFIG__;

export const useAppConfig = () => {
	return useSyncExternalStore(subscribe, getSnap);
};
