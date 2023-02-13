(async function () {
    function writeOutput(output) {
        const encodedOutput = new TextEncoder().encode(JSON.stringify(output));
        const buffer = new Uint8Array(encodedOutput);
        // Stdout file descriptor
        const fd = 1;
        Javy.IO.writeSync(fd, buffer);
    }

    let promise1 = Promise.resolve("foo");
    let v = await promise1;
    writeOutput(v);
    Promise.resolve("bar").then(writeOutput);
})();
