const buffer = new Uint8Array(1024);
const n = Javy.IO.readSync(0, buffer);
const input = new TextDecoder().decode(buffer.subarray(0, n));
if (input !== "hello") {
  throw new Error("unreachable");
}
const result = new TextEncoder().encode("world");
Javy.IO.writeSync(1, result);
