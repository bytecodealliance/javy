
export async function main() {
  const expectedHex = "97d2a569059bbcd8ead4444ff99071f4c01d005bcefe0d3567e1be628e5fdcd9";

  const result = await crypto.subtle.sign({name: "HMAC", hash: "sha-256"}, "my secret and secure key", "input message");
  console.log(result);
  console.log(result === expectedHex);
}

await main();

