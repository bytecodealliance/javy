#!/usr/bin/env node

import * as os from "os";
import * as path from "path";
import * as fs from "fs";
import * as childProcess from "child_process";
import * as gzip from "zlib";
import * as stream from "stream";
import fetch from "node-fetch";

const JAVY_URL = "https://github.com/Shopify/javy/releases/";
const JAVY_VERSION = "0.4.0";

async function main() {
	if (!(await isJavyAvailable()) || process.env.REFRESH_JAVY) {
		console.error("Javy is not available locally.");
		await fs.promises.unlink(javyBinaryPath()).catch(() => {});
		if (process.env.BUILD_JAVY) {
			console.error("Building Javy from source...");
			await buildJavy();
			console.error("Done.");
		} else {
			console.error("Downloading Javy...");
			await downloadJavy();
			console.error("Done.");
		}
	}
	try {
		childProcess.spawnSync(javyBinaryPath(), getArgs(), { stdio: "inherit" });
	} catch (e) {
		if (typeof e?.status === "number") return;
		console.error(e);
	}
}
main();

function cacheDir(...suffixes) {
	const cacheDir = path.join(os.homedir(), ".javy_cache", ...suffixes);
	fs.mkdirSync(cacheDir, { recursive: true });
	return cacheDir;
}

function javyBinaryPath() {
	return path.join(cacheDir(), "javy");
}

async function isJavyAvailable() {
	return fs.promises
		.stat(javyBinaryPath())
		.then(() => true)
		.catch(() => false);
}

async function downloadJavy() {
	const compressedStream = await new Promise(async (resolve) => {
		const resp = await fetch(binaryUrl());
		resolve(resp.body);
	});
	const gunzip = gzip.createGunzip();
	const output = fs.createWriteStream(javyBinaryPath());

	await new Promise((resolve, reject) => {
		stream.pipeline(compressedStream, gunzip, output, (err, val) => {
			if (err) return reject(err);
			return resolve(val);
		});
	});

	await fs.promises.chmod(javyBinaryPath(), 0o775);
}

function binaryUrl() {
	// https://github.com/Shopify/javy/releases/download/v0.3.0/javy-x86_64-linux-v0.3.0.gz
	return `${JAVY_URL}/download/v${JAVY_VERSION}/javy-${platarch()}-v${JAVY_VERSION}.gz`;
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

async function buildJavy() {
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
		javyBinaryPath()
	);
}
