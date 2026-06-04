# Peek

Peek is a small, lightweight image viewer built with Rust and GPUI.
It provides a simple desktop window for opening, dropping, panning, and zooming local image files.

## Features

- **Open from path**: Launch Peek with an image path from the command line.
- **Drag and drop**: Drop an image into the window to load or replace it.
- **Common formats**: Supports `jpg`, `jpeg`, `png`, `webp`, `gif`, `bmp`, `tif`, `tiff`, and `ico`.

## Installation

### Nix / NixOS

Add it to your flake inputs:

```nix
{
  inputs = {
    peek.url = "github:albanog00/peek";
  };
}
```

Then install it as a package:

```nix
{ pkgs, peek, ... } : {
  home.packages = [
    peek.packages.${pkgs.stdenv.hostPlatform.system}.default
  ]
}
```

Or run directly from the repository:

```bash
nix run github:albanog00/peek -- /path/to/image.png
```

### Build from Source

#### Dependencies

Peek uses Rust `1.95.0`, defined in `rust-toolchain.toml`.
You will also need a graphics API (OpenGL/Vulkan/Metal/DirectX).

With nix enter the development environment:

```bash
nix develop
```

Or if you use Direnv, allow the included `.envrc`:

```bash
direnv allow
```

#### Build

```bash
cargo build --release --locked --package peek
```

#### Run

Open an image directly:

```bash
cargo run --release --locked --package peek -- /path/to/image.png
```

Start with an empty window:

```bash
cargo run --release --locked --package peek
```

Then drop an image into the window.

## Controls

- **Left-click drag**: Pan the image.
- **Mouse wheel**: Zoom around the cursor.
- **Pinch gesture**: Zoom around the gesture position.
- **Double-click**: Toggle fit-to-window or zoomed view.

## Notes

Even though it is untested, Peek should work on macOS and Windows.
