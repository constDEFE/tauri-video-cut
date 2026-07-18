type Props = {
	defaultValue: string;
};

export const PrefixField = ({ defaultValue }: Props) => (
	<div class="surface rounded-lg p-4">
		<h2 class="text-accent mb-3 text-lg font-semibold select-none">File Prefix</h2>
		<input
			type="text"
			defaultValue={defaultValue ?? ""}
			name="prefix"
			placeholder="output"
			class="input w-full rounded px-3 py-2"
			autoComplete="off"
			autoCorrect="off"
		/>
		<p class="text-text mt-2 text-xs">Files will be named: PREFIX-1, PREFIX-2, etc.</p>
	</div>
);
