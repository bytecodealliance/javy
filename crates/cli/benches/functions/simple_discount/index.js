function main(i) {
    let input = JSON.parse(textDecoder.decode(i));
    const configuration = input?.discountNode?.metafield?.value || '{}';
    const vipMetafield = input.cart?.buyerIdentity?.customer?.metafield?.value;

    if (vipMetafield != "true") {
        return textEncoder.encode(JSON.stringify({
            discountApplicationStrategy: "MAXIMUM",
            discounts: []
        }));
    }

    return textEncoder.encode(JSON.stringify({
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
    }));
}

Shopify = {
    main
};
