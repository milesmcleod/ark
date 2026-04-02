#!/bin/sh
# ark installer - downloads the latest release binary for your platform
# Usage: curl -sSL https://raw.githubusercontent.com/milesmcleod/ark/main/install.sh | sh
set -e

REPO="milesmcleod/ark"
INSTALL_DIR="${ARK_INSTALL_DIR:-$HOME/.local/bin}"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  darwin) OS="apple-darwin" ;;
  linux)  OS="unknown-linux-gnu" ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

case "$ARCH" in
  arm64|aarch64) ARCH="aarch64" ;;
  x86_64)        ARCH="x86_64" ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

TARGET="${ARCH}-${OS}"
ARTIFACT="ark-${TARGET}"

# Get latest release tag
echo "Fetching latest release..."
LATEST=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')

if [ -z "$LATEST" ]; then
  echo "Could not determine latest release. Check https://github.com/${REPO}/releases"
  exit 1
fi

URL="https://github.com/${REPO}/releases/download/${LATEST}/${ARTIFACT}"

echo "Installing ark ${LATEST} for ${TARGET}..."
echo "  from: ${URL}"
echo "  to:   ${INSTALL_DIR}/ark"

# Download
mkdir -p "$INSTALL_DIR"
curl -sSL "$URL" -o "${INSTALL_DIR}/ark"
chmod +x "${INSTALL_DIR}/ark"

# Verify
if "${INSTALL_DIR}/ark" --version > /dev/null 2>&1; then
  echo ""
  echo "ark ${LATEST} installed successfully."
  VERSION=$("${INSTALL_DIR}/ark" --version)
  echo "  ${VERSION}"
else
  echo ""
  echo "Binary downloaded but verification failed."
  echo "  Check: ${INSTALL_DIR}/ark"
  exit 1
fi

# Check PATH
case ":$PATH:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo ""
    echo "Note: ${INSTALL_DIR} is not in your PATH."
    echo "Add it with:"
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    echo ""
    echo "Or add to your shell profile (~/.zshrc, ~/.bashrc)."
    ;;
esac
