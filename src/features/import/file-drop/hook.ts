import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "preact/hooks";
import { toast } from "sonner";

export const useFileDrop = (loadCb: (path: string) => Promise<void>, formats: string[]) => {
	useEffect(() => {
		const appWindow = getCurrentWindow();

		const sub = async () => {
			const unsub = await appWindow.onDragDropEvent((event) => {
				if (event.payload.type !== "drop" || event.payload.paths?.length < 1) {
					return;
				}

				const filePath = event.payload.paths[0]!;
				const fileName = filePath?.toLowerCase() ?? "file";
				const hasValidExtension = formats.some((ext) => fileName.endsWith(`.${ext}`));

				if (!hasValidExtension) {
					toast.error("Invalid file type. Please drop a video file");
					return;
				}

				loadCb(filePath);
			});

			return unsub;
		};

		let unsub: () => void;
		sub().then((fn) => (unsub = fn));

		return () => void unsub?.();
	}, [loadCb, formats]);
};
