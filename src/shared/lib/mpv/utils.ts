import { command, destroy, getProperty, init, observeProperties, setProperty } from "tauri-plugin-libmpv-api";

import type { MpvPropertyCallback } from "./types";
import type { MpvConfig, MpvObservableProperty } from "tauri-plugin-libmpv-api";

const WINDOWS_MPV_CONFIG: Record<string, string> = {
	vo: "gpu-next",
	"gpu-api": "auto",
	hwdec: "auto-safe",
	"hr-seek-framedrop": "no",
	"demuxer-max-back-bytes": "150MiB",
	"keep-open": "yes",
	"force-window": "yes",
	"input-default-bindings": "no",
	"input-vo-keyboard": "no",
	pause: "yes",
	autofit: "100%x100%",
	"autofit-larger": "100%x100%",
	"video-unscaled": "no",
};

const OBSERVED_PROPERTIES = [
	["pause", "flag"],
	["time-pos", "double", "none"],
	["duration", "double", "none"],
	["volume", "int64"],
	["filename", "string", "none"],
	["mute", "flag"],
	["aid", "int64", "none"],
	["eof-reached", "flag"]
] as const satisfies MpvObservableProperty[];

let mpvInitialized = false;
let endOfFile = false;

export const initializeMpv = async () => {
	if (mpvInitialized) {
		console.warn("MPV already initialized");
		return;
	}

	const config: MpvConfig = {
		initialOptions: WINDOWS_MPV_CONFIG,
		observedProperties: OBSERVED_PROPERTIES
	};

	try {
		await init(config);
		mpvInitialized = true;
	} catch (error) {
		console.error("MPV initialization failed:", error);
		throw error;
	}
};

export const observeMpvProperties = async (callback: MpvPropertyCallback) => {
	if (!mpvInitialized) {
		throw Error("MPV not initialized. Call initializeMpv() first.");
	}

	return await observeProperties(OBSERVED_PROPERTIES, callback);
};

export const executeCommand = async (commandToExecute: string, args?: (string | number | boolean)[]) => {
	if (!mpvInitialized) {
		throw Error("MPV not initialized");
	}

	try {
		await command(commandToExecute, args);
	} catch (error) {
		console.error(`MPV command failed: ${command}`, error);
		throw error;
	}
};

export const setMpvProperty = async <T extends string | number | boolean>(name: string, value: T) => {
	if (!mpvInitialized) {
		throw Error("MPV not initialized");
	}

	try {
		await setProperty(name, value);
	} catch (error) {
		console.error(`MPV setProperty failed: ${name}`, error);
		throw error;
	}
};

export const getMpvProperty = async <T>(name: string, format: "flag" | "int64" | "double" | "string"): Promise<T> => {
	if (!mpvInitialized) {
		throw Error("MPV not initialized");
	}

	try {
		return (await getProperty(name, format)) as T;
	} catch (error) {
		console.error(`MPV getProperty failed: ${name}`, error);
		throw error;
	}
};

export const loadFile = async (filePath: string) => {
	await executeCommand("loadfile", [filePath]);
};

export const play = async () => {
	if (endOfFile) {
		await seek(0);
	}

	await setMpvProperty("pause", false);
};

export const pause = async () => {
	await setMpvProperty("pause", true);
};

let prevAudioTrack = 0;
export const setAudioTrack = async (id: number | number) => {
	if (id === prevAudioTrack) return;
	prevAudioTrack = id;
	await setMpvProperty("aid", id.toString());
};

export const seek = async (position: number) => {
	await executeCommand("seek", [position, "absolute"]);
};

export const seekForward = async () => {
	await executeCommand("seek", ["5", "relative"]);
};

export const seekBackward = async () => {
	await executeCommand("seek", ["-5", "relative"]);
};

export const mute = async (state: boolean) => {
	await setMpvProperty("mute", state);
};

export const frameStep = async () => {
	await executeCommand("frame-step");
};

/**
 * @see https://github.com/mpv-player/mpv/issues/4019
 */
export const frameBackStep = async () => {
	await executeCommand("frame-back-step");
};

export const setVolume = async (volume: number) => {
	await setMpvProperty("volume", Math.max(0, Math.min(100, volume)));
};

export const destroyMpv = async () => {
	if (!mpvInitialized) {
		return;
	}

	try {
		await destroy();
		mpvInitialized = false;
	} catch (error) {
		console.error("MPV destroy failed:", error);
		throw error;
	}
};

export const setEndOfFile = (isEof: boolean) => {
	endOfFile = isEof;
};
