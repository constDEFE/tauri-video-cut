import type { Session } from "./types";

declare global {
	interface Window {
		__SESSION__: Session;
	}
}
