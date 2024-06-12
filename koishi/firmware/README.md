# Firmware

## Dependencies

- Nix (with flake support)
- direnv (or just run `nix shell` before running the commands below)

## Build

```shell
cargo build --release -F telemetry
```

## Flash

```shell
cargo run --release -F telemetry -- -P /dev/ttyXXX
```
