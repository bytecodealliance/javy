async function foo() {
  return Promise.resolve("bar");
}

const output = new TextEncoder().encode(await foo());
Javy.IO.writeSync(1, output);
