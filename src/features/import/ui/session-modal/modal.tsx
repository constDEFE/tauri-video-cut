import { useSessionStore } from "@/entities/session";

import { useModal } from "./hook";

type Props = {
	isLoading?: boolean;
	loader: (path: string, fromSession?: boolean) => Promise<void>;
};

export const SessionModal = ({ isLoading, loader }: Props) => {
	const { isOpen, onDecline, onRestore } = useModal(loader);
	const session = useSessionStore((s) => s.state.session);

	if (!isOpen || isLoading) {
		return;
	}

	return (
		<div class="bg-accent-inverted fixed inset-0 grid place-items-center">
			<div class="surface w-full max-w-md rounded-lg border p-6 shadow-xl">
				<h2 class="text-accent mb-2 text-xl font-semibold">Restore Previous Session?</h2>
				<p class="text-text">
					Do you want to restore the previous session?
					<br />
				</p>
				<p class="text-text underline">{session.file_path}</p>
				<div class="mt-4 flex justify-end gap-3">
					<button onClick={onRestore} class="button rounded-md px-6 py-2 font-medium">
						Yes
					</button>
					<button onClick={onDecline} class="button secondary rounded-md px-6 py-2 font-medium">
						No
					</button>
				</div>
			</div>
		</div>
	);
};
