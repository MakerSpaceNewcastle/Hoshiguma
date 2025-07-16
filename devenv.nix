{pkgs, ...}: {
  packages = with pkgs; [
    # Code formatting tools
    treefmt
    alejandra
    mdl
    rustfmt

    # Rust toolchain
    rustup
    probe-rs

    # Peripheral controller telemetry receiver
    pkg-config
    systemd
  ];
}
