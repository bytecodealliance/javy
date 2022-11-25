function main(i) {
    let input = JSON.parse(new TextDecoder().decode(i));
    const configuration = input?.discountNode?.metafield?.value || '{}';
    const vipMetafield = input.cart?.buyerIdentity?.customer?.metafield?.value;

    const textEncoder = new TextEncoder();
    if (vipMetafield != "true") {
        return textEncoder.encode(JSON.stringify({
            discountApplicationStrategy: "MAXIMUM",
            discounts: []
        })).buffer;
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
    })).buffer;
}

Shopify = {
    main
};
