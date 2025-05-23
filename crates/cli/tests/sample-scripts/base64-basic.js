// Base64 functionality test
console.log("Testing base64 functionality");

// Test 1: Basic encoding
const text1 = "Hello, World!";
const encoded1 = btoa(text1);
console.log("Encoded:", encoded1);

// Test 2: Basic decoding
const decoded1 = atob(encoded1);
console.log("Decoded:", decoded1);
console.log("Round-trip test:", text1 === decoded1 ? "PASS" : "FAIL");

// Test 3: Empty string
const empty = btoa("");
console.log("Empty string:", empty === "" ? "PASS" : "FAIL");
const emptyDecoded = atob("");
console.log("Empty decode:", emptyDecoded === "" ? "PASS" : "FAIL");

// Test 4: Standard test vectors
const tests = [
    { input: "f", expected: "Zg==" },
    { input: "fo", expected: "Zm8=" },
    { input: "foo", expected: "Zm9v" },
    { input: "foob", expected: "Zm9vYg==" },
    { input: "fooba", expected: "Zm9vYmE=" },
    { input: "foobar", expected: "Zm9vYmFy" }
];

for (const test of tests) {
    const result = btoa(test.input);
    console.log(`btoa("${test.input}"):`, result === test.expected ? "PASS" : `FAIL (got ${result}, expected ${test.expected})`);
    
    const decoded = atob(result);
    console.log(`atob("${result}"):`, decoded === test.input ? "PASS" : `FAIL (got ${decoded})`);
}

// Test 5: Error handling for invalid input
try {
    btoa("Hello 🌍"); // Contains Unicode outside Latin1
    console.log("Unicode test: FAIL (should have thrown)");
} catch (e) {
    console.log("Unicode test: PASS (correctly threw error)");
}

try {
    atob("invalid!@#"); // Invalid base64
    console.log("Invalid base64 test: FAIL (should have thrown)");
} catch (e) {
    console.log("Invalid base64 test: PASS (correctly threw error)");
}

console.log("Base64 tests completed"); 