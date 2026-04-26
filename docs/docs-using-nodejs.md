# Embedding in Node.js, Deno, Bun Application
This example demonstrates how to run Javy in a Node.js (v20+), Deno, and Bun host application.

## Warning
This example does NOT show how to run a Node.js, Deno, and Bun application in Javy. This is
useful for when you want to run untrusted user generated code in a sandbox. This
code is meant to be an example not production-ready code.

It's also important to note that the WASI implementation in Node.js is currently
considered [experimental]. In Deno's current implementation of `node:wasi` all [exports are non-functional stubs].
In Bun's current implementation of `node:wasi` [is just a quick hack to get WASI working]. 
`wasi.js`, used here for Deno and Bun support is a modified version of [`deno-wasi`] [adapted to support `node`, `deno`, and  `bun`], and is intended to be JavaScript runtime agnostic.

[experimental]: https://nodejs.org/api/wasi.html#webassembly-system-interface-wasi
[exports are non-functional stubs]: https://docs.deno.com/api/node/wasi/
[is just a quick hack to get WASI working]: https://github.com/oven-sh/bun/blob/main/src/js/node/wasi.ts#L7
[`deno-wasi`]: https://github.com/caspervonb/deno-wasi
[adapted to support `node`, `deno`, and  `bun`]: https://github.com/guest271314/deno-wasi/tree/runtime-agnostic-nodejs-api

