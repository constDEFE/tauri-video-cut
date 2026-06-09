import { isRouteErrorResponse, useRouteError } from "react-router";

const Inner = () => {
	const error = useRouteError();

	if (isRouteErrorResponse(error)) {
		return (
			<div>
				<p>
					{error.status} {error.statusText}
				</p>
				<pre>{error.data}</pre>
			</div>
		);
	}

	if (error instanceof Error) {
		return (
			<div>
				<p class="text-accent">Message:</p>
				<pre class="text-text">{error.message}</pre>
				<p class="text-accent mt-4">The stack trace is:</p>
				<pre class="text-text">{error.stack}</pre>
			</div>
		);
	}

	return (
		<pre class="bg-accent-inverted text-text-text mt-2 overflow-auto rounded p-3 text-xs">{error?.toString()}</pre>
	);
};

export const ErrorBoundary = () => (
	<div class="bg-accent-inverted flex min-h-screen items-center justify-center p-4">
		<div class="surface w-full max-w-xl rounded-lg p-6 text-center">
			<h2 class="text-accent mb-4 text-2xl font-bold">Something went wrong</h2>
			<p class="text-text mb-4">
				The application encountered an unexpected error. Please try reloading the application.
			</p>
			<details class="mb-4 overflow-auto text-left">
				<summary class="text-text hover:text-accent cursor-pointer duration-100 ease-out">Error details</summary>
				<Inner />
			</details>
			<button onClick={() => window.location.reload()} class="button rounded px-4 py-2 font-semibold">
				Reload Application
			</button>
		</div>
	</div>
);
