type Props = {
	isLoading?: boolean;
	children: string;
};

export const StatusView = ({ children, isLoading }: Props) => (
	<div class="bg-accent-inverted absolute inset-0 grid place-items-center">
		<div class="flex flex-col items-center gap-3">
			{isLoading && <div class="border-accent size-8 animate-spin rounded-full border-2 border-t-transparent" />}
			<span class="text-text text-sm select-none">{children}</span>
		</div>
	</div>
);
