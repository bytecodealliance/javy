export async function foo() {
    console.log(await Promise.resolve("inside foo"));
}

(async function () {
    console.log(await Promise.resolve("Top-level"));
})();
