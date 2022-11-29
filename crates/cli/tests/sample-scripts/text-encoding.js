var Shopify = {
    main: (i) => {
        const decoder = new TextDecoder();
        const encoder = new TextEncoder();
        if (i === "hello") {
            const offset = 1;
            const length = 2;
            return decoder.decode(new Uint8Array(encoder.encode(i).buffer, offset, length));
        } else {
            return decoder.decode(new Uint8Array(Array.from(encoder.encode(i)).concat("2".charCodeAt(0))));
        }
    }
}
