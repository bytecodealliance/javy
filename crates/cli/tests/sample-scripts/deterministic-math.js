function collatz(n) {
  const seq = [n];
  while (n !== 1) {
    n = n % 2 === 0 ? n / 2 : 3 * n + 1;
    seq.push(n);
  }
  return seq;
}

function sieve(limit) {
  const flags = new Array(limit + 1).fill(true);
  flags[0] = flags[1] = false;
  for (let i = 2; i * i <= limit; i++) {
    if (flags[i]) {
      for (let j = i * i; j <= limit; j += i) flags[j] = false;
    }
  }
  return flags.reduce((acc, v, i) => { if (v) acc.push(i); return acc; }, []);
}

function mandelbrot(cx, cy, maxIter) {
  let x = 0, y = 0, iter = 0;
  while (x * x + y * y <= 4 && iter < maxIter) {
    const tmp = x * x - y * y + cx;
    y = 2 * x * y + cy;
    x = tmp;
    iter++;
  }
  return iter;
}

function luDecompose(matrix) {
  const n = matrix.length;
  const L = Array.from({ length: n }, (_, i) => {
    const row = new Array(n).fill(0);
    row[i] = 1;
    return row;
  });
  const U = matrix.map(r => r.slice());
  for (let k = 0; k < n; k++) {
    for (let i = k + 1; i < n; i++) {
      const factor = U[i][k] / U[k][k];
      L[i][k] = factor;
      for (let j = k; j < n; j++) U[i][j] -= factor * U[k][j];
    }
  }
  return { L, U };
}

const collatzResults = [];
for (let i = 1; i <= 30; i++) collatzResults.push(collatz(i).length);

const primes = sieve(500);

const mandelbrotGrid = [];
for (let y = -1.0; y <= 1.0; y += 0.1) {
  for (let x = -2.0; x <= 0.5; x += 0.1) {
    mandelbrotGrid.push(mandelbrot(x, y, 100));
  }
}

const mat = [
  [2.5, 1.3, 0.7, 3.1],
  [0.4, 5.2, 2.8, 1.6],
  [3.9, 0.8, 4.5, 2.3],
  [1.2, 3.7, 0.6, 6.1],
];
const { L, U } = luDecompose(mat);

const randomWalk = [];
let pos = 0;
for (let i = 0; i < 200; i++) {
  pos += Math.random() > 0.5 ? 1 : -1;
  randomWalk.push(pos);
}

const result = {
  collatzLengths: collatzResults,
  primeCount: primes.length,
  mandelbrotSum: mandelbrotGrid.reduce((a, b) => a + b, 0),
  luTrace: L[3][0] + U[0][0],
  walkFinal: randomWalk[randomWalk.length - 1],
  timestamps: [Date.now(), Date.now()],
};

const output = JSON.stringify(result);
Javy.IO.writeSync(1, new Uint8Array(new TextEncoder().encode(output)));
