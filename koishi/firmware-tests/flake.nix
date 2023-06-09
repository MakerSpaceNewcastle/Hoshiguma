{
  description = "Koishi firmware simulator test build/development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }:
  let
    allSystems = [
      "x86_64-linux" # 64-bit Intel/AMD Linux
      "aarch64-linux" # 64-bit ARM Linux
      "x86_64-darwin" # 64-bit Intel macOS
      "aarch64-darwin" # 64-bit ARM macOS
    ];

    forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system: f {
      pkgs = import nixpkgs { inherit system; };
    });
  in
  {
    devShells = forAllSystems ({ pkgs }: {
      default = pkgs.mkShell {
        packages = (with pkgs; [
          clang
          libelf
          pkg-config
          rustup
          simavr
          zlib
        ]) ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ libiconv ]);

        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      };
    });
  };
}
