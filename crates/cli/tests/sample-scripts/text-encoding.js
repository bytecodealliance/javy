var Shopify = {
    main: (i) => {
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
}
