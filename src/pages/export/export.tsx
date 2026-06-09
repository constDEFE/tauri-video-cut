import { Navigate } from "react-router";

import { useVideoStore } from "@/entities/video";
import { ExportForm } from "@/widgets/export/form";

export const ExportPage = () => {
	const isLoaded = useVideoStore((s) => s.state.isLoaded);

	if (!isLoaded) {
		return <Navigate replace to="/import" />;
	}

	return (
		<div class="bg-accent-inverted grid min-h-screen place-items-center p-8">
			<ExportForm />
		</div>
	);
};
