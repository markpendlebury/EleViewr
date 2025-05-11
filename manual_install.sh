#!/bin/bash

# Exit on any error
set -e

echo "Building EleViewr..."
cargo build --release

echo "Installing EleViewr..."
# Create directories if they don't exist
mkdir -p ~/.local/bin
mkdir -p ~/.local/share/applications

# Copy binary to user's PATH
cp ./target/release/eleviewr ~/.local/bin/EleViewr

# Make it executable
chmod +x ~/.local/bin/EleViewr

# Copy desktop file
cp ./eleviewr.desktop ~/.local/share/applications/

# Update desktop database
update-desktop-database ~/.local/share/applications

# Set as default image viewer
xdg-mime default EleViewr.desktop image/jpeg
xdg-mime default EleViewr.desktop image/png
xdg-mime default EleViewr.desktop image/gif
xdg-mime default EleViewr.desktop image/webp
xdg-mime default EleViewr.desktop image/tiff
xdg-mime default EleViewr.desktop image/bmp

echo "Installation complete! EleViewr has been set as the default image viewer."