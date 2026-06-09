import { open } from "@tauri-apps/plugin-dialog";
import { useRef } from "preact/hooks";

type Props = {
	defaulValue: string | null;
};

export const OutputField = ({ defaulValue }: Props) => {
	const inputRef = useRef<HTMLInputElement>(null);

	const handleSelect = async () => {
		if (!inputRef.current) {
			return;
		}

		const selected = await open({
			directory: true,
			multiple: false,
			title: "Select output folder"
		});

		if (!selected) {
			return;
		}

		inputRef.current.value = selected;
	};

	return (
		<div class="surface rounded-lg p-4">
			<h2 class="text-accent mb-3 text-lg font-semibold select-none">Output Folder</h2>
			<div class="flex items-center gap-2">
				<input
					ref={inputRef}
					type="text"
					defaultValue={defaulValue ?? ""}
					name="output-folder"
					autoComplete="off"
					autoCorrect="off"
					readOnly
					placeholder="Select folder..."
					class="input w-full rounded px-3 py-2"
				/>
				<button type="button" onClick={handleSelect} class="button rounded px-4 py-2 font-medium">
					Browse
				</button>
			</div>
		</div>
	);
};
