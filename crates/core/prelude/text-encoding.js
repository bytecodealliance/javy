(function () {
    const __javy_decodeUtf8BufferToString = globalThis.__javy_decodeUtf8BufferToString;
    const __javy_encodeStringToUtf8Buffer = globalThis.__javy_encodeStringToUtf8Buffer;

    class TextDecoder {
        constructor(label = "utf-8", options = {}) {
            if (label !== "utf-8") {
                // Not spec-compliant behaviour
                throw new RangeError("The encoding label provided must be utf-8");
            }
            Object.defineProperties(this, {
                encoding: { value: "utf-8", enumerable: true, writable: false },
                fatal: { value: !!options.fatal, enumerable: true, writable: false },
                ignoreBOM: { value: !!options.ignoreBOM, enumerable: true, writable: false },
            })
        }

        decode(input, options = {}) {
            if (input === undefined) {
                return "";
            }

            if (options.stream) {
                throw new Error("Streaming decode is not supported");
            }

            // backing buffer would not have byteOffset and may have different byteLength
            let byteOffset = input.byteOffset || 0;
            let byteLength = input.byteLength;
            if (ArrayBuffer.isView(input)) {
                input = input.buffer;
            }

            if (!(input instanceof ArrayBuffer)) {
                throw new TypeError("The provided value is not of type '(ArrayBuffer or ArrayBufferView)'");
            }

            // ignoreBOM does not appear to change behaviour for UTF-8
            return __javy_decodeUtf8BufferToString(input, byteOffset, byteLength, this.fatal);
        }
    }

    class TextEncoder {
        constructor() {
            Object.defineProperties(this, {
                encoding: { value: "utf-8", enumerable: true, writable: false },
            });
        }

        encode(input = "") {
            input = input.toString(); // non-string inputs are converted to strings
            input = replaceUnpairedSurrogates(input); // QuickJS does not handle this correctly
            return new Uint8Array(__javy_encodeStringToUtf8Buffer(input));
        }

        encodeInto(source, destination) {
            throw new Error("encodeInto is not supported");
        }
    }

    globalThis.TextDecoder = TextDecoder;
    globalThis.TextEncoder = TextEncoder;

    Reflect.deleteProperty(globalThis, "__javy_decodeUtf8BufferToString");
    Reflect.deleteProperty(globalThis, "__javy_encodeStringToUtf8Buffer");

    function replaceUnpairedSurrogates(input) {
        const highSurrogateStart = '\uD800';
        const lowSurrogateStart = '\uDC00';
        const lowSurrogateEnd = '\uDFFF';
        const replacementChar = '\uFFFD';

        const segments = [];
        let start = 0;
        for (let i = 0; i < input.length; i++) {
            const currentChar = input[i];
            const isHighSurrogate = currentChar >= highSurrogateStart && currentChar < lowSurrogateStart;
            const isLowSurrogate = currentChar >= lowSurrogateStart && currentChar <= lowSurrogateEnd;

            if (
                (isHighSurrogate && !(input[i + 1] >= lowSurrogateStart && input[i + 1] <= lowSurrogateEnd))
                || (isLowSurrogate && !(input[i - 1] >= highSurrogateStart && input[i - 1] < lowSurrogateStart))
            ) {
                segments.push(input.substring(start, i));
                segments.push(replacementChar);
                start = i + 1;
            }
        }

        if (start === 0) {
            return input;
        }

        segments.push(input.substring(start));
        return segments.join("");
    }
})();
