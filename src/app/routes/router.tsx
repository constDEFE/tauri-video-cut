import { createMemoryRouter, Navigate, Outlet } from "react-router";

import { CompletePage, EditorPage, ExportPage, ImportPage, ProcessingPage } from "@/pages";

import { ErrorBoundary } from "./error-boundary";

export const router = createMemoryRouter([
	{
		path: "/",
		ErrorBoundary,
		element: <Outlet />,
		children: [
			{
				index: true,
				element: <ImportPage />
			},
			{
				path: "editor",
				element: <EditorPage />
			},
			{
				path: "export",
				element: <ExportPage />
			},
			{
				path: "processing",
				element: <ProcessingPage />
			},
			{
				path: "complete",
				element: <CompletePage />
			}
		]
	},
	{
		path: "*",
		element: <Navigate replace to="/" />
	}
]);
