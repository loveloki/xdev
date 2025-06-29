#!/usr/bin/env bash

set -e
set -u

# --- Configuration ---
readonly GITHUB_REPO="loveloki/xdev"
readonly BINARY_NAME="xdev"
readonly DEFAULT_INSTALL_DIR="$HOME/.local/bin"
readonly GITHUB_API_URL="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
readonly CURL_TIMEOUT=30
readonly CURL_MAX_TIME=300

# --- Global variables ---
ARCH=""
ASSET_NAME=""
VERSION=""
CHECKSUM=""
BINARY_PATH=""
EXECUTABLE_PATH=""
API_RESPONSE=""
DOWNLOAD_URL=""
OSTYPE=""
CPUTYPE=""
ACTUAL_CHECKSUM=""
ASSET_JSON=""
SUDO=""
APT_GET_UPDATED=false

setup_sudo() {
  if [ "$(id -u)" -ne 0 ]; then
    if ! check_cmd sudo; then
      err "❌ This script requires root privileges to install dependencies, but sudo is not found. Please run as root or install sudo."
    fi
    SUDO="sudo"
  fi
}

# --- Main execution ---
main() {
  # This script accepts no arguments and will ignore any provided.

  setup_sudo
  check_system_requirements
  detect_architecture
  fetch_release_info
  download_install_and_verify
  finalize_installation
  show_success_message

  return 0
}

# --- Core workflow functions ---

check_system_requirements() {
  # Check for necessary commands
  say "🔧 Checking system requirements..."
  need_cmd uname
  need_cmd chmod
  need_cmd mkdir
  need_cmd curl
  need_cmd grep
  need_cmd jq
  
  # 根据操作系统检查对应的校验工具
  local os_type="$(uname -s)"
  if [ "$os_type" = "Darwin" ]; then
    need_cmd shasum
  else
    need_cmd sha256sum
  fi
}

detect_architecture() {
  get_architecture || return 1
  assert_nz "$ARCH" "arch"
  ASSET_NAME="xdev-${ARCH}"
  say "✅ Detected architecture: ${ARCH}"
}

fetch_release_info() {
  say "🔍 Fetching latest release information..."

  # Fetch API response directly into variable
  API_RESPONSE=$(curl --fail --location --silent --show-error \
    --connect-timeout "$CURL_TIMEOUT" \
    --max-time "$CURL_MAX_TIME" \
    --user-agent "xdev-installer/1.0" \
    "$GITHUB_API_URL") || err "❌ Failed to fetch release information from GitHub API."

  # Parse release info using jq
  VERSION=$(jq -r '.tag_name' <<< "$API_RESPONSE")
  ASSET_JSON=$(jq --arg asset_name "$ASSET_NAME" '.assets[] | select(.name == $asset_name)' <<< "$API_RESPONSE")

  if [ -z "$VERSION" ] || [ "$VERSION" == "null" ]; then
    err "❌ Could not parse version from GitHub API response"
  fi

  if [ -z "$ASSET_JSON" ] || [ "$ASSET_JSON" == "null" ]; then
    err "❌ Could not find asset for ${ASSET_NAME} from GitHub API response."
  fi

  DOWNLOAD_URL=$(jq -r '.browser_download_url' <<< "$ASSET_JSON")
  CHECKSUM=$(jq -r '.digest | split(":")[1]' <<< "$ASSET_JSON")

  assert_nz "$VERSION" "version"
  say "✅ Latest version: ${VERSION}"
}

download_install_and_verify() {
  # Prepare installation directory
  say "📁 Preparing installation directory: $DEFAULT_INSTALL_DIR"
  if [ ! -d "$DEFAULT_INSTALL_DIR" ]; then
    ensure mkdir -p "$DEFAULT_INSTALL_DIR"
  fi

  # Construct download URL and target path
  BINARY_PATH="$DEFAULT_INSTALL_DIR/$BINARY_NAME"

  say "⏳ Downloading xdev ${VERSION} for ${ARCH}..."

  if ! curl --fail --location --progress-bar \
    --connect-timeout "$CURL_TIMEOUT" \
    --max-time "$CURL_MAX_TIME" \
    --user-agent "xdev-installer/1.0" \
    "$DOWNLOAD_URL" \
    --output "$BINARY_PATH"; then
    # Clean up partial download on failure
    [ -f "$BINARY_PATH" ] && rm -f "$BINARY_PATH"
    err "❌ Failed to download binary from ${DOWNLOAD_URL}"
  fi

  # Verify checksum
  if [ -z "${CHECKSUM:-}" ]; then
    rm -f "$BINARY_PATH"
    err "❌ Could not find checksum for asset ${ASSET_NAME} from GitHub API."
  fi

  say "🔒 Verifying checksum..."
  if ! verify_checksum; then
    rm -f "$BINARY_PATH"
    err "❌ Checksum verification failed"
  fi
  say "✅ Checksum verification passed"
}

finalize_installation() {
  # Set executable permissions
  say "📦 Finalizing installation..."
  ensure chmod +x "$BINARY_PATH"

  # Verify installation
  if ! "$BINARY_PATH" version >/dev/null 2>&1; then
    rm -f "$BINARY_PATH"
    err "❌ Installation verification failed. The downloaded binary is not working correctly."
  fi

  EXECUTABLE_PATH="$BINARY_PATH"
}

