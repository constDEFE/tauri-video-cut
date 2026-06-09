/* eslint-disable @typescript-eslint/no-explicit-any */
type Fn<Args extends any[]> = {
	(...args: Args): void;
};

type DebouncedFunction<Args extends any[]> = {
	(...args: Args): void;
	cancel: () => void;
};

export const debounce = <Args extends any[]>(fn: Fn<Args>, ms: number): DebouncedFunction<Args> => {
	let timeoutId: ReturnType<typeof setTimeout>;

	const debounced = (...args: Parameters<typeof fn>) => {
		clearTimeout(timeoutId);

		timeoutId = setTimeout(() => {
			fn(...args);
		}, ms);
	};

	debounced.cancel = () => {
		clearTimeout(timeoutId);
	};

	return debounced;
};
