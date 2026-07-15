import { createContext } from "preact";
import { useCallback, useContext, useState } from "preact/hooks";

import type { NavigateFn, Route } from "./types";
import type { PropsWithChildren } from "preact/compat";

const INITIAL_ROUTE: Route = { path: "/", state: null };

const RouteContext = createContext<Route>(INITIAL_ROUTE);
const NavigateContext = createContext<NavigateFn>(() => {});

type Props = {
	initialPath?: string;
};

export const RouterProvider = ({ children, initialPath = "/" }: PropsWithChildren<Props>) => {
	const [route, setRoute] = useState<Route>({ path: initialPath, state: null });

	const navigate = useCallback<NavigateFn>((path, options) => {
		setRoute({ path, state: options?.state ?? null });
	}, []);

	return (
		<NavigateContext.Provider value={navigate}>
			<RouteContext.Provider value={route}>{children}</RouteContext.Provider>
		</NavigateContext.Provider>
	);
};

export const useNavigate = () => useContext(NavigateContext);
export const useRoute = () => useContext(RouteContext);
