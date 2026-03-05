#!/bin/sh
set -e

REPO="mystico53/gdcli"
INSTALL_DIR="$HOME/.local/bin"

# Detect OS
OS="$(uname -s)"
case "$OS" in
    Linux)  ARCHIVE="gdcli-linux-x86_64.tar.gz" ;;
    Darwin) ARCHIVE="gdcli-macos-universal.tar.gz" ;;
    *)
        echo "Error: unsupported OS: $OS"
        exit 1
        ;;
esac

# Get latest release tag
TAG=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
if [ -z "$TAG" ]; then
    echo "Error: could not fetch latest release tag"
    exit 1
fi

echo "Installing gdcli $TAG for $OS..."

# Download and extract
URL="https://github.com/$REPO/releases/download/$TAG/$ARCHIVE"
mkdir -p "$INSTALL_DIR"
curl -fsSL "$URL" | tar xz -C "$INSTALL_DIR"
chmod +x "$INSTALL_DIR/gdcli"

echo "Installed gdcli to $INSTALL_DIR/gdcli"

# Check PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo ""
        echo "WARNING: $INSTALL_DIR is not in your PATH."
        echo "Add it with:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
        echo "Or add that line to your ~/.bashrc or ~/.zshrc."
        ;;
esac
