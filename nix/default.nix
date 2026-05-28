{
  lib,
  stdenv,
  craneLib,
  pkg-config,
  makeWrapper,
  # GPU backend
  vulkan-headers,
  vulkan-loader,
  libGL,
  # Window system
  libxkbcommon,
  wayland,
  libX11,
  libXcursor,
  libXi,
  libXrandr,
  # Text
  fontconfig,
  freetype,
  atk,
  gio-sharp,
  glib,
  gtk3,
}: let
  pname = "peek";
  version = "0.1.0";

  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];

  buildInputs =
    [
      fontconfig
      freetype
    ]
    ++ lib.optionals stdenv.isLinux [
      vulkan-loader
      vulkan-headers
      libGL

      libxkbcommon
      wayland

      libX11
      libXcursor
      libXi
      libXrandr

      gio-sharp
      gtk3
      glib
      atk
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
        cargo build --release --frozen --package peek
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
