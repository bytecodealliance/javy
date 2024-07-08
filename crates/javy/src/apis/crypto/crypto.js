(function() {
  const __javy_cryptoSubtleSign = globalThis.__javy_cryptoSubtleSign;

  const crypto = {
    subtle: {}
  };


  crypto.subtle.sign = function(obj, key, msg) {
    return new Promise((resolve, _) => {
      resolve(__javy_cryptoSubtleSign(obj, key, msg));
    });
  }

  globalThis.crypto = crypto;
  Reflect.deleteProperty(globalThis, "__javy_cryptoSubtleSign");
})();
