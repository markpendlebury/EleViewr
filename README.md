<div align="center">
  <img src="images/logo.png" alt="EleViewr Logo">
</div>


## Features

- Simple, minimalist image viewer specifically designed for Wayland/Hyprland (other platforms coming soon)
- Set as your default image viewer to quickly preview an image
- Use left/right or h/l to navigate through images in the same directory


## Requirements

- Hyprland or other Wayland compositor
- Arch Linux recommended, additional platform support coming soon

## Installation


### Building from source

1. Clone the repository:
   ```
   git clone https://github.com/markpendlebury/EleViewr.git
   cd EleViewr
   ```

2. Manual install
   ```
   ./manual_install.sh
   ```
   This will compile and install the binary to your PATH as well as making it your default image viewer.

## Usage

From a directory containing images: 

```
eleviewr 

```
or to open a single image (you can still browse in the current directory) 

```
eleviewr /path/to/image.jpg
```

### Controls

- Left Arrow or h: Previous image
- Right Arrow or l: Next image
- Escape: Quit

## License

This project is released under the MIT License.