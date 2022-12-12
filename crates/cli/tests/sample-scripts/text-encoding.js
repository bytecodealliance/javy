function tests(i) {
    const decoder = new TextDecoder();
    const encoder = new TextEncoder();
    const invalidUnicode = new Uint8Array([0xFE]);
    if (i === "hello") {
        const offset = 1;
        const length = 2;
        return decoder.decode(new Uint8Array(encoder.encode(i).buffer, offset, length));
    } else if (i === "invalid") {
        return (decoder.decode(invalidUnicode) === "�").toString();
    } else if (i === "invalid_fatal") {
        try {
            new TextDecoder("utf-8", { fatal: true }).decode(invalidUnicode);
            return "failed";
        } catch (e) {
            return e.message;
        }
    } else {
        return decoder.decode(new Uint8Array(Array.from(encoder.encode(i)).concat("2".charCodeAt(0))));
    }
}

const buffer = new Uint8Array(1024);
const n = Javy.IO.readSync(0, buffer);
const input = new TextDecoder().decode(buffer.subarray(0, n));
const result = tests(input)
Javy.IO.writeSync(1, new TextEncoder().encode(result));
