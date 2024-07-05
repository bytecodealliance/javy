const promiseA = new Promise((resolve, reject) => {
  resolve(777);
});
// At this point, "promiseA" is already settled.
async f () => {
  await promiseA.then((val) => console.log("asynchronous logging has val:", val));
}
f();
console.log("immediate logging");
