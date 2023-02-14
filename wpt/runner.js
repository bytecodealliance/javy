import "./global_fix.js";
import "./upstream/resources/testharness.js";
import { resultReporter } from "./reporter.js";

// This is not a normal import and will be handled
// by a custom rollup plugin in `rollup.config.js`.
import testFunc from "custom:test_spec";

add_result_callback(resultReporter);
testFunc();
