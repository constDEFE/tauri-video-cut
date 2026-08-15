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

console.log("Frontend licenses...");
await tryRun(
	"license-checker",
	$`bunx --bun license-checker-rseidelsohn --production --json --files > ${OUT}/frontend-licenses.json`
);

console.log("Rust licenses...");
const MANIFEST = "src-tauri/Cargo.toml";
if (has("cargo")) {
	await tryRun("cargo-license", $`cargo license --manifest-path ${MANIFEST} --json > ${OUT}/rust-licenses.json`);
	await tryRun("cargo-deny", $`cargo deny --manifest-path ${MANIFEST} check licenses`);

	if ((await exists("legal/templates/about.hbs")) && (await exists("legal/about.toml"))) {
		await tryRun(
			"cargo-about",
			$`cargo about generate --manifest-path ${MANIFEST} --config legal/about.toml legal/templates/about.hbs > ${OUT}/rust-licenses.html`
		);
	}
}

if (await exists(`${OUT}/frontend-licenses.json`)) {
	const data = JSON.parse(await readFile(`${OUT}/frontend-licenses.json`, "utf8"));
	let md = `# Frontend Licenses\n\n_Generated: ${new Date().toISOString()}_\n\n`;

	for (const [pkg, m] of Object.entries<any>(data)) {
		md += `## ${pkg}\n- License: ${m.licenses ?? "UNKNOWN"}\n`;

		if (m.repository) {
			md += `- Repository: ${m.repository}\n`;
		}

		md += "\n";

		if (m.licenseFile && (await exists(m.licenseFile)))
			md += "```text\n" + clean(await readFile(m.licenseFile, "utf8")) + "\n```\n\n";
		{
		}
	}

	await writeFile(`${OUT}/frontend-licenses.md`, md);
}

if (await exists(`${OUT}/rust-licenses.json`)) {
	const crates = JSON.parse(await readFile(`${OUT}/rust-licenses.json`, "utf8"));
	let md = `# Rust Crate Licenses\n\n_Generated: ${new Date().toISOString()}_\n\n`;
	for (const c of crates) {
		md += `## ${c.name} ${c.version}\n- License: ${c.license ?? "UNKNOWN"}\n`;

		if (c.repository) {
			md += `- Repository: ${c.repository}\n`;
		}

		md += "\n";

		if (c.license_file && (await exists(c.license_file))) {
			md += "```text\n" + clean(await readFile(c.license_file, "utf8")) + "\n```\n\n";
		}
	}
	await writeFile(`${OUT}/rust-licenses.md`, md);
}

// Combine
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
