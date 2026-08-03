import { useSyncExternalStore } from "preact/compat";

const subscribe = (cb: () => void) => {
	window.addEventListener("zoomUpdate", cb);

	return () => {
		window.removeEventListener("zoomUpdate", cb);
	};
};

const getSnap = () => window.__CONFIG__.zoomScale;

export const useAppZoom = () => {
	const zoomScale = useSyncExternalStore(subscribe, getSnap);

	return zoomScale;
};
