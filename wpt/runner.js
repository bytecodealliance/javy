import "./global_fix.js";
import "./upstream/resources/testharness.js";
import { result_reporter } from "./reporter.js";

// Magic
import testFunc from "custom:test_spec";

function main() {
  add_result_callback(result_reporter);
  try {
    testFunc();
  } catch (e) {
    console.log("[FAIL]");
    console.log(e);
  }
  return new ArrayBuffer();
}
Shopify = { main };
