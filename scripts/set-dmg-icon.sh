#!/bin/bash
# Post-process a Tauri-built DMG:
# 1. Remove the custom volume icon (so macOS uses the default disk icon)
# 2. Add a Retina @2x background image for crisp display on HiDPI screens
#
# Usage: set-dmg-icon.sh <dmg_path> [icon_dir]
#   dmg_path  — path to the .dmg file
#   icon_dir  — directory containing dmg-background.png and dmg-background@2x.png
#               (defaults to src-tauri/icons)

set -e

DMG_PATH="$1"
ICON_DIR="${2:-src-tauri/icons}"

if [[ -z "$DMG_PATH" ]]; then
    echo "Usage: $0 <dmg_path> [icon_dir]"
    exit 1
fi

if [[ ! -f "$DMG_PATH" ]]; then
    echo "Error: DMG not found: $DMG_PATH"
    exit 1
fi

BG_2X="$ICON_DIR/dmg-background@2x.png"
if [[ ! -f "$BG_2X" ]]; then
    echo "Warning: No @2x background found at $BG_2X, skipping Retina background"
    HAS_2X=false
else
    HAS_2X=true
fi

# Create a temporary directory
TMPDIR=$(mktemp -d)
trap "rm -rf '$TMPDIR'" EXIT

# Create a read-write DMG from the compressed one
RW_DMG="$TMPDIR/rw.dmg"
hdiutil convert "$DMG_PATH" -format UDRW -o "$RW_DMG" 2>/dev/null

# Mount the read-write DMG
MOUNT_POINT="$TMPDIR/mount"
mkdir -p "$MOUNT_POINT"
hdiutil attach "$RW_DMG" -mountpoint "$MOUNT_POINT" -nobrowse -quiet

# 1. Remove the custom volume icon (Tauri sets app icon as volume icon by default)
if [ -f "$MOUNT_POINT/.VolumeIcon.icns" ]; then
    rm -f "$MOUNT_POINT/.VolumeIcon.icns"
    echo "Removed .VolumeIcon.icns (will use default macOS disk icon)"
fi

# Clear the custom icon Finder attribute
SetFile -a C "$MOUNT_POINT" 2>/dev/null || true

# 2. Add Retina @2x background for HiDPI displays
if $HAS_2X; then
    BG_DIR="$MOUNT_POINT/.background"
    if [ -d "$BG_DIR" ]; then
        cp "$BG_2X" "$BG_DIR/dmg-background@2x.png"
        echo "Added Retina @2x background for HiDPI displays"
    else
        echo "Warning: .background directory not found in DMG, skipping @2x copy"
    fi
fi

# Unmount
hdiutil detach "$MOUNT_POINT" -quiet

# Convert back to compressed DMG, replacing the original
COMPRESSED_DMG="$TMPDIR/compressed.dmg"
hdiutil convert "$RW_DMG" -format UDZO -imagekey zlib-level=9 -o "$COMPRESSED_DMG" 2>/dev/null

# Replace the original DMG
mv "$COMPRESSED_DMG" "$DMG_PATH"

echo "DMG post-processing complete: $DMG_PATH"
