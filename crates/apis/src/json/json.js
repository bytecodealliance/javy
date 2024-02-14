(function () {
  const __javy_json_parse = globalThis.__javy_json_parse;
  globalThis.Javy.JSON = {
    parseFromStdin(fd, data) {
      return __javy_json_parse();
    },
  };

  Reflect.deleteProperty(globalThis, "__javy_json_parse");
})();
