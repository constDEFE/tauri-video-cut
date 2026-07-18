import { toast } from "sonner";
import { create } from "zustand";

import type { Segment } from "./types";

type State = {
	segments: Segment[];
	selectedSegment: Segment | null;
	maxSegments: number;
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
	actions: Actions;
};

export const useSegmentsStore = create<SegmentsStore>((set, get) => ({
	state: {
		segments: [],
		selectedSegment: null,
		maxSegments: 999
	},
	actions: {
		getById: (id) => {
			const store = get();
			const idx = store.state.segments.findIndex((s) => s.id === id);

			if (idx === -1) {
				throw new Error(`Segment not found: ${id}`);
			}

			return store.state.segments[idx] as Segment;
		},
		set: (segments) => {
			if (segments.length < 1) {
				throw new Error("Cannot set zero segments");
			}

			const selectedSegment = segments.at(-1)!;

			set((s) => ({
				state: {
					...s.state,
					segments,
					selectedSegment
				}
			}));
		},
		add: (segment) => {
			const store = get();

			if (store.state.segments.length >= store.state.maxSegments) {
				toast.error(`Max segments reached: ${store.state.maxSegments}`);
				return;
			}

			set({
				state: {
					...store.state,
					segments: [...store.state.segments, segment],
					selectedSegment: segment
				}
			});
		},
		remove: (id) => {
			const store = get();

			if (store.state.segments.length <= 1) {
				toast.error("Cannot remove the only segment");
				return;
			}

			const segments = store.state.segments.filter((s) => s.id !== id);
			const selectedSegment = id === store.state.selectedSegment?.id ? segments.at(-1)! : store.state.selectedSegment;

			set({
				state: {
					...store.state,
					segments,
					selectedSegment
				}
			});
		},
		update: (id, values) => {
			const store = get();
			const idx = store.state.segments.findIndex((s) => s.id === id);

			if (idx === -1) {
				throw new Error(`Segment not found: ${id}`);
			}

			const current = store.state.segments[idx] as Segment;
			const updated = { ...current, ...values } as Segment;
			const segments = store.state.segments.with(idx, updated);
			const selectedSegment = store.state.selectedSegment?.id === id ? updated : store.state.selectedSegment;

			set({
				state: {
					...store.state,
					segments,
					selectedSegment
				}
			});
		},
		select: (id) => {
			const store = get();
			const selectedSegment = store.state.segments.find((s) => s.id === id) ?? null;

			set((s) => ({
				state: {
					...s.state,
					selectedSegment
				}
			}));
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

			const segments = Array.from<Segment>({ length: store.state.segments.length + 1 });

			let writeIdx = 0;
			for (let i = 0; i < store.state.segments.length; i++) {
				const s = store.state.segments[i]!;

				if (s.id === id) {
					segments[writeIdx++] = segment1;
					segments[writeIdx++] = segment2;
				} else {
					segments[writeIdx++] = s;
				}
			}

			set({
				state: {
					...store.state,
					segments,
					selectedSegment: segment1
				}
			});
		}
	}
}));
