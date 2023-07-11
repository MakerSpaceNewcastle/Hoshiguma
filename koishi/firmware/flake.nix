{
  description = "Koishi firmware build/development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    ravedude.url = "github:Rahix/avr-hal?dir=ravedude";
  };

  outputs = { self, nixpkgs, ravedude }:
  let
    allSystems = [
      "x86_64-linux" # 64-bit Intel/AMD Linux
      "aarch64-linux" # 64-bit ARM Linux
      "x86_64-darwin" # 64-bit Intel macOS
      "aarch64-darwin" # 64-bit ARM macOS
    ];

    forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system: f {
      pkgs = import nixpkgs { inherit system; };
      ravedude = [ ravedude.packages."${system}".default ];
    });
  in
  {
    devShells = forAllSystems ({ pkgs, ravedude }: {
      default = pkgs.mkShell {
        packages = (with pkgs; [
          avrdude
          pkgs.pkgsCross.avr.buildPackages.gcc
          rustup
        ])
        ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ libiconv ])
        ++ ravedude;
      };
    });
  };
}
