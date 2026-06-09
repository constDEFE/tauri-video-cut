import type { ComponentProps } from "preact";

type Props = ComponentProps<"svg">;

export const PauseIcon = (props: Props) => (
	<svg stroke="currentColor" fill="currentColor" stroke-width="0" viewBox="0 0 512 512" {...props}>
		<path d="M208 432h-48a16 16 0 0 1-16-16V96a16 16 0 0 1 16-16h48a16 16 0 0 1 16 16v320a16 16 0 0 1-16 16zm144 0h-48a16 16 0 0 1-16-16V96a16 16 0 0 1 16-16h48a16 16 0 0 1 16 16v320a16 16 0 0 1-16 16z"></path>
	</svg>
);
