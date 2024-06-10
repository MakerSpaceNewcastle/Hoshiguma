{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
      in {
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            # Code formatting tools
            treefmt
            alejandra
            mdl
            rustfmt

            # Common
            rustup
            pkg-config

            # koishi firmware
            avrdude
            ravedude
            pkgsCross.avr.buildPackages.gcc

            # koishi firmware in-simulator tests
            clang
            libelf
            zlib

            # koishi telemetry receiver demo
            systemd
          ];

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };
      }
    );
}
