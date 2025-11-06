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
              llvmPackages_latest.libclang
            ];

            LIBCLANG_PATH = "${pkgs.llvmPackages_latest.libclang.lib}/lib";
          };
        });
  }
