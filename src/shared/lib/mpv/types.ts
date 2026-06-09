export type MpvPropertyFormat = "flag" | "int64" | "double" | "string" | "node";

export type MpvObservableProperty =
	| readonly [string, "flag"]
	| readonly [string, "int64"]
	| readonly [string, "double"]
	| readonly [string, "string"]
	| readonly [string, "node"]
	| readonly [string, "flag", "none"]
	| readonly [string, "int64", "none"]
	| readonly [string, "double", "none"]
	| readonly [string, "string", "none"]
	| readonly [string, "node", "none"];

export interface MpvConfig {
	initialOptions: Record<string, string>;
	observedProperties: readonly MpvObservableProperty[];
}

export interface MpvPropertyEvent {
	name: string;
	data: boolean | number | string | null | unknown;
}

export type MpvPropertyCallback = (event: MpvPropertyEvent) => void;