show_success_message() {
  say ""
  say "🎉 xdev ${VERSION} installed successfully!"
  say "   Location: ${EXECUTABLE_PATH}"

  # Check if the directory is already in PATH
  if echo "$PATH" | grep -q "$DEFAULT_INSTALL_DIR"; then
    say "✅ $DEFAULT_INSTALL_DIR is already in your PATH - you can use xdev immediately!"
  else
    local os_type="$(uname -s)"
    say "💡 To use xdev, you need to add $DEFAULT_INSTALL_DIR to your PATH."
    
    if [ "$os_type" = "Darwin" ]; then
      # macOS 说明
      say "   For Bash: echo 'export PATH=\"$DEFAULT_INSTALL_DIR:\$PATH\"' >> ~/.bash_profile"
      say "   For Zsh:  echo 'export PATH=\"$DEFAULT_INSTALL_DIR:\$PATH\"' >> ~/.zshrc"
      say "   Then restart your terminal or run: source ~/.bash_profile (or ~/.zshrc)"
    else
      # Linux 说明
      say "   You can add it by running: echo 'export PATH=\"$DEFAULT_INSTALL_DIR:\$PATH\"' >> ~/.bashrc"
      say "   Then restart your terminal or run: source ~/.bashrc"
    fi
  fi
}

# --- Helper functions ---

get_architecture() {
  OSTYPE="$(uname -s)"
  CPUTYPE="$(uname -m)"

  case "$OSTYPE" in
    Linux)
      OSTYPE="unknown-linux-gnu"
      ;;
    Darwin)
      OSTYPE="apple-darwin"
      ;;
    *)
      err "❌ Unsupported OS: ${OSTYPE}. Supported: Linux, macOS (Darwin)"
      ;;
  esac

  case "$CPUTYPE" in
    x86_64 | amd64)
      CPUTYPE="x86_64"
      ;;
    aarch64 | arm64)
      CPUTYPE="aarch64"
      ;;
    *)
      err "❌ Unsupported architecture: ${CPUTYPE}. Supported: x86_64, aarch64"
      ;;
  esac

  ARCH="${CPUTYPE}-${OSTYPE}"
}

verify_checksum() {
  # 根据操作系统选择合适的校验工具
  local os_type="$(uname -s)"
  
  if [ "$os_type" = "Darwin" ]; then
    # macOS 使用 shasum
    if command -v shasum >/dev/null 2>&1; then
      ACTUAL_CHECKSUM=$(shasum -a 256 "$BINARY_PATH" | cut -d ' ' -f 1)
    else
      say "❌ 'shasum' command not found. Cannot verify checksum on macOS." >&2
      return 1
    fi
  else
    # Linux 使用 sha256sum
    if command -v sha256sum >/dev/null 2>&1; then
      ACTUAL_CHECKSUM=$(sha256sum "$BINARY_PATH" | cut -d ' ' -f 1)
    else
      say "❌ 'sha256sum' command not found. Cannot verify checksum on Linux." >&2
      return 1
    fi
  fi

  if [ "$ACTUAL_CHECKSUM" != "$CHECKSUM" ]; then
    say "❌ Checksum verification failed! Expected: $CHECKSUM, Got: $ACTUAL_CHECKSUM" >&2
    return 1
  fi

  return 0
}

say() {
  printf '%s\n' "$1"
}

err() {
  say "$1" >&2
  exit 1
}

need_cmd() {
  local cmd="$1"
  if check_cmd "$cmd"; then
    return 0 # Command exists, do nothing.
  fi

  # If command does not exist, check if it's one we can auto-install.
  case "$cmd" in
  curl | jq)
    say "🔍 Command '$cmd' not found. Attempting to install..."

    local pkg_manager=""
    local os_type="$(uname -s)"
    
    if [ "$os_type" = "Darwin" ]; then
      # macOS 使用 Homebrew
      if check_cmd brew; then
        pkg_manager="brew"
      else
        err "❌ Homebrew not found on macOS. Please install Homebrew first: /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
      fi
    else
      # Linux 使用各种包管理器
      if check_cmd apt-get; then
        pkg_manager="apt-get"
      elif check_cmd yum; then
        pkg_manager="yum"
      elif check_cmd dnf; then
        pkg_manager="dnf"
      elif check_cmd pacman; then
        pkg_manager="pacman"
      elif check_cmd apk; then
        pkg_manager="apk"
      else
        err "❌ Could not find a supported package manager (apt-get, yum, dnf, pacman, apk). Please install '$cmd' manually."
      fi
    fi

    say "📦 Using $pkg_manager to install $cmd..."

    case "$pkg_manager" in
    brew) brew install "$cmd" ;;
    apt-get)
      if ! $APT_GET_UPDATED; then
        $SUDO apt-get update -y
        APT_GET_UPDATED=true
      fi
      $SUDO apt-get install -y "$cmd"
      ;;
    yum) $SUDO yum install -y "$cmd" ;;
    dnf) $SUDO dnf install -y "$cmd" ;;
    pacman) $SUDO pacman -S --noconfirm "$cmd" ;;
    apk) $SUDO apk add "$cmd" ;;
    esac

    if ! check_cmd "$cmd"; then
      err "❌ Failed to install '$cmd' automatically. Please try installing it manually."
    fi
    say "✅ Successfully installed $cmd."
    ;;
  *)
    # For any other command, fail as before.
    err "❌ Required command not found: '$cmd'. Please install it first."
    ;;
  esac
}

check_cmd() {
  command -v "$1" >/dev/null 2>&1
}

assert_nz() {
  if [ -z "$1" ]; then err "❌ Internal error: Missing required value for $2"; fi
}

ensure() {
  if ! "$@"; then err "❌ Command failed: $*"; fi
}

# --- Run main ---
main "$@" || exit 1
