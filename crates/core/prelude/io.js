Javy.IO = {
  readSync(fd, data) {
    if (!(data instanceof Uint8Array)) {
      throw Error("Data needs to be an Uint8Array");
    }
    return __javy_io_readSync(fd, data.buffer);
  },
  writeSync(fd, data) {
    if (!(data instanceof Uint8Array)) {
      throw Error("Data needs to be an Uint8Array");
    }
    return __javy_io_writeSync(fd, data.buffer);
  },
};
