import type { ComponentProps, PropsWithChildren } from "preact/compat";

type Props = PropsWithChildren<ComponentProps<"input">>;

export const Checkbox = ({ children, ...props }: Props) => (
	<label class="flex items-center gap-2">
		<input type="checkbox" {...props} />
		<span class="text-accent font-medium">{children}</span>
	</label>
);
