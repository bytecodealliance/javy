// Enhanced console functionality test
console.log("Testing enhanced console functionality");

// Test console.log (should go to stdout)
console.log("This is a log message");

// Test console.error (should go to stderr)  
console.error("This is an error message");

// Test console.warn (should go to stderr)
console.warn("This is a warning message");

// Test with multiple arguments
console.log("Log with", "multiple", "arguments");
console.warn("Warn with", "multiple", "arguments");  
console.error("Error with", "multiple", "arguments");

// Test with different data types
console.log("Number:", 42);
console.warn("Boolean:", true);
console.error("Object:", { key: "value" });

console.log("Console tests completed"); 