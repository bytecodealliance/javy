// META: script=/WebCryptoAPI/sign-verify/hmac.js

function getTestVectors() {
  var plaintext = new Uint8Array([95, 77, 186, 79, 50, 12, 12, 232, 118, 114, 90, 252, 229, 251, 210, 91, 248, 62, 90, 113, 37, 160, 140, 175, 231, 60, 62, 186, 196, 33, 119, 157, 249, 213, 93, 24, 12, 58, 233, 148, 38, 69, 225, 216, 47, 238, 140, 157, 41, 75, 60, 177, 160, 138, 153, 49, 32, 27, 60, 14, 129, 252, 71, 202, 207, 131, 21, 162, 175, 102, 50, 65, 19, 195, 182, 98, 48, 195, 70, 8, 196, 244, 89, 54, 52, 206, 2, 178, 103, 54, 34, 119, 240, 168, 64, 202, 116, 188, 61, 26, 98, 54, 149, 44, 94, 215, 170, 248, 168, 254, 203, 221, 250, 117, 132, 230, 151, 140, 234, 93, 42, 91, 159, 183, 241, 180, 140, 139, 11, 229, 138, 48, 82, 2, 117, 77, 131, 118, 16, 115, 116, 121, 60, 240, 38, 170, 238, 83, 0, 114, 125, 131, 108, 215, 30, 113, 179, 69, 221, 178, 228, 68, 70, 255, 197, 185, 1, 99, 84, 19, 137, 13, 145, 14, 163, 128, 152, 74, 144, 25, 16, 49, 50, 63, 22, 219, 204, 157, 107, 225, 104, 184, 72, 133, 56, 76, 160, 62, 18, 96, 10, 193, 194, 72, 2, 138, 243, 114, 108, 201, 52, 99, 136, 46, 168, 192, 42, 171]);

  var raw = {
      "SHA-256": new Uint8Array([229, 136, 236, 8, 17, 70, 61, 118, 114, 65, 223, 16, 116, 180, 122, 228, 7, 27, 81, 242, 206, 54, 83, 123, 166, 156, 205, 195, 253, 194, 183, 168]),
  };

  var signatures = {
      "SHA-256": new Uint8Array([133, 164, 12, 234, 46, 7, 140, 40, 39, 163, 149, 63, 251, 102, 194, 123, 41, 26, 71, 43, 13, 112, 160, 0, 11, 69, 216, 35, 128, 62, 235, 84]),
  };

  // Each test vector has the following fields:
  //     name - a unique name for this vector
  //     keyBuffer - an arrayBuffer with the key data
  //     key - a CryptoKey object for the keyBuffer. INITIALLY null! You must fill this in first to use it!
  //     hashName - the hash function to sign with
  //     plaintext - the text to encrypt
  //     signature - the expected signature
  var vectors = [];
  Object.keys(raw).forEach(function(hashName) {
      vectors.push({
          name: "HMAC with " + hashName,
          hash: hashName,
          keyBuffer: raw[hashName],
          key: null,
          plaintext: plaintext,
          signature: signatures[hashName]
      });
  });

  return vectors;
}


run_test();
