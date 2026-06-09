import type { ComponentProps } from "preact";

type Props = ComponentProps<"svg">;

export const SidebarIcon = (props: Props) => (
	<svg
		stroke="currentColor"
		fill="none"
		stroke-width="2"
		viewBox="0 0 24 24"
		stroke-linecap="round"
		stroke-linejoin="round"
		{...props}
	>
		<path d="M4 4m0 2a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2z" />
		<path d="M15 4v16" />
		<path d="M10 10l-2 2l2 2" />
	</svg>
);
