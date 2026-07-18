import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "preact/hooks";
import { toast } from "sonner";

export const useFileDrop = (loadCb: (path: string) => Promise<void>, formats: string[], isLoading: boolean) => {
	useEffect(() => {
		const appWindow = getCurrentWindow();
		let unsub: (() => void) | undefined;
		let cancelled = false;

		appWindow
			.onDragDropEvent((event) => {
				if (isLoading || event.payload.type !== "drop" || event.payload.paths?.length < 1) {
					return;
				}

				const filePath = event.payload.paths[0]!;
				const fileName = filePath?.toLowerCase() ?? "file";

				if (!formats.some((ext) => fileName.endsWith(`.${ext}`))) {
					toast.error("Invalid file type. Please drop a video file");
					return;
				}

				loadCb(filePath);
			})
			.then((fn) => {
				if (cancelled) fn();
				else unsub = fn;
			});

		return () => {
			cancelled = true;
			unsub?.();
		};
	}, [loadCb, formats, isLoading]);
};
