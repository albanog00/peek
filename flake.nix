{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    crane,
  }: let
    forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
    pkgsForEach = system: nixpkgs.legacyPackages.${system}.extend rust-overlay.overlays.default;
    mkCraneLib = pkgs:
      (crane.mkLib pkgs).overrideToolchain (
        p: (p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
      );
  in {
    devShells = forAllSystems (
      system: let
        pkgs = pkgsForEach system;
      in {
        default = pkgs.callPackage ./nix/shell.nix {inherit self;};
      }
    );

    packages = forAllSystems (
      system: let
        pkgs = pkgsForEach system;
        craneLib = mkCraneLib pkgs;
      in {
        peek = pkgs.callPackage ./nix/default.nix {inherit craneLib;};
        default = self.packages.${system}.peek;
      }
    );

    checks = forAllSystems (system: {
      default = self.packages.${system}.default;
    });
  };
}
