import preact from "@preact/preset-vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
	plugins: [preact(), tailwindcss()],
	resolve: {
		tsconfigPaths: true,
		alias: {
			react: "preact/compat",
			"react-dom": "preact/compat"
		}
	},
	clearScreen: false,
	build: {
		minify: "terser",
		terserOptions: {
			compress: {
				drop_console: true,
				drop_debugger: true,
				pure_funcs: ["console.log", "console.info", "console.debug"]
			}
		},
		rolldownOptions: {
			output: {
				manualChunks: undefined
			}
		},
		target: "esnext",
		cssMinify: "lightningcss"
	},
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host
			? {
					protocol: "ws",
					host,
					port: 1421
				}
			: undefined,
		watch: {
			ignored: ["**/src-tauri/**"]
		}
	}
});
