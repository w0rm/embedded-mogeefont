{
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs;
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, rust-overlay, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit overlays system; };

        latestStableRust = pkgs.rust-bin.stable.latest.default;

        rust-bin = latestStableRust.override {
          extensions = [ "rust-src" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = latestStableRust;
          rustc = latestStableRust;
        };

        wasm-server-runner = rustPlatform.buildRustPackage rec {
          pname = "wasm-server-runner";
          version = "0.6.3";

          src = pkgs.fetchCrate {
            inherit pname version;
            sha256 = "sha256-4NuvNvUHZ7n0QP42J9tuf1wqBe9f/R6iJAGeuno9qtg=";
          };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux [ ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ ];
          cargoSha256 = "sha256-L9SK+CILDlmYwXIAESWaqnLQyZQ4oC29av1T6zE6qJo=";
        };
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = [
            pkgs.cargo-watch
            rust-bin
            wasm-server-runner
          ];
        };
      }
    );
}
