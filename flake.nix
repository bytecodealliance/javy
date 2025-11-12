{
    inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    inputs.flake-utils.url = "github:numtide/flake-utils";

    outputs = { nixpkgs, flake-utils, ... }:
      flake-utils.lib.eachDefaultSystem (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              wasmtime
              llvmPackages_latest.libclang
            ];

            BINDGEN_EXTRA_CLANG_ARGS = builtins.concatStringsSep " " [
              "-isystem ${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"
              "-isystem ${pkgs.glibc.dev}/include"
            ];
            # rquickjs automatically installs the WASI SDK.
            BINDGEN_EXTRA_CLANG_ARGS_wasm32_wasip1 = "";
            BINDGEN_EXTRA_CLANG_ARGS_wasm32_wasip2 = "";
            LIBCLANG_PATH = "${pkgs.llvmPackages_latest.libclang.lib}/lib";
          };
        });
  }
