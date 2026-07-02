import { defineConfig } from "oxlint";

export default defineConfig({
	plugins: ["react", "react-perf", "import", "promise", "eslint", "typescript", "unicorn", "oxc"],
	env: {
		builtin: true
	},
	ignorePatterns: ["dist", "node_modules"],
	overrides: [
		{
			files: ["**/*.{ts,tsx}"],
			rules: {
				"typescript/consistent-type-imports": "error",
				"import/no-named-as-default": "off",
				"import/no-named-as-default-member": "off",
				"import/first": "error",
				"import/newline-after-import": "error",
				"import/no-duplicates": "error",
				"import/no-cycle": "error"
			}
		}
	]
});
