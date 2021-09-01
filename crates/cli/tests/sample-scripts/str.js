var Shopify = {
  main: (i) => {
    if (i === "hello") {
      return "world";
    }
    throw new Error("unreachable");
  }
}
