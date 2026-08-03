import { Toaster as CoreToaster } from "sonner";

import { useAppTheme } from "@/shared/lib/theme";

export const Toaster = () => {
	const { theme } = useAppTheme();

	return <CoreToaster expand closeButton theme={theme} />;
};
