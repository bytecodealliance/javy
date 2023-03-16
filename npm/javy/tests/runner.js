import { spawn } from "node:child_process";
import * as stream from "node:stream";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { randomUUID } from "node:crypto";
import {
	ReadableStream,
	TextDecoderStream,
	TextEncoderStream,
} from "node:stream/web";
import { unlink } from "node:fs/promises";

import { rollup } from "rollup";
import swc from "rollup-plugin-swc";
import nodeResolve from "@rollup/plugin-node-resolve";

import * as tests from "./tests.js";

async function main() {
	console.log("Running tests...");

	await forceJavyDownload(); // trying to download Javy in parallel causes problems with the tests

	const resultPromises = Object.entries(tests).map(
		async ([testName, testFunc]) => {
			try {
				const value = await testFunc();
				return { testName, success: true, value };
			} catch (err) {
				return { testName, sucess: false, value: err };
			}
		}
	);
	const results = await Promise.all(resultPromises);

	for (const { testName, success, value } of results) {
		const marker = success ? "PASS" : "FAIL";
		console.log(`[${marker}] ${testName}${success ? "" : `: ${value}`}`);
	}
	process.exit(results.every(r => r.success) ? 0 : 1);
}
await main();

async function forceJavyDownload() {
	let { exitCode, stderr } = await runCommand("javy", ["--version"]);
	if (await exitCode !== 0) {
		throw Error(await collectStream(stderr));
	}
}

/**
 * Passes the stream's controller to a callback where strings can be enqueue.
 * @param {(ctr: ReadableStreamDefaultController<string>) => Promise<void>} f
 * @returns {ReadableStream<Uint8Array>} Binary stream
 */
export function customStringInputStream(f) {
	return new ReadableStream({
		async start(controller) {
			await f(controller);
			controller.close();
		},
	}).pipeThrough(new TextEncoderStream());
}

/**
 * Creates a ReadableStream from a given string.
 * @param {string} str
 * @returns {ReadableStream<Uint8Array>} Binary stream
 */
export function stringAsInputStream(str) {
	const CHUNK_SIZE = 10;
	return new ReadableStream({
		start(controller) {
			// Artificial chunking
			for (let i = 0; i < str.length; i += CHUNK_SIZE) {
				const substr = str.slice(i, i + CHUNK_SIZE);
				controller.enqueue(substr);
			}
			controller.close();
		},
	}).pipeThrough(new TextEncoderStream());
}

/**
 * Runs the given source code with the given input stream and compares the programs output against the expected output string.
 * @param {Object} options
 * @param {string} options.source JS source code.
 * @param {ReadableStream<Uint8Array>} options.stdin Binary input stream.
 * @param {string} options.expectedOutput Expected output as a string.
 * @returns {Promise<void>} Resolves on success, rejects on failure.
 */
export async function runJS({ source, stdin, expectedOutput }) {
	const dir = tmpdir();
	const outfile = join(dir, `${randomUUID()}.wasm`);
	const rawInfile = new URL(source, import.meta.url).pathname;
	const infile = join(dir, `${randomUUID()}.js`);
	const bundle = await rollup({
		input: rawInfile,
		plugins: [
			nodeResolve({
				extensions: [".mjs", ".js", ".ts"],
			}),
			swc.default(),
		],
	});
	await bundle.write({
		file: infile,
	});
	await compileWithJavy(infile, outfile);
	const { exitCode, stdout, stderr } = await runCommand(
		"wasmtime",
		[outfile],
		stdin
	);
	if ((await exitCode) != 0) {
		throw Error(await collectStream(stderr));
	}
	const output = await collectStream(stdout);
	if (output != expectedOutput) {
		throw Error(`Unexpected output.\n${infile}\n${outfile}`);
	}
	await unlink(outfile);
}

async function compileWithJavy(infile, outfile) {
	const { exitCode, stdout, stderr } = await runCommand("javy", [
		"compile",
		"-o",
		outfile,
		infile,
	]);
	if ((await exitCode) != 0) {
		throw Error(await collectStream(stderr));
	}
}
/**
 * @param {ReadableStream} stdin
 */
async function runCommand(cmd, args, stdin = emptyStream()) {
	const process = spawn(cmd, args, {
		stdio: "pipe",
	});
	stdin.pipeTo(stream.Writable.toWeb(process.stdin));

	const exitCode = new Promise((resolve) => {
		process.on("exit", (code) => resolve(code));
	});

	return {
		exitCode,
		stdout: stream.Readable.toWeb(process.stdout),
		stderr: stream.Readable.toWeb(process.stderr),
	};
}

/**
 * @param {ReadableStream} stream
 */

async function collectStream(stream) {
	const items = [];
	const reader = stream.pipeThrough(new TextDecoderStream()).getReader();
	while (true) {
		const { value, done } = await reader.read();
		if (done) return items.join("");
		items.push(value);
	}
}
function emptyStream() {
	return new ReadableStream({
		start(controller) {
			controller.close();
		},
	});
}
