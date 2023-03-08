#!/usr/bin/env node

import * as path from "path";
import * as fs from "fs";
import * as childProcess from "child_process";
import * as gzip from "zlib";
import * as stream from "stream";
import fetch from "node-fetch";
import cachedir from "cachedir";

const REPO = "Shopify/javy";
const NAME = "javy";

async function main() {
	const version = await getDesiredVersionNumber();
	if (!(await isBinaryDownloaded(version))) {
		if (process.env.FORCE_FROM_SOURCE) {
			await buildBinary();
		} else {
			await downloadBinary(version);
		}
	}
	try {
		childProcess.spawnSync(binaryPath(version), getArgs(), {
			stdio: "inherit",
		});
	} catch (e) {
		if (typeof e?.status === "number") return;
		console.error(e);
	}
}
main();

function cacheDir(...suffixes) {
	const cacheDir = path.join(cachedir("binarycache"), ...suffixes);
	fs.mkdirSync(cacheDir, { recursive: true });
	return cacheDir;
}

function binaryPath(version) {
	return path.join(cacheDir(), `${NAME}-${version}`);
}

async function isBinaryDownloaded(version) {
	return fs.promises
		.stat(binaryPath(version))
		.then(() => true)
		.catch(() => false);
}

async function downloadBinary(version) {
	const targetPath = binaryPath(version);
	const compressedStream = await new Promise(async (resolve) => {
		const url = binaryUrl(version);
		console.log(`Downloading ${NAME} ${version} to ${targetPath}...`);
		const resp = await fetch(url);
		resolve(resp.body);
	});
	const gunzip = gzip.createGunzip();
	const output = fs.createWriteStream(targetPath);

	await new Promise((resolve, reject) => {
		stream.pipeline(compressedStream, gunzip, output, (err, val) => {
			if (err) return reject(err);
			return resolve(val);
		});
	});

	await fs.promises.chmod(binaryPath(version), 0o775);
}

/**
 * getDesiredVersionNumber returns the version number of the release that
 * should be downloaded and launched. If the FORCE_RELEASE env variable is set,
 * that will be used as the desired version number, if not, we determine the
 * latest release available on GitHub.
 *
 * GitHub has a public Release API, but  rate limits it per IP, so that the
 * CLI can end up breaking. Instead, we use a little trick. You can download
 * artifacts from the latest release by using `latest` as your version number.
 * The server will respond with a 302 redirect to the artifact's URL. That URL
 * contains the actual release version number, which we can extract.
 */
async function getDesiredVersionNumber() {
	if (process.env.FORCE_RELEASE) return process.env.FORCE_RELEASE;
	const resp = await fetch(
		`https://github.com/${REPO}/releases/latest/download/lol`,
		{ redirect: "manual" }
	);
	if (resp.status != 302) {
		throw Error(
			`Could not determine latest release using the GitHub (Status code ${resp.status
			}): ${await resp.text().catch(() => "<No error message>")}`
		);
	}
	return resp.headers.get("location").split("/").at(-2);
}

function binaryUrl(version) {
	return `https://github.com/${REPO}/releases/download/${version}/${NAME}-${platarch()}-${version}.gz`;
}

const SUPPORTED_TARGETS = [
	"arm-linux",
	"arm-macos",
	"x86_64-macos",
	"x86_64-windows",
	"x86_64-linux",
];

function platarch() {
	let platform, arch;
	switch (process.platform.toLowerCase()) {
		case "darwin":
			platform = "macos";
			break;
		case "linux":
			platform = "linux";
			break;
		case "win32":
			platform = "windows";
			break;
		default:
			throw Error(`Unsupported platform ${process.platform}`);
	}
	switch (process.arch.toLowerCase()) {
		case "arm":
		case "arm64":
			arch = "arm";
			break;
		// A 32 bit arch likely needs that someone has 32bit Node installed on a
		// 64 bit system, and wasmtime doesn't support 32bit anyway.
		case "ia32":
		case "x64":
			arch = "x86_64";
			break;
		default:
			throw Error(`Unsupported architecture ${process.arch}`);
	}
	const result = `${arch}-${platform}`;
	if (!SUPPORTED_TARGETS.includes(result)) {
		throw Error(
			`Unsupported platform/architecture combination ${platform}/${arch}`
		);
	}
	return result;
}

function getArgs() {
	const args = process.argv.slice(2);
	return args;
}

async function buildBinary() {
	const repoDir = cacheDir("build", NAME);
	try {
		console.log(`Downloading ${NAME}'s source code...`);
		childProcess.execSync(
			`git clone https://github.com/${REPO} ${repoDir}`
		);
		console.log("Downloading WASI SDK...");
		childProcess.execSync("make download-wasi-sdk", { cwd: repoDir });
		console.log(`Building ${NAME}...`);
		childProcess.execSync("make", { cwd: repoDir });
	} catch (e) {
		console.error(e);
		console.error("");
		console.error(`BUILDING ${NAME} FAILED`);
		console.error(
			"Please make sure you have cmake, Rust with the wasm32-wasi target, wasmtime-cli and cargo-wasi installed"
		);
		console.error("See the README for more details.");
	}
	await fs.promises.rename(
		path.join(repoDir, "target", "release", NAME),
		binaryPath()
	);
}
