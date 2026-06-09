import js from "@eslint/js";
import { createTypeScriptImportResolver } from 'eslint-import-resolver-typescript'
import { defineConfig, globalIgnores } from "eslint/config";
import { importX } from 'eslint-plugin-import-x'
import perfectionist from 'eslint-plugin-perfectionist'
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import globals from "globals";
import tseslint from "typescript-eslint";

export default defineConfig([
	globalIgnores(["dist", "node_modules"]),
	{
		files: ["**/*.{ts,tsx}"],
		extends: [
			js.configs.recommended,
			tseslint.configs.recommended,
			importX.flatConfigs.recommended,
			importX.flatConfigs.typescript,
			reactHooks.configs.flat.recommended,
			reactRefresh.configs.vite,
		],
		languageOptions: {
			ecmaVersion: 2020,
			sourceType: 'module',
			globals: globals.browser,
			parser: tseslint.parser
		},
		plugins: {
			"@typescript-eslint": tseslint.plugin,
			"import-x": importX,
			perfectionist,
		},
		settings: {
			"import-x/parsers": {
				"@typescript-eslint/parser": [".ts", ".tsx"],
			},
			'import-x/extensions': ['.ts', '.tsx', '.js', '.jsx'],
			"import-x/resolver-next": [
				createTypeScriptImportResolver({
					project: "./tsconfig.json",
					alwaysTryTypes: true,
					bun: true
				})
			]
		},
		rules: {
			"@typescript-eslint/consistent-type-imports": "error",
			"import-x/no-named-as-default": "off",
			"import-x/no-named-as-default-member": "off",
			"import-x/first": "error",
			"import-x/newline-after-import": "error",
			"import-x/no-duplicates": "error",
			/* for whatever reason marks half of imports as unresolved */
			"import-x/no-unresolved": "off",
			"import-x/order": "off",
			"import-x/no-cycle": "error",
			"import-x/no-restricted-paths": [
				"error",
				{
					"zones": [
						{ "target": "./src/shared", "from": "./src/entities" },
						{ "target": "./src/entities", "from": "./src/features" },
						{ "target": "./src/features", "from": "./src/widgets" }
					]
				}
			],
			"perfectionist/sort-imports": [
				"error",
				{
					type: "alphabetical",
					order: "asc",
					groups: [
						"builtin",
						"external",
						"internal",
						["parent", "sibling", "index"],
						"type",
					],
				},
			]
		}
	},
]);
