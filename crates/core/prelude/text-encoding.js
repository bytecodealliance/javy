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
})();
