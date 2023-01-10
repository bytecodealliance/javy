const buffer = new Uint8Array(8);
const n = Javy.IO.readSync(0, buffer);
Javy.IO.writeSync(1, buffer.subarray(0, n));
