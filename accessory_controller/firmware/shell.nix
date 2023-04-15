{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    avrdude
    pkgs.pkgsCross.avr.buildPackages.gcc
    rustup
  ];
}
