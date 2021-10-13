// import * as std from 'std';
import * as os from 'os';
// globalThis.std = std;
globalThis.os = os;

console.log("hello world");

os.setTimeout(() => {
  console.log("will it blend?");
}, 1);

let call = (i) => { return i; };

let input = getInput();
setOutput(call(input));
