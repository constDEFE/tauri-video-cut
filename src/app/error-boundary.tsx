import { Component } from "preact";

import type { ComponentChildren } from "preact";

interface Props {
	children: ComponentChildren;
	fallback?: ComponentChildren;
}

interface State {
	hasError: boolean;
	error: Error | null;
}

export class AppErrorBoundary extends Component<Props, State> {
	constructor(props: Props) {
		super(props);
		this.state = { hasError: false, error: null };
	}

	static override getDerivedStateFromError(error: Error): State {
		return { hasError: true, error };
	}

	override componentDidCatch(error: Error, errorInfo: unknown) {
		console.error("ErrorBoundary caught an error:", error, errorInfo);
	}

	render() {
		if (!this.state.hasError) {
			return this.props.children;
		}

		if (this.props.fallback) {
			return this.props.fallback;
		}

		return (
			<div class="bg-accent-inverted flex min-h-screen items-center justify-center p-4">
				<div class="surface w-full max-w-md rounded-lg p-6 text-center">
					<h2 class="text-accent mb-4 text-2xl font-bold">Something went wrong</h2>
					<p class="text-text mb-4">
						The application encountered an unexpected error. Please try reloading the application.
					</p>
					{this.state.error && (
						<details class="mb-4 text-left">
							<summary class="text-text hover:text-accent cursor-pointer duration-100 ease-out">Error details</summary>
							<pre class="bg-accent-inverted text-text mt-2 overflow-auto rounded p-3 text-xs">
								{this.state.error.toString()}
							</pre>
						</details>
					)}
					<button onClick={() => window.location.reload()} class="button rounded px-4 py-2 font-semibold">
						Reload Application
					</button>
				</div>
			</div>
		);
	}
}
