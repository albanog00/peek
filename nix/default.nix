{
  lib,
  stdenv,
  craneLib,
  pkg-config,
  # GPU backend
  vulkan-headers,
  vulkan-loader,
  libGL,
  mesa,
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
  fd,
  wrapGAppsHook3,
}: let
  pname = "peek";
  version = "0.1.0";

  nativeBuildInputs =
    [
      pkg-config
    ]
    ++ lib.optionals stdenv.isLinux [
      wrapGAppsHook3
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
      mesa

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

  baseArgs = {
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

  rawCargoVendorDir = craneLib.vendorCargoDeps baseArgs;
  patchedCargoVendorDir = stdenv.mkDerivation {
    name = "${pname}-cargo-vendor";
    buildCommand = ''
      mkdir -p "$out"
      cp -aL ${rawCargoVendorDir}/. "$out"/
      chmod -R u+rwX "$out"

      # gpui-component expects the upstream repo layout: `../assets/assets/icons`.
      # Cargo/Crane vendors the assets crate as `gpui-component-assets-*`, so add
      # the sibling `assets` name the proc macro expects.

      root="$(${lib.getExe fd} "^gpui-component-[0-9].*" "$out" -t d | head -n1)"
      assets="$(${lib.getExe fd} "^gpui-component-assets-[0-9].*" "$out" -t d | head -n1)"
      root="$(dirname "$root")"

      ln -s "$assets" "$root/assets"

      substituteInPlace "$out/config.toml" \
      --replace-fail '${rawCargoVendorDir}' "$out"

      test -L "$root/assets"
      test -d "$root/assets/assets/icons"
    '';
  };

  commonArgs =
    baseArgs
    // {
      cargoVendorDir = patchedCargoVendorDir;
    };

  cargoArtifacts = craneLib.buildDepsOnly (commonArgs
    // {
      cargoExtraArgs = "--locked";
    });
in
  craneLib.mkCargoDerivation (
    commonArgs
    // {
      inherit cargoArtifacts;

      buildPhaseCargoCommand = ''
        cargo build --release --frozen --package peek
      '';

      installPhaseCommand = ''
        mkdir -p $out/bin
        cp target/release/${pname} $out/bin/${pname}
      '';

      preFixup = lib.optionalString stdenv.isLinux ''
        gappsWrapperArgs+=(
          --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath (
          map (input: input.out) [
            vulkan-loader
            libGL

            libxkbcommon
            wayland

            libX11
            libXcursor
            libXi
            libXrandr
          ]
        )}
        )
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
