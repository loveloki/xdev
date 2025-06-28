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

# --- Main execution ---
main() {
  # This script accepts no arguments and will ignore any provided.

  check_system_requirements

  local _arch
  detect_architecture
  _arch="$RETVAL"

  local _version _checksum
  fetch_release_info "$_arch"
  _version="$RETVAL_VERSION"
  _checksum="$RETVAL_CHECKSUM"

  local _binary_path
  download_install_and_verify "$_arch" "$_version" "$_checksum"
  _binary_path="$RETVAL"

  local _executable_path
  finalize_installation "$_binary_path"
  _executable_path="$RETVAL"

  show_success_message "$_version" "$_executable_path"

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
  need_cmd cut
  need_cmd sha256sum
}

detect_architecture() {
  get_architecture || return 1
  local _arch="$RETVAL"
  assert_nz "$_arch" "arch"
  say "✅ Detected architecture: ${_arch}"

  RETVAL="$_arch"
}

fetch_release_info() {
  local _arch="$1"

  say "🔍 Fetching latest release information..."

  # Fetch API response directly into variable
  local _api_response
  _api_response=$(curl --fail --location --silent --show-error \
    --connect-timeout "$CURL_TIMEOUT" \
    --max-time "$CURL_MAX_TIME" \
    --user-agent "xdev-installer/1.0" \
    "$GITHUB_API_URL") || err "❌ Failed to fetch release information from GitHub API."

  # Parse version from API response
  local _version
  _version=$(echo "$_api_response" | grep -o '"tag_name":"[^"]*"' | cut -d '"' -f 4)

  if [ -z "$_version" ]; then
    err "❌ Could not parse version from GitHub API response"
  fi

  # Extract architecture-specific checksum from digest field
  local _asset_name="xdev-${_arch}"
  local _checksum=""
  if echo "$_api_response" | grep -q "\"name\":\"${_asset_name}\""; then
    _checksum=$(echo "$_api_response" | grep -A 10 "\"name\":\"${_asset_name}\"" | grep -o '"digest":"sha256:[^"]*"' | cut -d ':' -f 3 | tr -d '"' | head -1)
  fi

  assert_nz "$_version" "version"
  say "✅ Latest version: ${_version}"

  RETVAL_VERSION="$_version"
  RETVAL_CHECKSUM="$_checksum"
}

download_install_and_verify() {
  local _arch="$1"
  local _version="$2"
  local _checksum="$3"

  # Prepare installation directory
  say "📁 Preparing installation directory: $DEFAULT_INSTALL_DIR"
  if [ ! -d "$DEFAULT_INSTALL_DIR" ]; then
    ensure mkdir -p "$DEFAULT_INSTALL_DIR"
  fi

  # Construct download URL and target path
  local _download_url="https://github.com/${GITHUB_REPO}/releases/download/${_version}/xdev-${_arch}"
  local _binary_path="$DEFAULT_INSTALL_DIR/$BINARY_NAME"

  say "⏳ Downloading xdev ${_version} for ${_arch}..."

  if ! curl --fail --location --progress-bar \
    --connect-timeout "$CURL_TIMEOUT" \
    --max-time "$CURL_MAX_TIME" \
    --user-agent "xdev-installer/1.0" \
    "$_download_url" \
    --output "$_binary_path"; then
    # Clean up partial download on failure
    [ -f "$_binary_path" ] && rm -f "$_binary_path"
    err "❌ Failed to download binary from ${_download_url}"
  fi

  # Verify checksum
  if [ -z "${_checksum:-}" ]; then
    rm -f "$_binary_path"
    err "❌ Could not find checksum for asset xdev-${_arch} from GitHub API."
  fi

  say "🔒 Verifying checksum..."
  if ! verify_checksum "$_binary_path" "$_checksum"; then
    rm -f "$_binary_path"
    err "❌ Checksum verification failed"
  fi
  say "✅ Checksum verification passed"

  RETVAL="$_binary_path"
}

finalize_installation() {
  local _binary_path="$1"

  # Set executable permissions
  say "📦 Finalizing installation..."
  ensure chmod +x "$_binary_path"

  # Verify installation
  if ! "$_binary_path" version >/dev/null 2>&1; then
    rm -f "$_binary_path"
    err "❌ Installation verification failed. The downloaded binary is not working correctly."
  fi

  RETVAL="$_binary_path"
}

show_success_message() {
  local _version="$1"
  local _executable_path="$2"

  say ""
  say "🎉 xdev ${_version} installed successfully!"
  say "   Location: ${_executable_path}"

  # Check if the directory is already in PATH
  if echo "$PATH" | grep -q "$DEFAULT_INSTALL_DIR"; then
    say "✅ $DEFAULT_INSTALL_DIR is already in your PATH - you can use xdev immediately!"
  else
    say "💡 To use xdev, you need to add $DEFAULT_INSTALL_DIR to your PATH."
    say "   You can add it by running: echo 'export PATH=\"$DEFAULT_INSTALL_DIR:\$PATH\"' >> ~/.bashrc"
    say "   Then restart your terminal or run: source ~/.bashrc"
  fi
}

# --- Helper functions ---

get_architecture() {
  local _ostype _cputype _arch
  _ostype="$(uname -s)"
  _cputype="$(uname -m)"

  case "$_ostype" in
    Linux)
      _ostype="unknown-linux-gnu"
      ;;
    *)
      err "❌ Unsupported OS: ${_ostype}"
      ;;
  esac

  case "$_cputype" in
    x86_64 | amd64)
      _cputype="x86_64"
      ;;
    aarch64 | arm64)
      _cputype="aarch64"
      ;;
    *)
      err "❌ Unsupported architecture: ${_cputype}"
      ;;
  esac

  _arch="${_cputype}-${_ostype}"
  RETVAL="$_arch"
}

verify_checksum() {
  local _file="$1"
  local _expected_checksum="$2"

  local _actual_checksum
  _actual_checksum=$(sha256sum "$_file" | cut -d ' ' -f 1)

  if [ "$_actual_checksum" != "$_expected_checksum" ]; then
    say "❌ Checksum verification failed! Expected: $_expected_checksum, Got: $_actual_checksum" >&2
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
  if ! check_cmd "$1"; then
    err "❌ Required command not found: '$1'. Please install it first."
  fi
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
