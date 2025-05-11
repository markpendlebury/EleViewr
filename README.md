# EleViewr

A lightweight image viewer for Hyprland (Wayland) on Arch Linux.

## Features

- Simple, minimalist image viewer specifically designed for Wayland/Hyprland
- Navigate between images in a directory using arrow keys or vim bindings (h/l)
- Fast loading and rendering using GPU acceleration (wgpu)
- Automatic window sizing based on image dimensions
- Supports common image formats (JPG, PNG, GIF, WEBP, TIFF, BMP)

## Requirements

- Rust (installation instructions: https://www.rust-lang.org/tools/install)
- Hyprland or other Wayland compositor
- Arch Linux recommended (though may work on other Linux distros)

## Installation

### Building from source

1. Clone the repository:
   ```
   git clone https://github.com/markpendlebury/EleViewr.git
   cd EleViewr
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. The binary will be available at `target/release/EleViewr`

4. Optionally, copy the binary to a directory in your PATH:
   ```
   sudo cp target/release/EleViewr /usr/local/bin/
   ```

## Usage

```
EleViewr 

```
or 

```
EleViewr /path/to/image.jpg
```


### Controls

- Left Arrow or h: Previous image
- Right Arrow or l: Next image
- Escape: Quit

## Setting as Default Image Viewer

You can configure EleViewr as your default image viewer by creating a desktop entry file:

1. Create a file at `~/.local/share/applications/EleViewr.desktop`:

```
[Desktop Entry]
Name=EleViewr
Comment=A lightweight image viewer for Hyprland
Exec=EleViewr %f
Terminal=false
Type=Application
Categories=Graphics;Viewer;
MimeType=image/jpeg;image/png;image/gif;image/webp;image/tiff;image/bmp;
```

2. Update the desktop database:
```
update-desktop-database ~/.local/share/applications
```

3. Set EleViewr as default using your file manager's preferences or with `xdg-mime`:
```
xdg-mime default EleViewr.desktop image/jpeg
xdg-mime default EleViewr.desktop image/png
```

## License

This project is released under the MIT License.