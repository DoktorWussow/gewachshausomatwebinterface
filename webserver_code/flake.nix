{
  description = "basic rust flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix"; # Rust toolchain
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:

    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rustToolchain = fenix.packages.${system}.stable.withComponents [
          "rustc"
          "cargo"
          "clippy"
          "rustfmt"
          "rust-src"
          "llvm-tools-preview"
        ];
        withTargets = with fenix.packages.${system};
          combine [
            rustToolchain
            targets.x86_64-pc-windows-gnu.stable.rust-std
          ];
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [

            (pkgs.pkgsCross.mingwW64.stdenv.cc.override ({
              extraBuildCommands = ''
                printf '%s' '-L ${pkgs.pkgsCross.mingwW64.windows.mingw_w64_pthreads}/lib' >>$out/nix-support/cc-ldflags
              '';
            }))
            withTargets
            pkgs.pkg-config
            pkgs.pkgsCross.mingwW64.stdenv
            pkgs.pkgsCross.mingwW64.stdenv.cc
            pkgs.pkgsCross.mingwW64.buildPackages.gcc
            pkgs.pkgsCross.mingwW64.libpthread-stubs.buildInputs
          ];

        };
      });
}
