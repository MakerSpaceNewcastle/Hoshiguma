# Firmware

## Prerequisities

- rustup
- Nix
- ravedude (`cargo install ravedude`)

## Build

```shell
nix-shell --run "cargo build"
```

## Flash

```shell
nix-shell --run "cargo run -- -P /dev/ttyXXX"
```
