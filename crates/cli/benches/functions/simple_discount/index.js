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

Shopify = {
    main
};
