{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    ravedude.url = "github:Rahix/avr-hal?dir=ravedude";
  };

  outputs = { self, nixpkgs, flake-utils, ravedude }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        ravedude' = ravedude.packages."${system}".default;
      in rec {
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            # Common
            rustup
            pkg-config

            # koishi firmware
            avrdude
            ravedude'
            pkgs.pkgsCross.avr.buildPackages.gcc

            # koishi firmware in-simulator tests
            clang
            libelf
            simavr
            zlib

            # koishi telemetry receiver demo
            systemd
          ];

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };
      }
    );
}
