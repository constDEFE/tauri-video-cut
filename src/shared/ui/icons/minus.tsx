import type { ComponentProps } from "preact";

type Props = ComponentProps<"svg">;

export const MinusIcon = (props: Props) => (
	<svg
		stroke="currentColor"
		fill="none"
		stroke-width="2"
		viewBox="0 0 24 24"
		stroke-linecap="round"
		stroke-linejoin="round"
		{...props}
	>
		<path d="M5 12h14" />
	</svg>
);