## Summary
This example shows how to use a dynamically linked Javy compiled Wasm module. We
use std in/out/error to communicate with the embedded JavaScript. See [this blog
post](https://k33g.hashnode.dev/wasi-communication-between-nodejs-and-wasm-modules-another-way-with-stdin-and-stdout)
for details.


### Steps

1. Emit the Javy plugin
```shell
javy emit-plugin -o plugin.wasm
```
2. Compile the `embedded.js` with Javy using dynamic linking:
```shell
javy build -C dynamic -C plugin=plugin.wasm -o embedded.wasm embedded.js
```
3. Run `host.mjs`
```shell
node --no-warnings=ExperimentalWarning host.js
```


`embedded.js`
```javascript
// Read input from stdin
const input = readInput();
// Call the function with the input
const result = foo(input);
// Write the result to stdout
writeOutput(result);

// The main function.
function foo(input) {
  if (input && typeof input === "object" && typeof input.n === "number") {
    return { n: input.n + 1 };
  }
  return { n: 0 };
}

// Read input from stdin
function readInput() {
  const chunkSize = 1024;
  const inputChunks = [];
  let totalBytes = 0;

  // Read all the available bytes
  while (1) {
    const buffer = new Uint8Array(chunkSize);
    // Stdin file descriptor
    const fd = 0;
    const bytesRead = Javy.IO.readSync(fd, buffer);

    totalBytes += bytesRead;
    if (bytesRead === 0) {
      break;
    }
    inputChunks.push(buffer.subarray(0, bytesRead));
  }

  // Assemble input into a single Uint8Array
  const { finalBuffer } = inputChunks.reduce(
    (context, chunk) => {
      context.finalBuffer.set(chunk, context.bufferOffset);
      context.bufferOffset += chunk.length;
      return context;
    },
    { bufferOffset: 0, finalBuffer: new Uint8Array(totalBytes) },
  );

  const maybeJson = new TextDecoder().decode(finalBuffer);
  try {
    return JSON.parse(maybeJson);
  } catch {
    return;
  }
}

// Write output to stdout
function writeOutput(output) {
  const encodedOutput = new TextEncoder().encode(JSON.stringify(output));
  const buffer = new Uint8Array(encodedOutput);
  // Stdout file descriptor
  const fd = 1;
  Javy.IO.writeSync(fd, buffer);
}
```


`host.js`
```javascript
import { readFile } from "node:fs/promises";
import {
  closeSync,
  openSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { randomUUID } from "node:crypto";
import { WASI } from "node:wasi";

try {
  const [embeddedModule, pluginModule] = await Promise.all([
    compileModule("./embedded.wasm"),
    compileModule("./plugin.wasm"),
  ]);
  const result = await runJavy(pluginModule, embeddedModule, { n: 100 });
  console.log("Success!", JSON.stringify(result, null, 2));
} catch (e) {
  console.log(e);
}

async function compileModule(wasmPath) {
  const bytes = await readFile(new URL(wasmPath, import.meta.url));
  return WebAssembly.compile(bytes);
}

async function runJavy(pluginModule, embeddedModule, input) {
  const uniqueId = randomUUID();
  // Use stdin/stdout/stderr to communicate with Wasm instance
  // See https://k33g.hashnode.dev/wasi-communication-between-nodejs-and-wasm-modules-another-way-with-stdin-and-stdout
  const workDir = tmpdir();
  const stdinFilePath = join(workDir, `stdin.wasm.${uniqueId}.txt`);
  const stdoutFilePath = join(workDir, `stdout.wasm.${uniqueId}.txt`);
  const stderrFilePath = join(workDir, `stderr.wasm.${uniqueId}.txt`);

  // 👋 send data to the Wasm instance
  writeFileSync(stdinFilePath, JSON.stringify(input), { encoding: "utf8" });

  const [stdinFileFd, stdoutFileFd, stderrFileFd] = [
    openSync(stdinFilePath, "r"),
    openSync(stdoutFilePath, "a"),
    openSync(stderrFilePath, "a"),
  ];

  const wasiOptions = {
    version: "preview1",
    returnOnExit: true,
    args: [],
    env: {},
    stdin: stdinFileFd,
    stdout: stdoutFileFd,
    stderr: stderrFileFd,
  };

  try {
    // Deno's "node:wasi" is a stub, not implemented
    // https://docs.deno.com/api/node/wasi/
    // https://github.com/denoland/deno/issues/21025
    let wasi = null;

    try {
      wasi = navigator.userAgent.startsWith("Node.js")
        ? new WASI(wasiOptions)
        : wasi;
    } catch (e) {
      // Deno
      // https://docs.deno.com/api/node/wasi/
      // All exports are non-functional stubs.
      // Error: Context is currently not supported at new Context (node:wasi:6:11)
      console.log(e);
    } finally {
      wasi ??= new (await import("./wasi.js")).default(wasiOptions);
    }

    const pluginInstance = await WebAssembly.instantiate(
      pluginModule,
      { wasi_snapshot_preview1: wasi?.wasiImport || wasi?.exports },
    ).catch(console.log);

    const instance = await WebAssembly.instantiate(embeddedModule, {
      "javy-default-plugin-v3": pluginInstance.exports,
    }).catch(console.log);
    wasi.memory = pluginInstance.exports.memory;
    // Javy plugin is a WASI reactor see https://github.com/WebAssembly/WASI/blob/main/legacy/application-abi.md?plain=1
    // Bun's "node:wasi" module doesn't have an `initialize` method, hangs here
    // Bun's documentation says the method is implemented
    // https://bun.com/reference/node/wasi
    wasi.initialize ??= wasi?.start;
    wasi?.initialize?.(pluginInstance);
    instance.exports._start();

    const [out, err] = [
      readOutput(stdoutFilePath),
      readOutput(stderrFilePath),
    ];
    if (err) {
      throw new Error(err);
    }

    return out;
  } catch (e) {
    if (e instanceof WebAssembly.RuntimeError) {
      const errorMessage = await readOutput(stderrFilePath);
      if (errorMessage) {
        throw new Error(errorMessage);
      }
    }
    throw e;
  } finally {
    closeSync(stdinFileFd);
    closeSync(stdoutFileFd);
    closeSync(stderrFileFd);
    rmSync(stdinFilePath);
    rmSync(stdoutFilePath);
    rmSync(stderrFilePath);
  }
}

function readOutput(filePath) {
  const str = readFileSync(filePath, "utf8").trim();
  try {
    return JSON.parse(str);
  } catch {
    return str;
  }
}
```
`wasi.js`
```javascript
// Modified deno-wasi implementation for Deno, Bun, Node.js
// https://github.com/caspervonb/deno-wasi
// https://github.com/guest271314/deno-wasi/tree/runtime-agnostic-nodejs-api
// deno-fmt-ignore-file
// deno-lint-ignore-file
// This code was bundled using `deno bundle` and it's not recommended to edit it manually
import * as process from "node:process";
import fs from "node:fs";
import * as crypto from "node:crypto";
// https://github.com/nodejs/node/issues/11568#issuecomment-282765300
process.stdout?._handle?.setBlocking?.(true);
process.stdin?._handle?.setBlocking?.(true);

function assertPath(path) {
  if (typeof path !== "string") {
    throw new TypeError(`Path must be a string. Received ${JSON.stringify(path)}`);
  }
}
const CHAR_FORWARD_SLASH = 47;

function isPathSeparator(code) {
  return code === 47 || code === 92;
}

function isWindowsDeviceRoot(code) {
  return code >= 97 && code <= 122 || code >= 65 && code <= 90;
}

function normalizeString(path, allowAboveRoot, separator, isPathSeparator) {
  let res = "";
  let lastSegmentLength = 0;
  let lastSlash = -1;
  let dots = 0;
  let code;
  for (let i = 0; i <= path.length; ++i) {
    if (i < path.length) code = path.charCodeAt(i);
    else if (isPathSeparator(code)) break;
    else code = CHAR_FORWARD_SLASH;
    if (isPathSeparator(code)) {
      if (lastSlash === i - 1 || dots === 1) {} else if (lastSlash !== i - 1 && dots === 2) {
        if (res.length < 2 || lastSegmentLength !== 2 || res.charCodeAt(res.length - 1) !== 46 || res.charCodeAt(res.length - 2) !== 46) {
          if (res.length > 2) {
            const lastSlashIndex = res.lastIndexOf(separator);
            if (lastSlashIndex === -1) {
              res = "";
              lastSegmentLength = 0;
            } else {
              res = res.slice(0, lastSlashIndex);
              lastSegmentLength = res.length - 1 - res.lastIndexOf(separator);
            }
            lastSlash = i;
            dots = 0;
            continue;
          } else if (res.length === 2 || res.length === 1) {
            res = "";
            lastSegmentLength = 0;
            lastSlash = i;
            dots = 0;
            continue;
          }
        }
        if (allowAboveRoot) {
          if (res.length > 0) res += `${separator}..`;
          else res = "..";
          lastSegmentLength = 2;
        }
      } else {
        if (res.length > 0) res += separator + path.slice(lastSlash + 1, i);
        else res = path.slice(lastSlash + 1, i);
        lastSegmentLength = i - lastSlash - 1;
      }
      lastSlash = i;
      dots = 0;
    } else if (code === 46 && dots !== -1) {
      ++dots;
    } else {
      dots = -1;
    }
  }
  return res;
}

function resolve(...pathSegments) {
  let resolvedDevice = "";
  let resolvedTail = "";
  let resolvedAbsolute = false;
  for (let i = pathSegments.length - 1; i >= -1; i--) {
    let path;
    // const { Deno: Deno1 } = globalThis;
    if (i >= 0) {
      path = pathSegments[i];
    } else if (!resolvedDevice) {
      if (typeof process.cwd !== "function") {
        throw new TypeError("Resolved a drive-letter-less path without a CWD.");
      }
      path = process.cwd();
    } else {
      if (typeof process?.env?.get !== "function" || typeof process?.cwd !== "function") {
        throw new TypeError("Resolved a relative path without a CWD.");
      }
      path = process.cwd();
      if (path === undefined || path.slice(0, 3).toLowerCase() !== `${resolvedDevice.toLowerCase()}\\`) {
        path = `${resolvedDevice}\\`;
      }
    }
    assertPath(path);
    const len = path.length;
    if (len === 0) continue;
    let rootEnd = 0;
    let device = "";
    let isAbsolute = false;
    const code = path.charCodeAt(0);
    if (len > 1) {
      if (isPathSeparator(code)) {
        isAbsolute = true;
        if (isPathSeparator(path.charCodeAt(1))) {
          let j = 2;
          let last = j;
          for (; j < len; ++j) {
            if (isPathSeparator(path.charCodeAt(j))) break;
          }
          if (j < len && j !== last) {
            const firstPart = path.slice(last, j);
            last = j;
            for (; j < len; ++j) {
              if (!isPathSeparator(path.charCodeAt(j))) break;
            }
            if (j < len && j !== last) {
              last = j;
              for (; j < len; ++j) {
                if (isPathSeparator(path.charCodeAt(j))) break;
              }
              if (j === len) {
                device = `\\\\${firstPart}\\${path.slice(last)}`;
                rootEnd = j;
              } else if (j !== last) {
                device = `\\\\${firstPart}\\${path.slice(last, j)}`;
                rootEnd = j;
              }
            }
          }
        } else {
          rootEnd = 1;
        }
      } else if (isWindowsDeviceRoot(code)) {
        if (path.charCodeAt(1) === 58) {
          device = path.slice(0, 2);
          rootEnd = 2;
          if (len > 2) {
            if (isPathSeparator(path.charCodeAt(2))) {
              isAbsolute = true;
              rootEnd = 3;
            }
          }
        }
      }
    } else if (isPathSeparator(code)) {
      rootEnd = 1;
      isAbsolute = true;
    }
    if (device.length > 0 && resolvedDevice.length > 0 && device.toLowerCase() !== resolvedDevice.toLowerCase()) {
      continue;
    }
    if (resolvedDevice.length === 0 && device.length > 0) {
      resolvedDevice = device;
    }
    if (!resolvedAbsolute) {
      resolvedTail = `${path.slice(rootEnd)}\\${resolvedTail}`;
      resolvedAbsolute = isAbsolute;
    }
    if (resolvedAbsolute && resolvedDevice.length > 0) break;
  }
  resolvedTail = normalizeString(resolvedTail, !resolvedAbsolute, "\\", isPathSeparator);
  return resolvedDevice + (resolvedAbsolute ? "\\" : "") + resolvedTail || ".";
}

function isPosixPathSeparator(code) {
  return code === 47;
}

function resolve1(...pathSegments) {
  let resolvedPath = "";
  let resolvedAbsolute = false;
  for (let i = pathSegments.length - 1; i >= -1 && !resolvedAbsolute; i--) {
    let path;
    if (i >= 0) path = pathSegments[i];
    else {
      // const { Deno: Deno1 } = globalThis;
      if (typeof process?.cwd !== "function") {
        throw new TypeError("Resolved a relative path without a CWD.");
      }
      path = process.cwd();
    }
    assertPath(path);
    if (path.length === 0) {
      continue;
    }
    resolvedPath = `${path}/${resolvedPath}`;
    resolvedAbsolute = isPosixPathSeparator(path.charCodeAt(0));
  }
  resolvedPath = normalizeString(resolvedPath, !resolvedAbsolute, "/", isPosixPathSeparator);
  if (resolvedAbsolute) {
    if (resolvedPath.length > 0) return `/${resolvedPath}`;
    else return "/";
  } else if (resolvedPath.length > 0) return resolvedPath;
  else return ".";
}
const osType = (() => {
  // const { Deno: Deno1 } = globalThis;
  if (typeof process?.build?.os === "string") {
    return process.build.os;
  }
  const {
    navigator
  } = globalThis;
  if (navigator?.appVersion?.includes?.("Win")) {
    return "windows";
  }
  return "linux";
})();
const isWindows = osType === "windows";

function resolve2(...pathSegments) {
  return isWindows ? resolve(...pathSegments) : resolve1(...pathSegments);
}
const CLOCKID_REALTIME = 0;
const CLOCKID_MONOTONIC = 1;
const CLOCKID_PROCESS_CPUTIME_ID = 2;
const CLOCKID_THREAD_CPUTIME_ID = 3;
const ERRNO_SUCCESS = 0;
const ERRNO_BADF = 8;
const ERRNO_INVAL = 28;
const ERRNO_NOSYS = 52;
const ERRNO_NOTDIR = 54;
const RIGHTS_FD_DATASYNC = 0x0000000000000001n;
const RIGHTS_FD_READ = 0x0000000000000002n;
const RIGHTS_FD_WRITE = 0x0000000000000040n;
const RIGHTS_FD_ALLOCATE = 0x0000000000000100n;
const RIGHTS_FD_READDIR = 0x0000000000004000n;
const RIGHTS_FD_FILESTAT_SET_SIZE = 0x0000000000400000n;
const FILETYPE_UNKNOWN = 0;
const FILETYPE_CHARACTER_DEVICE = 2;
const FILETYPE_DIRECTORY = 3;
const FILETYPE_REGULAR_FILE = 4;
const FILETYPE_SYMBOLIC_LINK = 7;
const FDFLAGS_APPEND = 0x0001;
const FDFLAGS_DSYNC = 0x0002;
const FDFLAGS_NONBLOCK = 0x0004;
const FDFLAGS_RSYNC = 0x0008;
const FDFLAGS_SYNC = 0x0010;
const FSTFLAGS_ATIM_NOW = 0x0002;
const FSTFLAGS_MTIM_NOW = 0x0008;
const OFLAGS_CREAT = 0x0001;
const OFLAGS_DIRECTORY = 0x0002;
const OFLAGS_EXCL = 0x0004;
const OFLAGS_TRUNC = 0x0008;
const PREOPENTYPE_DIR = 0;
const clock_res_realtime = function() {
  return BigInt(1e6);
};
const clock_res_monotonic = function() {
  return BigInt(1e3);
};
const clock_res_process = clock_res_monotonic;
const clock_res_thread = clock_res_monotonic;
const clock_time_realtime = function() {
  return BigInt(Date.now()) * BigInt(1e6);
};
const clock_time_monotonic = function() {
  const t = performance.now();
  const s = Math.trunc(t);
  const ms = Math.floor((t - s) * 1e3);
  return BigInt(s) * BigInt(1e9) + BigInt(ms) * BigInt(1e6);
};
const clock_time_process = clock_time_monotonic;
const clock_time_thread = clock_time_monotonic;

function errno(err) {
  switch (err.name) {
    case "NotFound":
      return 44;
    case "PermissionDenied":
      return 2;
    case "ConnectionRefused":
      return 14;
    case "ConnectionReset":
      return 15;
    case "ConnectionAborted":
      return 13;
    case "NotConnected":
      return 53;
    case "AddrInUse":
      return 3;
    case "AddrNotAvailable":
      return 4;
    case "BrokenPipe":
      return 64;
    case "InvalidData":
      return 28;
    case "TimedOut":
      return 73;
    case "Interrupted":
      return 27;
    case "BadResource":
      return 8;
    case "Busy":
      return 10;
    default:
      return 28;
  }
}
class Module {
  args;
  env;
  memory;
  fds;
  exports;
  constructor(options) {
    this.args = options?.args ? options.args : [];
    this.env = options?.env ? options.env : {};
    this.memory = options?.memory;
    this.fds = [{
        type: FILETYPE_CHARACTER_DEVICE,
        handle: options?.stdin ? { fd: options.stdin } : process.stdin
      },
      {
        type: FILETYPE_CHARACTER_DEVICE,
        handle: options?.stdout ? { fd: options.stdout } : process.stdout
      },
      {
        type: FILETYPE_CHARACTER_DEVICE,
        handle: options?.stderr ? { fd: options.stderr } : process.stderr
      }
    ];
    if (options?.preopens) {
      for (const [vpath, path] of Object.entries(options.preopens)) {
        const info = fs.statSync(path);
        if (!info.isDirectory) {
          throw new TypeError(`${path} is not a directory`);
        }
        const entry = {
          type: 3,
          path,
          vpath
        };
        this.fds.push(entry);
      }
    }
    this.exports = {
      args_get: (argv_ptr, argv_buf_ptr) => {
        const args = this.args;
        const text = new TextEncoder();
        const heap = new Uint8Array(this.memory.buffer);
        const view = new DataView(this.memory.buffer);
        for (let arg of args) {
          view.setUint32(argv_ptr, argv_buf_ptr, true);
          argv_ptr += 4;
          const data = text.encode(`${arg}\0`);
          heap.set(data, argv_buf_ptr);
          argv_buf_ptr += data.length;
        }
        return ERRNO_SUCCESS;
      },
      args_sizes_get: (argc_out, argv_buf_size_out) => {
        const args = this.args;
        const text = new TextEncoder();
        const view = new DataView(this.memory.buffer);
        view.setUint32(argc_out, args.length, true);
        view.setUint32(argv_buf_size_out, args.reduce(function(acc, arg) {
          return acc + text.encode(`${arg}\0`).length;
        }, 0), true);
        return ERRNO_SUCCESS;
      },
      environ_get: (environ_ptr, environ_buf_ptr) => {
        const entries = Object.entries(this.env);
        const text = new TextEncoder();
        const heap = new Uint8Array(this.memory.buffer);
        const view = new DataView(this.memory.buffer);
        for (let [key, value] of entries) {
          view.setUint32(environ_ptr, environ_buf_ptr, true);
          environ_ptr += 4;
          const data = text.encode(`${key}=${value}\0`);
          heap.set(data, environ_buf_ptr);
          environ_buf_ptr += data.length;
        }
        return ERRNO_SUCCESS;
      },
      environ_sizes_get: (environc_out, environ_buf_size_out) => {
        const entries = Object.entries(this.env);
        const text = new TextEncoder();
        const view = new DataView(this.memory.buffer);
        view.setUint32(environc_out, entries.length, true);
        view.setUint32(environ_buf_size_out, entries.reduce(function(acc, [key, value]) {
          return acc + text.encode(`${key}=${value}\0`).length;
        }, 0), true);
        return ERRNO_SUCCESS;
      },
      clock_res_get: (id, resolution_out) => {
        const view = new DataView(this.memory.buffer);
        switch (id) {
          case CLOCKID_REALTIME:
            view.setBigUint64(resolution_out, clock_res_realtime(), true);
            break;
          case CLOCKID_MONOTONIC:
            view.setBigUint64(resolution_out, clock_res_monotonic(), true);
            break;
          case CLOCKID_PROCESS_CPUTIME_ID:
            view.setBigUint64(resolution_out, clock_res_process(), true);
            break;
          case CLOCKID_THREAD_CPUTIME_ID:
            view.setBigUint64(resolution_out, clock_res_thread(), true);
            break;
          default:
            return ERRNO_INVAL;
        }
        return ERRNO_SUCCESS;
      },
      clock_time_get: (id, precision, time_out) => {
        const view = new DataView(this.memory.buffer);
        switch (id) {
          case CLOCKID_REALTIME:
            view.setBigUint64(time_out, clock_time_realtime(), true);
            break;
          case CLOCKID_MONOTONIC:
            view.setBigUint64(time_out, clock_time_monotonic(), true);
            break;
          case CLOCKID_PROCESS_CPUTIME_ID:
            view.setBigUint64(time_out, clock_time_process(), true);
            break;
          case CLOCKID_THREAD_CPUTIME_ID:
            view.setBigUint64(time_out, clock_time_thread(), true);
            break;
          default:
            return ERRNO_INVAL;
        }
        return ERRNO_SUCCESS;
      },
      fd_advise: (fd, offset, len, advice) => {
        return ERRNO_NOSYS;
      },
      fd_allocate: (fd, offset, len) => {
        return ERRNO_NOSYS;
      },
      fd_close: (fd) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        entry.handle.close();
        delete this.fds[fd];
        return ERRNO_SUCCESS;
      },
      fd_datasync: (fd) => {
        return ERRNO_NOSYS;
      },
      fd_fdstat_get: (fd, stat_out) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        const view = new DataView(this.memory.buffer);
        view.setUint8(stat_out, entry.type);
        view.setUint16(stat_out + 4, 0, true);
        view.setBigUint64(stat_out + 8, 0n, true);
        view.setBigUint64(stat_out + 16, 0n, true);
        return ERRNO_SUCCESS;
      },
      fd_fdstat_set_flags: (fd, flags) => {
        return ERRNO_NOSYS;
      },
      fd_fdstat_set_rights: (fd, fs_rights_base, fs_rights_inheriting) => {
        return ERRNO_NOSYS;
      },
      fd_filestat_get: (fd, buf_out) => {
        return ERRNO_NOSYS;
      },
      fd_filestat_set_size: (fd, size) => {
        return ERRNO_NOSYS;
      },
      fd_filestat_set_times: (fd, atim, mtim, fst_flags) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        if (!entry.path) {
          return ERRNO_INVAL;
        }
        if ((fst_flags & FSTFLAGS_ATIM_NOW) == FSTFLAGS_ATIM_NOW) {
          atim = BigInt(Date.now() * 1e6);
        }
        if ((fst_flags & FSTFLAGS_MTIM_NOW) == FSTFLAGS_MTIM_NOW) {
          mtim = BigInt(Date.now() * 1e6);
        }
        try {
          fs.utimeSync(entry.path, Number(atim), Number(mtim));
        } catch (err) {
          return errno(err);
        }
        return ERRNO_SUCCESS;
      },
      fd_pread: (fd, iovs_ptr, iovs_len, offset, nread_out) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        const seek = entry.handle.seekSync(0, 1);
        const view = new DataView(this.memory.buffer);
        let nread = 0;
        for (let i = 0; i < iovs_len; i++) {
          const data_ptr = view.getUint32(iovs_ptr, true);
          iovs_ptr += 4;
          const data_len = view.getUint32(iovs_ptr, true);
          iovs_ptr += 4;
          const data = new Uint8Array(this.memory.buffer, data_ptr, data_len);
          nread += entry.handle.readSync(data);
        }
        entry.handle.seekSync(seek, 0);
        view.setUint32(nread_out, nread, true);
        return ERRNO_SUCCESS;
      },
      fd_prestat_get: (fd, buf_out) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        if (!entry.vpath) {
          return ERRNO_BADF;
        }
        const view = new DataView(this.memory.buffer);
        view.setUint8(buf_out, PREOPENTYPE_DIR);
        view.setUint32(buf_out + 4, new TextEncoder().encode(entry.vpath).byteLength, true);
        return ERRNO_SUCCESS;
      },
      fd_prestat_dir_name: (fd, path_ptr, path_len) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        if (!entry.vpath) {
          return ERRNO_BADF;
        }
        const data = new Uint8Array(this.memory.buffer, path_ptr, path_len);
        data.set(new TextEncoder().encode(entry.vpath));
        return ERRNO_SUCCESS;
      },
      fd_pwrite: (fd, iovs_ptr, iovs_len, offset, nwritten_out) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        const seek = entry.handle.seekSync(0, 1);
        const view = new DataView(this.memory.buffer);
        let nwritten = 0;
        for (let i = 0; i < iovs_len; i++) {
          const data_ptr = view.getUint32(iovs_ptr, true);
          iovs_ptr += 4;
          const data_len = view.getUint32(iovs_ptr, true);
          iovs_ptr += 4;
          const data = new Uint8Array(this.memory.buffer, data_ptr, data_len);
          nwritten += entry.handle.writeSync(data);
        }
        entry.handle.seekSync(seek, 0);
        view.setUint32(nwritten_out, nwritten, true);
        return ERRNO_SUCCESS;
      },
      fd_read: (fd, iovs_ptr, iovs_len, nread_out) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        //console.log(entry);
        const view = new DataView(this.memory.buffer);
        let nread = 0;
        for (let i = 0; i < iovs_len; i++) {
          const data_ptr = view.getUint32(iovs_ptr, true);
          iovs_ptr += 4;
          const data_len = view.getUint32(iovs_ptr, true);
          iovs_ptr += 4;
          const data = new Uint8Array(this.memory.buffer, data_ptr, data_len);
          nread += fs.readSync(entry.handle.fd, data); // entry.handle.readSync(data);
        }
        view.setUint32(nread_out, nread, true);
        return ERRNO_SUCCESS;
      },
      fd_readdir: (fd, buf_ptr, buf_len, cookie, bufused_out) => {
        return ERRNO_NOSYS;
      },
      fd_renumber: (fd, to) => {
        if (!this.fds[fd]) {
          return ERRNO_BADF;
        }
        if (!this.fds[to]) {
          return ERRNO_BADF;
        }
        this.fds[to].handle.close();
        this.fds[to] = this.fds[fd];
        delete this.fds[fd];
        return ERRNO_SUCCESS;
      },
      fd_seek: (fd, offset, whence, newoffset_out) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        const view = new DataView(this.memory.buffer);
        try {
          const newoffset = entry.handle.seekSync(Number(offset), whence);
          view.setBigUint64(newoffset_out, BigInt(newoffset), true);
        } catch (err) {
          return ERRNO_INVAL;
        }
        return ERRNO_SUCCESS;
      },
      fd_sync: (fd) => {
        return ERRNO_NOSYS;
      },
      fd_tell: (fd, offset_out) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        const view = new DataView(this.memory.buffer);
        try {
          const offset = entry.handle.seekSync(0, 1);
          view.setBigUint64(offset_out, offset, true);
        } catch (err) {
          return ERRNO_INVAL;
        }
        return ERRNO_NOSYS;
      },
      fd_write: (fd, iovs_ptr, iovs_len, nwritten_out) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        const view = new DataView(this.memory.buffer);
        let nwritten = 0;
        for (let i = 0; i < iovs_len; i++) {
          const data_ptr = view.getUint32(iovs_ptr, true);
          iovs_ptr += 4;
          const data_len = view.getUint32(iovs_ptr, true);
          iovs_ptr += 4;
          nwritten += fs.writeSync(entry.handle.fd || entry.handle, new Uint8Array(this.memory.buffer, data_ptr, data_len));
        }
        view.setUint32(nwritten_out, nwritten, true);
        return ERRNO_SUCCESS;
      },
      path_create_directory: (fd, path_ptr, path_len) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        if (!entry.path) {
          return ERRNO_INVAL;
        }
        const text = new TextDecoder();
        const data = new Uint8Array(this.memory.buffer, path_ptr, path_len);
        const path = resolve2(entry.path, text.decode(data));
        try {
          fs.mkdirSync(path);
        } catch (err) {
          return errno(err);
        }
        return ERRNO_SUCCESS;
      },
      path_filestat_get: (fd, flags, path_ptr, path_len, buf_out) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        if (!entry.path) {
          return ERRNO_INVAL;
        }
        const text = new TextDecoder();
        const data = new Uint8Array(this.memory.buffer, path_ptr, path_len);
        const path = resolve2(entry.path, text.decode(data));
        const view = new DataView(this.memory.buffer);
        try {
          const info = fs.statSync(path);
          view.setBigUint64(buf_out, BigInt(info.dev ? info.dev : 0), true);
          buf_out += 8;
          view.setBigUint64(buf_out, BigInt(info.ino ? info.ino : 0), true);
          buf_out += 8;
          switch (true) {
            case info.isFile:
              view.setUint8(buf_out, FILETYPE_REGULAR_FILE);
              buf_out += 4;
              break;
            case info.isDirectory:
              view.setUint8(buf_out, FILETYPE_DIRECTORY);
              buf_out += 4;
              break;
            case info.isSymlink:
              view.setUint8(buf_out, FILETYPE_SYMBOLIC_LINK);
              buf_out += 4;
              break;
            default:
              view.setUint8(buf_out, FILETYPE_UNKNOWN);
              buf_out += 4;
              break;
          }
          view.setUint32(buf_out, Number(info.nlink), true);
          buf_out += 4;
          view.setBigUint64(buf_out, BigInt(info.size), true);
          buf_out += 8;
          view.setBigUint64(buf_out, BigInt(info.atime ? info.atime.getTime() * 1e6 : 0), true);
          buf_out += 8;
          view.setBigUint64(buf_out, BigInt(info.mtime ? info.mtime.getTime() * 1e6 : 0), true);
          buf_out += 8;
          view.setBigUint64(buf_out, BigInt(info.birthtime ? info.birthtime.getTime() * 1e6 : 0), true);
          buf_out += 8;
        } catch (err) {
          return errno(err);
        }
        return ERRNO_SUCCESS;
      },
      path_filestat_set_times: (fd, flags, path_ptr, path_len, atim, mtim, fst_flags) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        if (!entry.path) {
          return ERRNO_INVAL;
        }
        const text = new TextDecoder();
        const data = new Uint8Array(this.memory.buffer, path_ptr, path_len);
        const path = resolve2(entry.path, text.decode(data));
        if ((fst_flags & FSTFLAGS_ATIM_NOW) == FSTFLAGS_ATIM_NOW) {
          atim = BigInt(Date.now()) * BigInt(1e6);
        }
        if ((fst_flags & FSTFLAGS_MTIM_NOW) == FSTFLAGS_MTIM_NOW) {
          mtim = BigInt(Date.now()) * BigInt(1e6);
        }
        try {
          fs.utimesSync(path, Number(atim), Number(mtim));
        } catch (err) {
          return errno(err);
        }
        return ERRNO_SUCCESS;
      },
      path_link: (old_fd, old_flags, old_path_ptr, old_path_len, new_fd, new_path_ptr, new_path_len) => {
        const old_entry = this.fds[old_fd];
        const new_entry = this.fds[new_fd];
        if (!old_entry || !new_entry) {
          return ERRNO_BADF;
        }
        if (!old_entry.path || !new_entry.path) {
          return ERRNO_INVAL;
        }
        const text = new TextDecoder();
        const old_data = new Uint8Array(this.memory.buffer, old_path_ptr, old_path_len);
        const old_path = resolve2(old_entry.path, text.decode(old_data));
        const new_data = new Uint8Array(this.memory.buffer, new_path_ptr, new_path_len);
        const new_path = resolve2(new_entry.path, text.decode(new_data));
        try {
          fs.linkSync(old_path, new_path);
        } catch (err) {
          return errno(err);
        }
        return ERRNO_SUCCESS;
      },
      path_open: (fd, dirflags, path_ptr, path_len, oflags, fs_rights_base, fs_rights_inherting, fdflags, opened_fd_out) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        if (!entry.path) {
          return ERRNO_INVAL;
        }
        const text = new TextDecoder();
        const data = new Uint8Array(this.memory.buffer, path_ptr, path_len);
        const path = resolve2(entry.path, text.decode(data));
        const options = {
          read: false,
          write: false,
          append: false,
          truncate: false,
          create: false,
          createNew: false
        };
        if ((oflags & OFLAGS_CREAT) !== 0) {
          options.create = true;
          options.write = true;
        }
        if ((oflags & OFLAGS_DIRECTORY) !== 0) {}
        if ((oflags & OFLAGS_EXCL) !== 0) {
          options.createNew = true;
        }
        if ((oflags & OFLAGS_TRUNC) !== 0) {
          options.truncate = true;
          options.write = true;
        }
        if ((BigInt(fs_rights_base) & BigInt(RIGHTS_FD_READ | RIGHTS_FD_READDIR)) != 0n) {
          options.read = true;
        }
        if ((BigInt(fs_rights_base) & BigInt(RIGHTS_FD_DATASYNC | RIGHTS_FD_WRITE | RIGHTS_FD_ALLOCATE | RIGHTS_FD_FILESTAT_SET_SIZE)) != 0n) {
          options.write = true;
        }
        if ((fdflags & FDFLAGS_APPEND) != 0) {
          options.append = true;
        }
        if ((fdflags & FDFLAGS_DSYNC) != 0) {}
        if ((fdflags & FDFLAGS_NONBLOCK) != 0) {}
        if ((fdflags & FDFLAGS_RSYNC) != 0) {}
        if ((fdflags & FDFLAGS_SYNC) != 0) {}
        if (!options.read && !options.write && !options.truncate) {
          options.read = true;
        }
        try {
          const handle = fs.openSync(path, options);
          const opened_fd = this.fds.push({
            handle,
            path
          }) - 1;
          const view = new DataView(this.memory.buffer);
          view.setUint32(opened_fd_out, opened_fd, true);
        } catch (err) {
          return errno(err);
        }
        return ERRNO_SUCCESS;
      },
      path_readlink: (fd, path_ptr, path_len, buf_ptr, buf_len, bufused_out) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        if (!entry.path) {
          return ERRNO_INVAL;
        }
        const view = new DataView(this.memory.buffer);
        const heap = new Uint8Array(this.memory.buffer);
        const data = new Uint8Array(this.memory.buffer, path_ptr, path_len);
        const path = resolve2(entry.path, new TextDecoder().decode(data));
        try {
          const link = fs.readlinkSync(path);
          const data = new TextEncoder().encode(link);
          heap.set(new Uint8Array(data, 0, buf_len), buf_ptr);
          const bufused = Math.min(data.byteLength, buf_len);
          view.setUint32(bufused_out, bufused, true);
        } catch (err) {
          return errno(err);
        }
        return ERRNO_SUCCESS;
      },
      path_remove_directory: (fd, path_ptr, path_len) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        if (!entry.path) {
          return ERRNO_INVAL;
        }
        const text = new TextDecoder();
        const data = new Uint8Array(this.memory.buffer, path_ptr, path_len);
        const path = resolve2(entry.path, text.decode(data));
        try {
          if (!fs.statSync(path).isDirectory) {
            return ERRNO_NOTDIR;
          }
          fs.removeSync(path);
        } catch (err) {
          return errno(err);
        }
        return ERRNO_SUCCESS;
      },
      path_rename: (fd, old_path_ptr, old_path_len, new_fd, new_path_ptr, new_path_len) => {
        const old_entry = this.fds[fd];
        const new_entry = this.fds[new_fd];
        if (!old_entry || !new_entry) {
          return ERRNO_BADF;
        }
        if (!old_entry.path || !new_entry.path) {
          return ERRNO_INVAL;
        }
        const text = new TextDecoder();
        const old_data = new Uint8Array(this.memory.buffer, old_path_ptr, old_path_len);
        const old_path = resolve2(old_entry.path, text.decode(old_data));
        const new_data = new Uint8Array(this.memory.buffer, new_path_ptr, new_path_len);
        const new_path = resolve2(new_entry.path, text.decode(new_data));
        try {
          fs.renameSync(old_path, new_path);
        } catch (err) {
          return errno(err);
        }
        return ERRNO_SUCCESS;
      },
      path_symlink: (old_path_ptr, old_path_len, fd, new_path_ptr, new_path_len) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        if (!entry.path) {
          return ERRNO_INVAL;
        }
        const text = new TextDecoder();
        const old_data = new Uint8Array(this.memory.buffer, old_path_ptr, old_path_len);
        const old_path = text.decode(old_data);
        const new_data = new Uint8Array(this.memory.buffer, new_path_ptr, new_path_len);
        const new_path = resolve2(entry.path, text.decode(new_data));
        try {
          fs.symlinkSync(old_path, new_path);
        } catch (err) {
          return errno(err);
        }
        return ERRNO_SUCCESS;
      },
      path_unlink_file: (fd, path_ptr, path_len) => {
        const entry = this.fds[fd];
        if (!entry) {
          return ERRNO_BADF;
        }
        if (!entry.path) {
          return ERRNO_INVAL;
        }
        const text = new TextDecoder();
        const data = new Uint8Array(this.memory.buffer, path_ptr, path_len);
        const path = resolve2(entry.path, text.decode(data));
        try {
          fs.removeSync(path);
        } catch (err) {
          return errno(err);
        }
        return ERRNO_SUCCESS;
      },
      poll_oneoff: (in_ptr, out_ptr, nsubscriptions, nevents_out) => {
        return ERRNO_NOSYS;
      },
      proc_exit: (rval) => {
        process.exit(rval);
      },
      proc_raise: (sig) => {
        return ERRNO_NOSYS;
      },
      sched_yield: () => {
        return ERRNO_SUCCESS;
      },
      random_get: (buf_ptr, buf_len) => {
        const buffer = new Uint8Array(this.memory.buffer, buf_ptr, buf_len);
        crypto.getRandomValues(buffer);
        return ERRNO_SUCCESS;
      },
      sock_recv: (fd, ri_data_ptr, ri_data_len, ri_flags, ro_datalen_out, ro_flags_out) => {
        return ERRNO_NOSYS;
      },
      sock_send: (fd, si_data_ptr, si_data_len, si_flags, so_datalen_out) => {
        return ERRNO_NOSYS;
      },
      sock_shutdown: (fd, how) => {
        return ERRNO_NOSYS;
      }
    };
  }
}
export {
  Module as Module
};
export {
  Module as
  default
};
```
