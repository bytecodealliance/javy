class TextDecoder {
    constructor(label, options) {
        if (label && label !== "utf-8") {
            // Not spec-compliant behaviour
            throw new Error("Labels other than utf-8 are not supported");
        }
        this.fatal = options?.fatal;
        this.ignoreBOM = options?.ignoreBOM;
    }

    decode(buffer, options) {
        if (!buffer) {
            return "";
        }
        if (options && options.stream) {
            // FIXME
            throw new Error("Streaming decoding is not supported");
        }

        // FIXME take `fatal` and `ignoreBOM` into account
        if (ArrayBuffer.isView(buffer)) {
            // FIXME take offset and length of typed array or data view into account
            buffer = buffer.buffer;
        }
        if (!(buffer instanceof ArrayBuffer)) {
            throw new Error("buffer must be ArrayBuffer, TypedArray, or DataView");
        }
        return javy.decodeUtf8BufferToString(buffer);
    }
}

class TextEncoder {
    encode(str) {
        return new Uint8Array(javy.encodeStringToUtf8(str));
    }
}
