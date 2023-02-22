// to build this for benchmarking, run `npm run build`

import _ from "underscore";

const EMPTY_DISCOUNT = {
    discountApplicationStrategy: "First",
    discounts: [],
};

function main(input) {
    const configuration = JSON.parse(
        _.get(input, ["discountNode", "metafield", "value"], "{}")
    );

    const vipMetafield = JSON.parse(
        _.get(input, ["cart", "buyerIdentity", "customer", "metafield", "value"], "{}")
    );

    if (!vipMetafield) return EMPTY_DISCOUNT;

    const sortedCart = _.chain(input.cart.lines).sortBy(item => item.quantity).map(item => ({ ...item, id: _.escape(item.id) })).value();
    const totalItems = _.reduce(sortedCart, (sum, item) => sum + item.quantity, 0);

    if (totalItems < 0) return EMPTY_DISCOUNT;

    return {
        discountApplicationStrategy: "Maximum",
        discounts: [
            {
                message: "VIP Discount",
                targets: [
                    {
                        productVariant: {
                            id: sortedCart[0].id
                        },
                    },
                ],
                value: {
                    percentage: {
                        value: configuration.discountPercentage,
                    },
                },
            },
        ],
    };
}

let buffer = new Uint8Array(1024);
const bytesRead = Javy.IO.readSync(0, buffer);
buffer = buffer.subarray(0, bytesRead);
const output = new TextEncoder().encode(JSON.stringify(main(JSON.parse(new TextDecoder().decode(buffer)))));
Javy.IO.writeSync(1, output);

