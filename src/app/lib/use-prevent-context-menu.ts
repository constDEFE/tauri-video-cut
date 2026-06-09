import { useEffect } from "preact/hooks";

const sub = (e: Event) => {
	e.preventDefault();
};

export const usePreventContextMenu = () => {
	useEffect(() => {
		window.addEventListener("contextmenu", sub);

		return () => {
			window.removeEventListener("contextmenu", sub);
		};
	}, []);
};
