import { runJS, stringAsInputStream } from "./runner.js";

export async function smallEcho() {
	await runJS({
		source: "./fixtures/small_echo.js",
		expectedOutput: "123",
		stdin: stringAsInputStream("123"),
	});
}
