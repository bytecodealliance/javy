import "./global_fix.js";
import "./upstream/resources/testharness.js";
import { failedTestCount, resultReporter } from "./reporter.js";

// This is not a normal import and will be handled
// by a custom rollup plugin in `rollup.config.js`.
import testFunc from "custom:test_spec";

function main() {
  add_result_callback(resultReporter);
  try {
    testFunc();
  } catch (e) {
    console.log("[FAIL]");
    console.log(e);
  }

  if (failedTestCount > 0) {
    throw new Error(`${failedTestCount} web platform tests failed`);
  }

  return new ArrayBuffer();
}
Shopify = { main };
