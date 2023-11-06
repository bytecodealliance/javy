(function () {
  const __javy_io_readSync = globalThis.__javy_io_readSync;
  const __javy_io_writeSync = globalThis.__javy_io_writeSync;
  const __javy_io_readFile = globalThis.__javy_io_readFile;
  globalThis.Javy.IO = {
    readSync(fd, data) {
      if (!(data instanceof Uint8Array)) {
        throw TypeError("Data needs to be an Uint8Array");
      }
      return __javy_io_readSync(
        fd,
        data.buffer,
        data.byteOffset,
        data.byteLength
      );
    },
    writeSync(fd, data) {
      if (!(data instanceof Uint8Array)) {
        throw TypeError("Data needs to be an Uint8Array");
      }
      return __javy_io_writeSync(
        fd,
        data.buffer,
        data.byteOffset,
        data.byteLength
      );
    },
    readFile(path) {
      return __javy_io_readFile(
        path
      )
    }
  };

  Reflect.deleteProperty(globalThis, "__javy_io_readSync");
  Reflect.deleteProperty(globalThis, "__javy_io_writeSync");
  Reflect.deleteProperty(globalThis, "__javy_io_readFile");
})();
