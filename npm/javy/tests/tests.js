import { runJS, stringAsInputStream } from "./runner.js";

export async function smallEcho() {
	await runJS({
		source: "./fixtures/small_echo.js",
		expectedOutput: "123",
		stdin: stringAsInputStream("123"),
	});
}

async function libraryEcho() {
	await runJS({
		source: "./fixtures/lib_echo.js",
		expectedOutput: "x".repeat(16 * 1024),
		stdin: stringAsInputStream("x".repeat(16 * 1024)),
	});
}
