import { Toaster as CoreToaster } from "sonner";

import { useTheme } from "@/shared/lib/theme";

export const Toaster = () => {
	const { theme } = useTheme();

	return <CoreToaster expand closeButton theme={theme} />;
};
