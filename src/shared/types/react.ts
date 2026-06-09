import type { JSX } from "preact";

type GetHandlers<T extends keyof JSX.IntrinsicElements, A = keyof JSX.IntrinsicElements[T]> = Extract<A, `on${string}`>;

export type EventsFor<
	E extends keyof JSX.IntrinsicElements,
	A = keyof JSX.IntrinsicElements[E],
	H = Extract<A, `on${string}`>
> = H extends `on${infer Event}` ? Uncapitalize<Event> : never;

export type EventFor<
	El extends keyof JSX.IntrinsicElements,
	Ev extends EventsFor<El>
> = JSX.IntrinsicElements[El][`on${Capitalize<Ev>}` extends GetHandlers<El> ? `on${Capitalize<Ev>}` : never] extends
	| ((e: infer HandlerEvent) => unknown)
	| undefined
	? HandlerEvent
	: never;

export type EventHandlerFor<
	El extends keyof JSX.IntrinsicElements,
	Ev extends EventsFor<El>
> = JSX.IntrinsicElements[El][`on${Capitalize<Ev>}` extends GetHandlers<El> ? `on${Capitalize<Ev>}` : never];
