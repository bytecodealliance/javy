#!/usr/bin/env node

import * as os from "os";
import * as path from "path";
import * as fs from "fs";
import * as childProcess from "child_process";
import * as gzip from "zlib";
import * as stream from "stream";
import fetch from "node-fetch";

const REPO = "Shopify/javy";
const NAME = "javy";

async function main() {
	if (!(await isBinaryDownloaded()) || shouldIgnoreLocalBinary()) {
		console.error(`${NAME} is not available locally.`);
		await fs.promises.unlink(binaryPath()).catch(() => {});
		if (process.env.FORCE_FROM_SOURCE) {
			console.error(`Building ${NAME} from source...`);
			await buildBinary();
			console.error(`Done.`);
		} else {
			console.error(`${NAME} needs to be downloaded...`);
			await downloadBinary();
			console.error(`Done.`);
		}
	}
	try {
		childProcess.spawnSync(binaryPath(), getArgs(), { stdio: "inherit" });
	} catch (e) {
		if (typeof e?.status === "number") return;
		console.error(e);
	}
}
main();

function shouldIgnoreLocalBinary() {
	return process.env.FORCE_RELEASE || process.env.FORCE_FROM_SOURCE;
}

function cacheDir(...suffixes) {
	const cacheDir = path.join(os.homedir(), ".binary_cache", ...suffixes);
	fs.mkdirSync(cacheDir, { recursive: true });
	return cacheDir;
}

function binaryPath() {
	return path.join(cacheDir(), NAME);
}

async function isBinaryDownloaded() {
	return fs.promises
		.stat(binaryPath())
		.then(() => true)
		.catch(() => false);
}

async function downloadBinary() {
	const compressedStream = await new Promise(async (resolve) => {
		const { url, version } = await binaryUrl();
		console.log(`Downloading ${NAME} ${version}...`);
		const resp = await fetch(url);
		resolve(resp.body);
	});
	const gunzip = gzip.createGunzip();
	const output = fs.createWriteStream(binaryPath());

	await new Promise((resolve, reject) => {
		stream.pipeline(compressedStream, gunzip, output, (err, val) => {
			if (err) return reject(err);
			return resolve(val);
		});
	});

	await fs.promises.chmod(binaryPath(), 0o775);
}

async function binaryUrl() {
	let version = process.env.FORCE_RELEASE;
	// If no version is forced, use the GitHub API to grab the latest release.
	if (!version || version?.toLowerCase() === "latest") {
		const releaseDataResponse = await fetch(
			`https://api.github.com/repos/${REPO}/releases?per_page=3`,
			{
				headers: {
					Accept: "application/vnd.github+json",
				},
			}
		);
		if (!releaseDataResponse.ok) {
			throw Error(
				`Could not determine latest release using the GitHub API (Status code ${
					releaseDataResponse.status
				}): ${await releaseDataResponse
					.text()
					.catch(() => "<No error message>")}`
			);
		}
		const releaseData = await releaseDataResponse.json();
		version = releaseData.find((release) => release.tag_name)?.tag_name;
		if (!version) {
			throw Error(
				"None of the three most recent release have a valid tag name."
			);
		}
	}
	const url = `https://github.com/${REPO}/releases/download/${version}/javy-${platarch()}-${version}.gz`;
	return { url, version };
}

const SUPPORTED_TARGETS = [
	"arm-macos",
	"x64_64-macos",
	"x64_64-windows",
	"x64_64-linux",
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
	// TODO: Check if this needs to be changed when javy is installed via `npm install`.
	return args;
}

async function buildBinary() {
	const repoDir = cacheDir("build", "javy");
	try {
		console.log("Downloading Javy's source code...");
		childProcess.execSync(
			`git clone https://github.com/shopify/javy ${repoDir}`
		);
		console.log("Downloading WASI SDK...");
		childProcess.execSync("make download-wasi-sdk", { cwd: repoDir });
		console.log("Building Javy...");
		childProcess.execSync("make", { cwd: repoDir });
	} catch (e) {
		console.error(e);
		console.error("");
		console.error("BUILDING JAVY FAILED");
		console.error(
			"Please make sure you have cmake, Rust with the wasm32-wasi target, wasmtime-cli and cargo-wasi installed"
		);
		console.error("See the javy README for more details.");
	}
	await fs.promises.rename(
		path.join(repoDir, "target", "release", "javy"),
		binaryPath()
	);
}
