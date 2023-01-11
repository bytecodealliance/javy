export let failedTestCount = 0;

function logFailure({ name, message, stack }) {
  console.log("[FAIL]", name);
  console.log(message);
  console.log(stack);
  failedTestCount += 1;
}

export function resultReporter(test) {
  // No logging on success;
  if (test.status === 0) return;
  let ignoredTests = (globalThis.ignoredTests ?? []).map((matcher) => {
    // If a string starts with a slash, treat it like a RegExp.
    if (matcher.startsWith("/")) {
      return new RegExp(matcher.slice(1).slice(0, -1));
    }
    return matcher;
  });
  const shouldSkipTest = ignoredTests.some((matcher) => {
    if (matcher instanceof RegExp) return matcher.test(test.name);
    return matcher === test.name;
  });
  if (shouldSkipTest) return;
  logFailure(test);
}

export function completionReporter(tests, testStatus) {
  if (testStatus.status == 0) return;
  // For some reason, neither the `tests` object nor the `testStatus`
  // object contain a name to reference. We will have to work with the
  // stack if this one goes wrong.
  logFailure({ name: "???", ...testStatus });
}
