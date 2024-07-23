{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
    nixpkgs-unstable.url = "github:nixos/nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    nixpkgs-unstable,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
        pkgs-unstable = import nixpkgs-unstable {inherit system;};
      in {
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            # Code formatting tools
            treefmt
            alejandra
            mdl
            rustfmt

            # Common
            pkgs-unstable.rustup
            pkg-config

            # koishi firmware
            avrdude
            ravedude
            pkgsCross.avr.buildPackages.gcc

            # koishi firmware in-simulator tests
            clang
            libelf
            zlib

            # satori firmware
            flip-link
            probe-rs

            # telemetry receiver
            systemd
          ];

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };
      }
    );
}
