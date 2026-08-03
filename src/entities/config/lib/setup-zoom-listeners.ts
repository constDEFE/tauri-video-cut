import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";

import { zoomIn, zoomOut } from "@/shared/lib/zoom";

import type { AppConfig } from "../model";

const updateHandler = async (e: Event) => {
	const zoomScale = (e as CustomEvent<AppConfig["zoomScale"]>).detail;

	await getCurrentWebview().setZoom(zoomScale).catch(console.error);
	invoke("set_app_config_var", { key: "zoomScale", value: zoomScale }).catch(console.error);
};

const keyboardListener = (e: KeyboardEvent) => {
	const isIn = e.ctrlKey && (e.key === "+" || e.key === "=");
	const isOut = e.ctrlKey && e.key === "-";

	if (isIn) {
		e.preventDefault();
		zoomIn();
	} else if (isOut) {
		e.preventDefault();
		zoomOut();
	}
};

export const setupZoomListeners = () => {
	window.addEventListener("keyup", keyboardListener);
	window.addEventListener("zoomUpdate", updateHandler);
	return () => window.removeEventListener("zoomUpdate", updateHandler);
};
