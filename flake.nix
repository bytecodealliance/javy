{
    inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    inputs.flake-utils.url = "github:numtide/flake-utils";

    inputs.rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };

    outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
      flake-utils.lib.eachDefaultSystem (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          wasiSdk = pkgs.runCommand "wasi-sdk-24" {
                buildInputs = with pkgs; [ gnutar llvmPackages_latest.clang-unwrapped binutils coreutils ];
              } ''
                wasi_tar=${pkgs.fetchurl {
                  url = "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-24/wasi-sdk-24.0-x86_64-linux.tar.gz";
                  sha256 = "sha256-xsOKq1bl3oit9sHrycOujacviOwrZW+wJO2o1BZ6C8U=";
                }}
                ${pkgs.gnutar}/bin/tar xvfz $wasi_tar
                ${pkgs.coreutils}/bin/ln -fs ${pkgs.llvmPackages_latest.clang-unwrapped}/bin/clang wasi-sdk-24.0-x86_64-linux/bin/clang
                ${pkgs.coreutils}/bin/ln -fs ${pkgs.binutils}/bin/ar wasi-sdk-24.0-x86_64-linux/bin/ar
                mv wasi-sdk-24.0-x86_64-linux $out
              '';
        in
        {
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              wasiSdk
              rustToolchain
              wasmtime
              llvmPackages_latest.libclang
            ];
            shellHook = ''
              export WASI_SDK=${wasiSdk}
            '';
            BINDGEN_EXTRA_CLANG_ARGS = builtins.concatStringsSep " " [
              "-isystem ${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"
              "-isystem ${pkgs.glibc.dev}/include"
            ];
            # rquickjs automatically installs the WASI SDK.
            BINDGEN_EXTRA_CLANG_ARGS_wasm32_wasip2 = "";
            LIBCLANG_PATH = "${pkgs.llvmPackages_latest.libclang.lib}/lib";
          };
        });
  }
