export function result_reporter(test) {
  // No logging on success;
  if (test.status == 0) return;
  console.log("[FAIL]", test.name);
  console.log(test.message);
  console.log(test.stack);
}
