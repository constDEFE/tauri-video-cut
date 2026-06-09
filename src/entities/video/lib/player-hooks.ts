import { useEffect } from "preact/hooks";
import { listenEvents } from "tauri-plugin-libmpv-api";

import { useVideoStore } from "@/entities/video";
import { observeMpvProperties } from "@/shared/lib/mpv";

import type { MpvPropertyEvent } from "@/shared/lib/mpv";
import type { MpvEvent } from "tauri-plugin-libmpv-api";

type EventsSub = (e: MpvEvent) => void | Promise<void>;

export const usePlayerEvents = (sub: EventsSub) => {
	useEffect(() => {
		let unsub: () => void;
		listenEvents(sub).then((fn) => (unsub = fn));

		return () => void unsub?.();
	}, [sub]);
};

type PropsSub = (e: MpvPropertyEvent) => void | Promise<void>;

export const usePlayerProps = (sub: PropsSub) => {
	const isInitialized = useVideoStore((s) => s.state.player.isInitialized);

	useEffect(() => {
		if (!isInitialized) return;

		let unsub: () => void;
		observeMpvProperties(sub).then((fn) => (unsub = fn));

		return () => void unsub?.();
	}, [isInitialized, sub]);
};
