function fibonacci(input) {
  var num = new Uint8Array(input)[0];
  var a = 1, b = 0, temp;

  while (num >= 0) {
    temp = a;
    a = a + b;
    b = temp;
    num--;
  }

  return new Uint8Array([b]).buffer;
}

var Shopify = {
  main: fibonacci,
};
