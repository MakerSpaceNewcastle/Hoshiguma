# Satori firmware

## Requirements

- Nix (with flake support)

## Development

Note that if the flake is reevaluated whilst inside the Distrobox container, you must exit and reenter the Distrobox container.

1. `distrobox create --name hoshiguma-satori --image ghcr.io/dannixon/esp-rs-distrobox:v2` (first run only)
2. `distrobox enter hoshiguma-satori`
3. `espup install` (first run only)
4. `. $HOME/export-esp.sh`
5. `cargo build`
