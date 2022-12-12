var Shopify = {
  main: (input) => {
    const i = new TextDecoder().decode(input);
    if (i === "hello") {
      return new TextEncoder().encode("world").buffer;
    }
    throw new Error("unreachable");
  }
}
