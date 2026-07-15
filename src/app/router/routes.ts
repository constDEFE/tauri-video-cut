import { CompletePage, EditorPage, ExportPage, ImportPage, ProcessingPage } from "@/pages";

import type { ComponentType } from "preact";

export const routes: Record<string, ComponentType> = {
	"/": ImportPage,
	"/editor": EditorPage,
	"/export": ExportPage,
	"/processing": ProcessingPage,
	"/complete": CompletePage
};
