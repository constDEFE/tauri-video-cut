import { Navigate } from "react-router";

import { useVideoStore } from "@/entities/video";
import { ImportArea } from "@/widgets/import-area";

export const ImportPage = () => {
	const isLoaded = useVideoStore((s) => s.state.isLoaded);

	if (isLoaded) {
		return <Navigate replace to="/editor" />;
	}

	return <ImportArea />;
};
