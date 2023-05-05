import { writeFileSync } from "../../src/fs/index.ts";

// use a stub version of writeSync that returns 0 bytes
Javy.IO.writeSync = function () {
    return 0;
}

try {
    writeFileSync(1, new Uint8Array([42]));
    throw Error("Expected writeFileSync to throw");
} catch (e) {
    if (e.message !== "Could not write all contents in buffer to file descriptor") {
        throw Error(e);
    }
}
