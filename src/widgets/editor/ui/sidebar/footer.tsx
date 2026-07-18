import { ExportButton, ImportButton } from "@/features/editor/export-controls";
import { AddSegmentButton, RemoveSegmentButton, SplitSegmentButton } from "@/features/editor/segment-edit";
import { SheetFooter } from "@/shared/ui";

export const Footer = () => (
	<SheetFooter class="border-secondary flex flex-col gap-1.5 border-t px-2 pt-2 pb-2.5">
		<div class="flex gap-1 *:flex-1">
			<AddSegmentButton />
			<RemoveSegmentButton />
			<SplitSegmentButton />
		</div>
		<div class="grid grid-cols-2 gap-1">
			<ImportButton />
			<ExportButton />
		</div>
	</SheetFooter>
);
