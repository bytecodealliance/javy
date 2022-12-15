import * as tests from "./tests.js";

const resultPromises = Object.entries(tests).map(async ([testName, testFunc]) =>
	testFunc()
		.then((value) => ({ testName, success: true, value }))
		.catch((value) => ({ testName, success: false, value }))
);
const results = await Promise.all(resultPromises);

console.log(results);
for (const { testName, success, value } of results) {
	const marker = success ? "PASS" : "FAIL";
	console.log(`[${marker}] ${testName}${success ? "" : `: ${value}`}`);
}
