// An exception thrown while looking up `toJSON` must propagate out of
// `JSON.stringify` instead of being dropped.
const boom = new Error("boom");
const values = [
  {
    get toJSON() {
      throw boom;
    },
  },
  new Proxy(
    { a: 1 },
    {
      get(target, key) {
        if (key === "toJSON") {
          throw boom;
        }
        return Reflect.get(target, key);
      },
    }
  ),
];

for (const value of values) {
  let thrown;
  try {
    JSON.stringify(value);
  } catch (e) {
    thrown = e;
  }
  if (thrown !== boom) {
    throw new Error(`expected JSON.stringify to throw, got ${thrown}`);
  }
}
