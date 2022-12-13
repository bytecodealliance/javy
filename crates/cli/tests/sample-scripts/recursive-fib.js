function fibonacci(num) {
  if (num <= 1) return 1;
  return fibonacci(num - 1) + fibonacci(num - 2);
}

const buffer = new Uint8Array(1);
Javy.IO.readSync(0, buffer);
const result = fibonacci(buffer[0]);
buffer[0] = result;
Javy.IO.writeSync(1, buffer);
