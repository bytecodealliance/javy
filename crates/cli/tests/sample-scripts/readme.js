function foo(input) {
    return { foo: input.n + 1, newBar: input.bar + "!" };
}

// read stdin into a list of buffers
const inputBufferSize = 1024;
let bytesRead = 0;
let totalBytesRead = 0;
const inputBuffers = [];
do {
    const inputBuffer = new Uint8Array(inputBufferSize);
    const stdinFileDescriptor = 0;
    bytesRead = Javy.IO.readSync(stdinFileDescriptor, inputBuffer);
    totalBytesRead += bytesRead;
    const startOfArrayIndex = 0;
    inputBuffers.push(inputBuffer.subarray(startOfArrayIndex, bytesRead));
} while (bytesRead === inputBufferSize);

// assemble input buffer from list of buffers
const completeInputBuffer = new Uint8Array(totalBytesRead);
let offsetIntoInputBuffer = 0;
inputBuffers.forEach(buffer => {
    completeInputBuffer.set(buffer, offsetIntoInputBuffer);
    offsetIntoInputBuffer += buffer.length;
});

// decode and deserialize input bytes
const input = JSON.parse(new TextDecoder().decode(completeInputBuffer));

const functionOutput = foo(input);

// serialize and encode output into output bytes
const output = new TextEncoder().encode(JSON.stringify(functionOutput));

// write output bytes to stdout
const outputBuffer = new Uint8Array(output);
const stdoutFileDescriptor = 1;
Javy.IO.writeSync(stdoutFileDescriptor, outputBuffer);
