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

        cargo-watch = rustPlatform.buildRustPackage rec {
          pname = "cargo-watch";
          version = "8.5.2";

          src = pkgs.fetchCrate {
            inherit pname version;
            sha256 = "sha256-39KR4TzQpJ+V8odnmNIPudsKc4XvFr1I2CJx/mZhaxU=";
          };
          cargoSha256 = "sha256-skUG1B6TCFEXeQSRwA6vWjXmNifk5bTR4+JESw7CZMo=";
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Foundation
            pkgs.darwin.apple_sdk.frameworks.Cocoa
          ];
          NIX_LDFLAGS = pkgs.lib.optionals (pkgs.stdenv.isDarwin && pkgs.stdenv.isx86_64) [ "-framework" "AppKit" ];

          # `test with_cargo` tries to call cargo-watch as a cargo subcommand
          # (calling cargo-watch with command `cargo watch`)
          preCheck = ''
            export PATH="$(pwd)/target/${pkgs.stdenv.hostPlatform.rust.rustcTarget}/release:$PATH"
          '';
        };
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = [
            cargo-watch
            rust-bin
            wasm-server-runner
          ];
        };
      }
    );
}
