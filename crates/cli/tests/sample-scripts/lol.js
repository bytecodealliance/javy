// import * as os from "os";

console.log("hello world");

os.setTimeout(() => {
  console.log("will it blend?");
}, 1);

let call = (i) => { return i; };

let input = getInput();
setOutput(call(input));
