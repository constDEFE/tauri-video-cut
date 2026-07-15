import { ImportPage } from "@/pages";
import { useRoute } from "@/shared/lib/router";

import { routes } from "./routes";

export const RouteOutlet = () => {
	const { path } = useRoute();
	const Page = routes[path] ?? ImportPage;

	return <Page />;
};
