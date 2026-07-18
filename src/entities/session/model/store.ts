import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

import type { Session } from "./types";

const FLUSH_INTERVAL_MS = 10_000;

type State = {
	session: Session;
	isDirty: boolean;
	isFlushing: boolean;
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
	isFlushing: false,
	saveTimer: null
};

export const useSessionStore = create<SessionStore>((set, get) => ({
	state: INITIAL_STATE,
	private: {
		markDirty: () => {
			const { isDirty, saveTimer } = get().state;

			if (isDirty || saveTimer !== null) {
				return;
			}

			const timer = setTimeout(() => get().private.flushToDisk(), FLUSH_INTERVAL_MS);

			set((s) => ({ state: { ...s.state, isDirty: true, saveTimer: timer } }));
		},
		flushToDisk: async () => {
			const store = get();

			if (store.state.saveTimer !== null) {
				clearTimeout(store.state.saveTimer);
			}

			if (!store.state.isDirty || store.state.isFlushing) {
				return;
			}

			set((s) => ({ state: { ...s.state, isFlushing: true, saveTimer: null } }));

			try {
				const snapshot = get().state.session;
				const mappedSegments = snapshot.segments?.map(({ id, start, end }) => ({ id, start, end })) ?? null;

				await invoke("set_session", { session: { ...snapshot, segments: mappedSegments } });

				set((s) => ({ state: { ...s.state, isDirty: false, isFlushing: false } }));
			} catch (err) {
				console.error("Failed to flush session:", err);
				set((s) => ({ state: { ...s.state, isDirty: true, isFlushing: false } }));
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
