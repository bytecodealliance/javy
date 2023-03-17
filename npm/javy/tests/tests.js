import {
	runJS,
	stringAsInputStream,
	customStringInputStream,
} from "./runner.js";

export async function smallEcho() {
	await runJS({
		source: "./fixtures/small_echo.js",
		expectedOutput: "123",
		stdin: stringAsInputStream("123"),
	});
}

export async function libraryEcho() {
	const len = 16 * 1024;
	await runJS({
		source: "./fixtures/lib_echo.js",
		expectedOutput: "x".repeat(len),
		stdin: stringAsInputStream("x".repeat(len)),
	});
}

export async function longDelay() {
	await runJS({
		source: "./fixtures/lib_echo.js",
		expectedOutput: "1234",
		stdin: customStringInputStream(async (controller) => {
			for (let i = 1; i <= 4; i++) {
				controller.enqueue(i.toString());
				await sleep(0);
			}
		}),
	});
}

export async function exceededWriteCapacity() {
	await runJS({
		source: "./fixtures/exceeded_write_capacity.js",
		expectedOutput: "",
	});
}

function sleep(ms) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}
