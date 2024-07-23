# Firmware

## Dependencies

- Nix (with flake support)
- direnv (or just run `nix shell` before running the commands below)

## Build

```shell
cargo build --release
```

## Flash

```shell
cargo run --release -- -P /dev/ttyXXX
```
