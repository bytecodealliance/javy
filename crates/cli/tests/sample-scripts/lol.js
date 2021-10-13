// import * as std from 'std';
import * as os from 'os';
// globalThis.std = std;
globalThis.os = os;

console.log("hello world");


function fetch(url, options) {


  // make_http_request(...)
  // loop {
  //   poll_oneoff()
  //
  return Promise.resolve({ body: "hello", status: 200 });
}

fetch("https://google.ca")
  .then((response) => { console.log(response.body); });

os.setTimeout(() => {
  console.log("will it blend?");
}, 10000);


os.setTimeout(() => {
  console.log("will it blend2?");
}, 20000);
console.log("patate");

let call = (i) => { return i; };

let input = getInput();
setOutput(call(input));
