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

const javyPath = new URL("../../../target/release/javy", import.meta.url)
	.pathname;

async function main() {
	console.log("Running tests...");
	const resultPromises = Object.entries(tests).map(([testName, testFunc]) =>
		Promise.resolve(testFunc())
			.then((value) => ({ testName, success: true, value }))
			.catch((value) => ({ testName, success: false, value }))
	);
	const results = await Promise.all(resultPromises);

	for (const { testName, success, value } of results) {
		const marker = success ? "PASS" : "FAIL";
		console.log(`[${marker}] ${testName}${success ? "" : `: ${value}`}`);
	}
}
await main();

export function stringFactoryInput(f) {
	return new ReadableStream({
		async start(controller) {
			await f(controller);
			controller.close();
		},
	}).pipeThrough(new TextEncoderStream());
}

/**
 * @param {String} str
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
	const { exitCode, stdout, stderr } = await runCommand(javyPath, [
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
