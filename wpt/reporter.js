export default function (tests) {
  const hasFailures = tests.some((test) => test.status != 0);
  if (!hasFailures) {
    console.log("PASS");
    return;
  }
  console.log("FAIL");
  for (const test of tests.filter((test) => test.status == 0)) {
    console.log("[PASS]", test.name);
  }
  for (const test of tests.filter((test) => test.status != 0)) {
    console.log("[FAIL]", test.name);
    console.log(test.message);
    console.log(test.stack);
  }
}
