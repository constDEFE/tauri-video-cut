const isProduction = import.meta.env["NODE_ENV"] === "production";
const prefix = "Invariant failed";

export function invariant<T>(condition: T, message?: string | (() => string)): asserts condition is NonNullable<T> {
	if (condition) {
		return;
	}

	if (isProduction) {
		throw new Error(prefix);
	}

	const provided = typeof message === "function" ? message() : message;
	const value = provided ? `${prefix}: ${provided}` : prefix;

	throw new Error(value);
}
