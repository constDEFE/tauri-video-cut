import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

import type { Session } from "./types";

const FLUSH_INTERVAL_MS = 10_000;

type State = {
	session: Session;
	isDirty: boolean;
	saveTimer: ReturnType<typeof setTimeout> | null;
};

type Private = {
	markDirty: () => void;
	flushToDisk: () => Promise<void>;
};

type Actions = {
	init: (session: Session) => void;
	updateSession: (partial: Partial<Session>, markDirty?: boolean) => void;
	blank: () => void;
};

export type SessionStore = {
	state: State;
	actions: Actions;
	private: Private;
};

const INITIAL_STATE: State = {
	session: { file_path: null, segments: null, audio_tracks: null },
	isDirty: false,
	saveTimer: null
};

export const useSessionStore = create<SessionStore>((set, get) => ({
	state: INITIAL_STATE,
	private: {
		markDirty: () => {
			const store = get();

			if (store.state.isDirty || store.state.saveTimer !== null) {
				return;
			}

			const timer = setTimeout(() => {
				get().private.flushToDisk();
			}, FLUSH_INTERVAL_MS);

			set({ state: { ...store.state, isDirty: true, saveTimer: timer } });
		},
		flushToDisk: async () => {
			const store = get();
			const timer = store.state.saveTimer;

			if (timer !== null) {
				clearTimeout(timer);
			}

			if (!store.state.isDirty) {
				return;
			}

			set({ state: { ...store.state, isDirty: false, saveTimer: null } });

			try {
				const mappedSegments =
					store.state.session.segments?.map((s) => ({ end: s.end, start: s.start, id: s.id })) ?? null;

				await invoke("set_session", { session: { ...store.state.session, segments: mappedSegments } });
			} catch (err) {
				console.error("Failed to flush session:", err);
				set({ state: { ...store.state, isDirty: true } });
			}
		}
	},
	actions: {
		init: (session) => {
			set({ state: { ...INITIAL_STATE, session } });
		},
		updateSession: (partial, markDirty = true) => {
			const store = get();

			set((s) => ({ state: { ...s.state, session: { ...s.state.session, ...partial } } }));

			if (markDirty) {
				store.private.markDirty();
			}
		},
		blank: () => {
			set({ state: { ...INITIAL_STATE } });
		}
	}
}));
