import { useAppConfig } from "@/entities/config";
import { useVideoStore } from "@/entities/video";
import { useNavigate } from "@/shared/lib/router";
import { getFileNameWithoutExtension } from "@/shared/utils";

import { useExportForm } from "../lib";
import { AudioTracks } from "./audio-tracks";
import { OutputField } from "./output-field";
import { PrefixField } from "./prefix-field";
import { SmartCutField } from "./smart-cut-field";

export const ExportForm = () => {
	const filePath = useVideoStore((s) => s.state.filePath);
	const config = useAppConfig();
	const navigate = useNavigate();
	const submitHandler = useExportForm();

	const handleBack = () => {
		navigate("/editor");
	};

	const defaultPrefix = getFileNameWithoutExtension(filePath);

	return (
		<div class="mx-auto w-full max-w-2xl">
			<h1 class="text-accent mb-8 text-4xl font-bold select-none">Export Settings</h1>
			<form onSubmit={submitHandler} class="flex flex-col gap-6">
				<OutputField defaultValue={config.outputFolder} />
				<PrefixField defaultValue={defaultPrefix} />
				<SmartCutField defaultChecked={true} />
				<AudioTracks defaultValue="global" />
				<div class="grid grid-cols-2 gap-2 self-center">
					<button type="submit" class="button rounded-md px-6 py-2 font-medium">
						Start Export
					</button>
					<button type="button" onClick={handleBack} class="button secondary rounded-md px-6 py-2 font-medium">
						Go Back
					</button>
				</div>
			</form>
		</div>
	);
};
