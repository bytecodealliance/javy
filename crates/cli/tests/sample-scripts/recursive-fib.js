function main(input) {
  const num = new Uint8Array(input)[0];
  const output = fibonacci(num);
  return new Uint8Array([output]).buffer;
}

function fibonacci(num) {
  if (num <= 1) return 1;
  return fibonacci(num - 1) + fibonacci(num - 2);
}

var Shopify = {
  main,
};
