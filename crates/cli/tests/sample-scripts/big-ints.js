var Shopify = {
  main: (i) => {
    if (i === 42) { // serialized as number
      return "a";
    }

    if (i === BigInt("9223372036854775807")) { // i64::MAX
      return "b";
    }

    if (i === BigInt("-9223372036854775808")) { // i64::MIN
      return "c";
    }

    if (i === BigInt("18446744073709551615")) { // u64::MAX
      return "d";
    }

    if (i === 0) { // u64::MIN serialized as number
      return "e";
    }

    throw new Error("i is invalid");
  }
}
