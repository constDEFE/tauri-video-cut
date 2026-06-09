import { toast } from "sonner";
import { create } from "zustand";

import type { Segment } from "./types";

type State = {
	segments: Segment[];
	selectedId: string;
	maxSegments: number;
};

type Private = {
	_idToIdxMap: Map<Segment["id"], number>;
};

type Actions = {
	getById: (id: string) => Segment;
	set: (segments: Segment[]) => void;
	remove: (id: string) => void;
	add: (segment: Segment) => void;
	update: (id: string, values: Partial<Segment>) => void;
	select: (id: string) => void;
	split: (id: string, splitTime: number) => void;
};

export type SegmentsStore = {
	state: State;
	private: Private;
	actions: Actions;
};

export const useSegmentsStore = create<SegmentsStore>((set, get) => ({
	state: {
		segments: [],
		selectedId: "",
		maxSegments: 999
	},
	private: {
		_idToIdxMap: new Map()
	},
	actions: {
		getById: (id) => {
			const store = get();
			const idx = store.private._idToIdxMap.get(id);

			if (typeof idx === "undefined" || !store.state.segments[idx]) {
				throw "Corrupted { _idToIdxMap }";
			}

			return store.state.segments[idx];
		},
		set: (segments) => {
			const store = get();

			store.private._idToIdxMap.clear();
			segments.forEach((s, idx) => store.private._idToIdxMap.set(s.id, idx));

			set({ state: { ...store.state, segments, selectedId: segments[0]?.id ?? "" } });
		},
		add: (segment) => {
			const store = get();

			if (store.state.segments.length >= store.state.maxSegments) {
				toast.error(`Max segments reached: ${store.state.maxSegments}`);
				return;
			}

			const segments = [...store.state.segments, segment];

			store.private._idToIdxMap.set(segment.id, segments.length - 1);

			set({ state: { ...store.state, segments, selectedId: segment.id } });
		},
		remove: (id) => {
			const store = get();

			if (store.state.segments.length <= 1) {
				toast.error("Cannot remove the only segment");
				return;
			}

			store.private._idToIdxMap.clear();

			let idx = 0;
			const segments = store.state.segments.reduce<Segment[]>(
				(acc, s) => {
					if (s.id === id) {
						return acc;
					}

					store.private._idToIdxMap.set(s.id, idx);
					acc[idx] = s;
					idx++;

					return acc;
				},
				new Array(store.state.segments.length - 1)
			);

			const selectedId = store.state.selectedId === id ? (segments.at(-1)?.id ?? "") : store.state.selectedId;

			set({ state: { ...store.state, segments, selectedId } });
		},
		update: (id, values) => {
			const store = get();
			const idx = store.private._idToIdxMap.get(id);

			if (typeof idx === "undefined") {
				throw "Corrupted { _idToIdxMap }";
			}

			const updated = { ...store.state.segments[idx], ...values } as Segment;
			const segments = store.state.segments.with(idx, updated);

			set({ state: { ...store.state, segments } });
		},
		select: (id) => {
			set((s) => ({ state: { ...s.state, selectedId: id } }));
		},
		split: (id, splitTime) => {
			const store = get();
			const segment = store.actions.getById(id);

			if (!segment || splitTime <= segment.start || splitTime >= segment.end) {
				toast.error("Cannot split the segment");
				return;
			}

			const segment1: Segment = {
				id: `${id}-1`,
				start: segment.start,
				end: splitTime,
				duration: splitTime - segment.start,
				estimatedSize: segment.estimatedSize * ((splitTime - segment.start) / segment.duration)
			};

			const segment2: Segment = {
				id: `${id}-2`,
				start: splitTime,
				end: segment.end,
				duration: segment.end - splitTime,
				estimatedSize: segment.estimatedSize * ((segment.end - splitTime) / segment.duration)
			};

			store.private._idToIdxMap.clear();

			let idx = 0;
			const segments = store.state.segments.reduce<Segment[]>(
				(acc, s) => {
					if (s.id === id) {
						store.private._idToIdxMap.set(segment1.id, idx);
						store.private._idToIdxMap.set(segment2.id, idx + 1);
						acc[idx] = segment1;
						acc[idx + 1] = segment2;

						idx += 2;

						return acc;
					}

					store.private._idToIdxMap.set(s.id, idx);
					acc[idx] = s;
					idx++;

					return acc;
				},
				new Array(store.state.segments.length + 1)
			);

			set({ state: { ...store.state, segments, selectedId: segment1.id } });
		}
	}
}));
