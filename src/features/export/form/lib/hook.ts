import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router";
import { toast } from "sonner";

import { useSegmentsStore } from "@/entities/segments";
import { useVideoStore } from "@/entities/video";
import { useTheme } from "@/shared/lib/theme";
import { invariant } from "@/shared/utils";

import { parseExportFields } from "./utils";

import type { EventFor } from "@/shared/types/react";

export const useExportForm = () => {
	const audioTracks = useVideoStore((s) => s.state.metadata?.audio_tracks);
	const segments = useSegmentsStore((s) => s.state.segments);
	const { theme } = useTheme();

	const navigate = useNavigate();

	const handleSubmit = (e: EventFor<"form", "submit">) => {
		e.preventDefault();

		invariant(audioTracks);

		const form = e.currentTarget;
		const parsedSettings = parseExportFields(form, audioTracks, segments);

		if (typeof parsedSettings === "string") {
			toast.error(parsedSettings);
			return;
		}

		const newConfig = {
			outputFolder: parsedSettings.output,
			prefix: parsedSettings.prefix,

			theme
		};

		window.__CONFIG__ = newConfig;
		window.dispatchEvent(new CustomEvent("configUpdate", { detail: newConfig }));

		invoke("set_app_config", { config: newConfig });
		navigate("/processing", { replace: true, state: { settings: parsedSettings } });
	};

	return handleSubmit;
};
