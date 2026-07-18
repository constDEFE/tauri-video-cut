import type { ExportSettings } from "@/features/export/execution";
import type { EventFor } from "@/shared/types/react";

type Props = {
	value: ExportSettings["audioMode"];
	onChange: (value: Props["value"]) => void;
};

export const ModeSelect = ({ value, onChange }: Props) => {
	const handleChange = (e: EventFor<"input", "change">) => {
		onChange(e.currentTarget.value as "global" | "per-segment");
	};

	return (
		<div class="mb-4 space-y-2">
			<label class="flex items-center gap-2">
				<input
					type="radio"
					name="audio-mode"
					onChange={handleChange}
					value="global"
					checked={value === "global"}
					class="size-4"
				/>
				<span class="text-accent">Global setting</span>
			</label>
			<label class="flex items-center gap-2">
				<input
					type="radio"
					name="audio-mode"
					onChange={handleChange}
					value="per-segment"
					checked={value === "per-segment"}
					class="size-4"
				/>
				<span class="text-accent">Individual per segment</span>
			</label>
		</div>
	);
};
