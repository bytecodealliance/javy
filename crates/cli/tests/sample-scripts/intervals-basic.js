// Basic interval functionality test
console.log("Testing setInterval functionality");

// Test 1: Basic interval
globalThis.counter1 = 0;
const id1 = setInterval("globalThis.counter1++; console.log('Interval 1:', globalThis.counter1); if(globalThis.counter1 >= 2) clearInterval(id1)", 0);

// Test 2: Interval cancellation
const cancelMe = setInterval("console.log('ERROR: This should not execute')", 1000);
clearInterval(cancelMe);
console.log("Interval cancellation successful");

// Test 3: Multiple intervals
globalThis.counter2 = 0;
const id2 = setInterval("globalThis.counter2++; console.log('Interval 2:', globalThis.counter2); if(globalThis.counter2 >= 2) clearInterval(id2)", 0);

// Test 4: Mixed with setTimeout
setTimeout("console.log('Timeout executed alongside intervals')", 0);

// Test 5: Self-clearing interval
globalThis.counter3 = 0;
globalThis.selfClearId = setInterval("globalThis.counter3++; console.log('Self-clearing:', globalThis.counter3); if(globalThis.counter3 >= 2) { clearInterval(globalThis.selfClearId); console.log('Self-cleared!'); }", 0);

console.log("All interval tests scheduled"); 