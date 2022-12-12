const buffer = new Uint8Array(1);
Javy.IO.readSync(0, buffer);
Javy.IO.writeSync(1, buffer);
