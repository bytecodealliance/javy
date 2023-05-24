const random = Math.random();
Javy.IO.writeSync(1, new Uint8Array(new TextEncoder().encode(random.toString())));
