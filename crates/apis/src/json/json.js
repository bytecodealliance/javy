(function () {
  const __javy_io_readSync = globalThis.__javy_io_readSync;
  const __javy_io_writeSync = globalThis.__javy_io_writeSync;
  globalThis.Javy.JSON = {
    parseFromStdin(fd, data) {
      return __javy_json_parse();
    },
  };

  Reflect.deleteProperty(globalThis, "__javy_io_readSync");
  Reflect.deleteProperty(globalThis, "__javy_io_writeSync");
})();
