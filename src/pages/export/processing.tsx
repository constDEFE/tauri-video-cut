import { Navigate, useLocation } from "react-router";

import { ExportProgress } from "@/widgets/export/progress";

import type { ExportSettings } from "@/features/export/execution";

export const ProcessingPage = () => {
	const location = useLocation();
	const settings = location.state?.settings as ExportSettings;

	if (!settings) {
		return <Navigate replace to="/export" />;
	}

	return (
		<div class="bg-accent-inverted flex min-h-screen flex-col items-center justify-center p-8">
			<ExportProgress settings={settings} />
		</div>
	);
};
