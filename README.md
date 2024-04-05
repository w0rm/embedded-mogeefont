# embedded-mogeefont

# Developing

To enter the development environment, first [install nix](https://nixos.org/download/#download-nix), enable [nix flakes](https://wiki.nixos.org/wiki/Flakes), then run:

```
nix develop
```

# Specimen

To see the font specimen in the browser run:

```
cargo run --target wasm32-unknown-unknown --manifest-path=specimen/Cargo.toml
```
