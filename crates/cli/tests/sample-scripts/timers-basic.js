// Basic timer functionality test
console.log("Testing basic setTimeout functionality");

// Test 1: Immediate execution (0 delay)
setTimeout("console.log('Timer 1: Immediate execution')", 0);

// Test 2: Also immediate for reliable testing
setTimeout("console.log('Timer 2: Also immediate')", 0);

// Test 3: Timer cancellation
const cancelMe = setTimeout("console.log('ERROR: This should not execute')", 1000);
clearTimeout(cancelMe);
console.log("Timer 3: Cancellation successful");

// Test 4: Multiple timers
setTimeout("console.log('Timer 4A: First')", 0);
setTimeout("console.log('Timer 4B: Second')", 0);
setTimeout("console.log('Timer 4C: Third')", 0);

// Test 5: Timer ID return values
const timerId = setTimeout("console.log('Timer 5: ID test')", 0);
console.log("Timer ID:", typeof timerId, timerId);

console.log("All timer tests scheduled"); 