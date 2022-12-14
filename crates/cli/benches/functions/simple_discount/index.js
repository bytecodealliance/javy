function main(input) {
    const configuration = JSON.parse(input?.discountNode?.metafield?.value || '{}');
    const vipMetafield = input.cart?.buyerIdentity?.customer?.metafield?.value;

    if (vipMetafield != "true") {
        return {
            discountApplicationStrategy: "MAXIMUM",
            discounts: []
        }
    }

    return {
        discountApplicationStrategy: "MAXIMUM",
        discounts: [
            {
                message: "VIP Discount",
                targets: [
                    {
                        orderSubtotal: {
                            excludedVariantIds: [],
                        }
                    }
                ],
                value: {
                    percentage: {
                        value: configuration.discountPercentage,
                    }
                }
            }
        ]
    };
}

let buffer = new Uint8Array(1024);
const bytesRead = Javy.IO.readSync(0, buffer);
buffer = buffer.subarray(0, bytesRead);
const output = new TextEncoder().encode(JSON.stringify(main(JSON.parse(new TextDecoder().decode(buffer)))));
Javy.IO.writeSync(1, output);
