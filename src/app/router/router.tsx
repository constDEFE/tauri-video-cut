import { RouterProvider } from "@/shared/lib/router";

import { RouteOutlet } from "./outlet";

export const AppRouter = () => (
	<RouterProvider>
		<RouteOutlet />
	</RouterProvider>
);
