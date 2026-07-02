import { defineConfig } from "oxfmt";

export default defineConfig({
	useTabs: true,
	tabWidth: 2,
	singleQuote: false,
	arrowParens: "always",
	trailingComma: "none",
	semi: true,
	endOfLine: "lf",
	printWidth: 120,
	sortTailwindcss: true,
	sortImports: {
		type: "alphabetical",
		order: "asc",
		groups: ["side_effect", "builtin", "external", "internal", ["parent", "sibling", "index"], "type"],
		newlinesBetween: true
	}
});
