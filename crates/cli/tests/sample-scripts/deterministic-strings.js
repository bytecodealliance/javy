function rot13(str) {
  return str.replace(/[a-zA-Z]/g, c => {
    const base = c <= 'Z' ? 65 : 97;
    return String.fromCharCode(((c.charCodeAt(0) - base + 13) % 26) + base);
  });
}

function levenshtein(a, b) {
  const m = a.length, n = b.length;
  const dp = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));
  for (let i = 0; i <= m; i++) dp[i][0] = i;
  for (let j = 0; j <= n; j++) dp[0][j] = j;
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      dp[i][j] = a[i - 1] === b[j - 1]
        ? dp[i - 1][j - 1]
        : 1 + Math.min(dp[i - 1][j], dp[i][j - 1], dp[i - 1][j - 1]);
    }
  }
  return dp[m][n];
}

function longestCommonSubsequence(a, b) {
  const m = a.length, n = b.length;
  const dp = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      dp[i][j] = a[i - 1] === b[j - 1]
        ? dp[i - 1][j - 1] + 1
        : Math.max(dp[i - 1][j], dp[i][j - 1]);
    }
  }
  return dp[m][n];
}

function huffmanFreqs(str) {
  const freq = {};
  for (const c of str) freq[c] = (freq[c] || 0) + 1;
  return Object.entries(freq).sort((a, b) => b[1] - a[1]);
}

function kmpSearch(text, pattern) {
  const lps = new Array(pattern.length).fill(0);
  let len = 0, i = 1;
  while (i < pattern.length) {
    if (pattern[i] === pattern[len]) { lps[i++] = ++len; }
    else if (len) { len = lps[len - 1]; }
    else { lps[i++] = 0; }
  }
  const matches = [];
  let ti = 0, pi = 0;
  while (ti < text.length) {
    if (text[ti] === pattern[pi]) { ti++; pi++; }
    if (pi === pattern.length) { matches.push(ti - pi); pi = lps[pi - 1]; }
    else if (ti < text.length && text[ti] !== pattern[pi]) {
      if (pi) pi = lps[pi - 1]; else ti++;
    }
  }
  return matches;
}

const words = ["deterministic", "compilation", "webassembly", "javascript", "quickjs",
  "runtime", "bytecode", "interpreter", "function", "module", "export", "import",
  "memory", "linear", "stack", "heap", "garbage", "collector", "prototype", "closure"];

const rotated = words.map(rot13);
const distances = [];
for (let i = 0; i < words.length; i++) {
  for (let j = i + 1; j < words.length; j++) {
    distances.push(levenshtein(words[i], words[j]));
  }
}

const longText = words.join(" ").repeat(10);
const lcsResult = longestCommonSubsequence(words[0], words[1]);
const freqs = huffmanFreqs(longText);
const searchHits = kmpSearch(longText, "tion");

const randomStrings = [];
for (let i = 0; i < 30; i++) {
  let s = "";
  for (let j = 0; j < 20; j++) {
    s += String.fromCharCode(97 + Math.floor(Math.random() * 26));
  }
  randomStrings.push(s);
}

const result = {
  rotSample: rotated.slice(0, 5),
  distanceSum: distances.reduce((a, b) => a + b, 0),
  lcs: lcsResult,
  topFreq: freqs.slice(0, 3),
  searchCount: searchHits.length,
  randomSample: randomStrings.slice(0, 3),
  timestamp: Date.now(),
};

const output = JSON.stringify(result);
Javy.IO.writeSync(1, new Uint8Array(new TextEncoder().encode(output)));
