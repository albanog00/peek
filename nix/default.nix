{
  lib,
  stdenv,
  craneLib,
  pkg-config,
  # GPU backend
  vulkan-loader,
  libGL,
  # Window system
  libxkbcommon,
  wayland,
  libX11,
  libXcursor,
  libXi,
  libXrandr,
}: let
  pname = "peek";
  version = "0.1.0";

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs =
    []
    ++ lib.optionals stdenv.isLinux [
      vulkan-loader
      libGL
      libxkbcommon
      wayland
      libX11
      libXcursor
      libXi
      libXrandr
    ];

  commonArgs = {
    inherit
      pname
      version
      nativeBuildInputs
      buildInputs
      ;
    strictDeps = true;
    doCheck = false;

    src = let
      fs = lib.fileset;
      s = ../.;
    in
      fs.toSource {
        root = s;
        fileset = fs.intersection (fs.fromSource (lib.sources.cleanSource s)) (
          fs.unions [
            (s + /crates)

            (s + /Cargo.toml)
            (s + /Cargo.lock)

            (s + /.clippy.toml)
          ]
        );
      };
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
  craneLib.mkCargoDerivation (
    commonArgs
    // {
      inherit cargoArtifacts;

      buildPhaseCargoCommand = ''
        cargo build --release --offline --frozen --package peek
      '';

      meta = {
        description = "A very basic flake";
        homepage = "https://github.com/albanog00/peek";
        license = lib.licenses.mit;
        maintainers = with lib.maintainers; [albanog00];
        platforms = lib.platforms.linux;
        mainProgram = "peek";
      };
    }
  )
