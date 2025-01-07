export async function foo() {
    console.error(await Promise.resolve("inside foo"));
}

(async function () {
    console.error(await Promise.resolve("Top-level"));
})();
