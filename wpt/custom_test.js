test(() => {
  const a = 1;
  const b = 2;
  assert_equals(a, b, "Variables should be equal");
}, "This is an ignored test");

test(() => {
  const a = 1;
  const b = 1;
  assert_equals(a, b, "Variables should be equal");
}, "Real test");
