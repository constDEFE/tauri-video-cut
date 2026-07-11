import { useState } from "preact/hooks";

import { useSessionStore } from "@/entities/session";

import { SS_SESSION_DECLINED_KEY, SS_SESSION_RESTORED_KEY } from "../../lib";

export const useModal = (loader: (path: string, fromSession?: boolean) => Promise<void>) => {
	const [isOpen, setIsOpen] = useState(() => {
		return (
			!!window.__SESSION__.file_path &&
			sessionStorage.getItem(SS_SESSION_DECLINED_KEY) !== "true" &&
			sessionStorage.getItem(SS_SESSION_RESTORED_KEY) !== "true"
		);
	});

	const session = useSessionStore((s) => s.state.session);

	const onDecline = () => {
		setIsOpen(false);
		sessionStorage.setItem(SS_SESSION_DECLINED_KEY, "true");
	};

	const onRestore = async () => {
		if (!session.file_path) {
			return;
		}

		await loader(session.file_path, true);
		sessionStorage.setItem(SS_SESSION_RESTORED_KEY, "true");
	};

	return {
		isOpen,
		onDecline,
		onRestore
	};
};
