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
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [ rustToolchain pkgs.pkg-config ];

        };
      });
}
