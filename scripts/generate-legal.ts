#!/usr/bin/env bun
import { mkdir, readFile, writeFile, access } from "node:fs/promises";
import { join } from "node:path";

import { $ } from "bun";

const OUT = "legal/generated";
await mkdir(OUT, { recursive: true });

const exists = async (p: string) => {
	try {
		await access(p);
		return true;
	} catch {
		return false;
	}
};

const has = (cmd: string) => Bun.which(cmd) !== null;

const clean = (t: string) => t.replace(/\r\n/g, "\n").trim();

async function tryRun(label: string, cmd: Promise<any>) {
	try {
		await cmd;
	} catch (e: any) {
		console.warn(`[WARN] ${label}: ${e?.message ?? e}`);
	}
}

async function readJson(path: string): Promise<any | null> {
	try {
		if (!(await exists(path))) return null;
		const raw = await readFile(path, "utf8");
		if (!raw.trim()) return null;
		return JSON.parse(raw);
	} catch (e: any) {
		console.warn(`[WARN] Could not parse ${path}: ${e?.message ?? e}`);
		return null;
	}
}

console.log("Frontend licenses...");
await tryRun(
	"license-checker",
	$`bunx --bun license-checker-rseidelsohn --production --json --files > ${OUT}/frontend-licenses.json`
);

console.log("[INFO] Skipping license-compliance (compound SPDX expressions). Review frontend-licenses.json.");

console.log("Rust licenses...");
const MANIFEST = "src-tauri/Cargo.toml";
if (has("cargo")) {
	await tryRun("cargo-license", $`cargo license --manifest-path ${MANIFEST} --json > ${OUT}/rust-licenses.json`);

	if ((await exists("legal/templates/about.hbs")) && (await exists("legal/about.toml"))) {
		await tryRun(
			"cargo-about",
			$`cargo about generate --manifest-path ${MANIFEST} --config legal/about.toml legal/templates/about.hbs > ${OUT}/rust-licenses.html`
		);
	}

	await tryRun("cargo-deny", $`cargo deny --manifest-path ${MANIFEST} check licenses`);
} else {
	console.warn("[WARN] cargo not found; skipping Rust license collection.");
}

const frontendData = await readJson(`${OUT}/frontend-licenses.json`);
if (frontendData) {
	let md = `# Frontend Licenses\n\n_Generated: ${new Date().toISOString()}_\n\n`;

	for (const [pkg, m] of Object.entries<any>(frontendData)) {
		md += `## ${pkg}\n- License: ${m.licenses ?? "UNKNOWN"}\n`;

		if (m.repository) {
			md += `- Repository: ${m.repository}\n`;
		}

		md += "\n";

		if (m.licenseFile && (await exists(m.licenseFile))) {
			md += "```text\n" + clean(await readFile(m.licenseFile, "utf8")) + "\n```\n\n";
		}
	}

	await writeFile(`${OUT}/frontend-licenses.md`, md);
} else {
	console.warn("[WARN] frontend-licenses.json missing or empty; skipping frontend report.");
}

const cargoData = await readJson(`${OUT}/rust-licenses.json`);
if (cargoData) {
	let md = `# Rust Crate Licenses\n\n_Generated: ${new Date().toISOString()}_\n\n`;
	md += "> Full license texts are available in [`rust-licenses.html`](./rust-licenses.html).\n\n";

	for (const c of cargoData) {
		md += `- **${c.name}** ${c.version} — ${c.license ?? "UNKNOWN"}`;

		if (c.repository) {
			md += ` ([repo](${c.repository}))`;
		}

		md += "\n";
	}

	await writeFile(`${OUT}/rust-licenses.md`, md);
} else {
	console.warn("[WARN] rust-licenses.json missing or empty; skipping Rust report.");
}

let all = `# Third-Party Licenses — tauri-video-cut\n\n_Generated: ${new Date().toISOString()}_\n\n`;
for (const f of ["frontend-licenses.md", "rust-licenses.md"]) {
	if (await exists(join(OUT, f))) {
		all += (await readFile(join(OUT, f), "utf8")) + "\n---\n\n";
	}
}

if (await exists(join(OUT, "msys2/packages.tsv"))) {
	all +=
		"## MSYS2 / UCRT64 runtime packages\n\nSee `legal/generated/msys2/packages.tsv` and `legal/generated/msys2/licenses/`.\n\n---\n\n";
}

all += "## FFmpeg / mpv (GPL-2.0)\n\nSee `NOTICE.md`, `SOURCE-OFFER.md`, and the repository tags.\n";

await writeFile(join(OUT, "THIRD_PARTY_LICENSES.md"), all);

console.log("Done →", join(OUT, "THIRD_PARTY_LICENSES.md"));
