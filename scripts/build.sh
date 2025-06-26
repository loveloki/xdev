#!/bin/bash
#
# This script automates the cross-compilation process for different architectures
# using `cross`. It cleans up previous builds, compiles for each specified
# target, and places the resulting binaries into a `dist` directory.

# Exit immediately if a command exits with a non-zero status.
set -e

# --- Configuration ---
PROJECT_NAME="xdev"
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
)
DIST_DIR="dist"

# --- Main Script ---
echo "--- Starting build process for $PROJECT_NAME ---"

# Clean previous builds and create the dist directory
echo "--- Cleaning up old builds and creating '$DIST_DIR' directory ---"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Loop through each target and build
for target in "${TARGETS[@]}"; do
    echo ""
    echo "--- Building for target: $target ---"
    cross build --target "$target" --release

    echo "--- Copying binary for $target ---"
    SOURCE_PATH="target/$target/release/$PROJECT_NAME"
    DEST_PATH="$DIST_DIR/${PROJECT_NAME}-${target}"

    cp "$SOURCE_PATH" "$DEST_PATH"
    echo "Binary for $target copied to $DEST_PATH"
done

echo ""
echo "--- All builds completed successfully! ---"
echo "Binaries are located in the '$DIST_DIR' directory:"
ls -l "$DIST_DIR"

echo ""
echo "--- Compressing binaries with UPX ---"
upx --best "$DIST_DIR"/*

echo ""
echo "--- Final compressed binaries ---"
ls -l "$DIST_DIR" 
