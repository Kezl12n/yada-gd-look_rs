{
  description = "Ultra-fast Mandarin & Double-Array Trie lookup for GoldenDict-ng";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "yada-gd-look_rs";
        version = "0.1.0";
        src = ./.;

   
        cargoLock = {
          lockFile = ./Cargo.lock;
        };
      };
    };
}
