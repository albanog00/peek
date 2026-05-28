{
  self,
  lib,
  mkShell,
  stdenv,
}: let
  peekPkgs = self.packages.${stdenv.hostPlatform.system}.peek;
in
  mkShell {
    name = "peek-dev";
    inputsFrom = [peekPkgs];

    nativeBuildInputs = [];

    env = {
      RUSTFLAGS = "-C link-arg=-fuse-ld=lld";
      RUST_BACKTRACE = "1";
      LD_LIBRARY_PATH = "${lib.makeLibraryPath (map (input: input.out) peekPkgs.buildInputs)}:$LD_LIBRARY_PATH";
    };
  }
