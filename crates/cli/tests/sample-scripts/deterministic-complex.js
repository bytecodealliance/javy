// Exercises many runtime code paths during Wizer pre-initialization to stress
// parallel compilation ordering and NaN canonicalization in Cranelift.

// --- Floating-point operations that produce NaN ---
const nanResults = [];
nanResults.push(0 / 0);
nanResults.push(Math.sqrt(-1));
nanResults.push(parseFloat("not a number"));
nanResults.push(Math.log(-1));
nanResults.push(Math.acos(2));
nanResults.push(Math.asin(2));
nanResults.push(Infinity - Infinity);
nanResults.push(Infinity * 0);
nanResults.push(undefined + 1);

// --- Random number generation ---
const randomResults = [];
for (let i = 0; i < 50; i++) {
  randomResults.push(Math.random());
}

// --- Date/time ---
const timestamps = [];
for (let i = 0; i < 10; i++) {
  timestamps.push(Date.now());
  timestamps.push(new Date().toISOString());
}

// --- Many distinct functions to increase compiled function count ---
function fib(n) { return n <= 1 ? n : fib(n - 1) + fib(n - 2); }
function factorial(n) { return n <= 1 ? 1 : n * factorial(n - 1); }
function isPrime(n) {
  if (n < 2) return false;
  for (let i = 2; i * i <= n; i++) { if (n % i === 0) return false; }
  return true;
}
function gcd(a, b) { return b === 0 ? a : gcd(b, a % b); }
function lcm(a, b) { return (a / gcd(a, b)) * b; }

function bubbleSort(arr) {
  const a = arr.slice();
  for (let i = 0; i < a.length; i++) {
    for (let j = 0; j < a.length - i - 1; j++) {
      if (a[j] > a[j + 1]) { const t = a[j]; a[j] = a[j + 1]; a[j + 1] = t; }
    }
  }
  return a;
}

function mergeSort(arr) {
  if (arr.length <= 1) return arr;
  const mid = Math.floor(arr.length / 2);
  const left = mergeSort(arr.slice(0, mid));
  const right = mergeSort(arr.slice(mid));
  const result = [];
  let i = 0, j = 0;
  while (i < left.length && j < right.length) {
    if (left[i] <= right[j]) result.push(left[i++]);
    else result.push(right[j++]);
  }
  return result.concat(left.slice(i)).concat(right.slice(j));
}

function matMul(a, b) {
  const rows = a.length, cols = b[0].length, inner = b.length;
  const c = Array.from({ length: rows }, () => new Array(cols).fill(0));
  for (let i = 0; i < rows; i++)
    for (let j = 0; j < cols; j++)
      for (let k = 0; k < inner; k++)
        c[i][j] += a[i][k] * b[k][j];
  return c;
}

function sha256ish(str) {
  let h = 0x6a09e667;
  for (let i = 0; i < str.length; i++) {
    h = Math.imul(h ^ str.charCodeAt(i), 0x5bd1e995);
    h ^= h >>> 15;
  }
  return (h >>> 0).toString(16);
}

function generatePermutations(arr) {
  if (arr.length <= 1) return [arr];
  const result = [];
  for (let i = 0; i < arr.length; i++) {
    const rest = arr.slice(0, i).concat(arr.slice(i + 1));
    for (const perm of generatePermutations(rest)) {
      result.push([arr[i], ...perm]);
    }
  }
  return result;
}

// --- Exercise all the functions at init time ---
const fibResults = [];
for (let i = 0; i < 20; i++) fibResults.push(fib(i));

const factResults = [];
for (let i = 0; i < 15; i++) factResults.push(factorial(i));

const primes = [];
for (let i = 0; i < 100; i++) { if (isPrime(i)) primes.push(i); }

const gcdResults = [];
for (let i = 1; i <= 20; i++) {
  for (let j = 1; j <= 20; j++) gcdResults.push(gcd(i, j));
}

const unsorted = Array.from({ length: 50 }, (_, i) => 50 - i);
const bubbled = bubbleSort(unsorted);
const merged = mergeSort(unsorted);

const matA = [[1.1, 2.2, 3.3], [4.4, 5.5, 6.6], [7.7, 8.8, 9.9]];
const matB = [[9.9, 8.8, 7.7], [6.6, 5.5, 4.4], [3.3, 2.2, 1.1]];
const matC = matMul(matA, matB);

const hashes = [];
for (let i = 0; i < 50; i++) hashes.push(sha256ish("test-string-" + i));

const perms = generatePermutations([1, 2, 3, 4, 5]);

// --- Float operations that stress NaN propagation through computation ---
const floatChain = [];
let val = 1.0;
for (let i = 0; i < 100; i++) {
  val = Math.sin(val) * Math.cos(val * 1.1) + Math.tan(val * 0.7);
  if (isNaN(val)) val = 0.0;
  floatChain.push(val);
}

// --- Build the result ---
const result = {
  nanCount: nanResults.filter(isNaN).length,
  randomSample: randomResults.slice(0, 5),
  timestamps: timestamps.slice(0, 4),
  fib19: fibResults[19],
  fact14: factResults[14],
  primeCount: primes.length,
  gcdSample: gcdResults.slice(0, 5),
  sorted: bubbled.slice(0, 5),
  merged: merged.slice(0, 5),
  matC00: matC[0][0],
  hashSample: hashes.slice(0, 3),
  permCount: perms.length,
  floatLast: floatChain[floatChain.length - 1],
};

const output = JSON.stringify(result);
Javy.IO.writeSync(1, new Uint8Array(new TextEncoder().encode(output)));
