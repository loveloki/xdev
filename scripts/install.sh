#!/usr/bin/env sh

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
_temp_dir=""
_cleanup_needed=false

# --- Main execution ---
main() {
    # This script accepts no arguments.
    # Any arguments passed to it will be ignored.

    local _executable_path="$DEFAULT_INSTALL_DIR/$BINARY_NAME"

    # Set up cleanup trap
    trap cleanup_on_exit EXIT INT TERM

    # Check for necessary commands
    say "🔧 Checking system requirements..."
    need_cmd uname
    need_cmd chmod
    need_cmd mkdir
    need_cmd curl
    need_cmd grep
    need_cmd cut
    need_cmd sha256sum

    # Detect platform architecture
    get_architecture || return 1
    local _arch="$RETVAL"
    assert_nz "$_arch" "arch"
    say "✅ Detected architecture: ${_arch}"

    # Create temporary directory
    _temp_dir=$(mktemp -d)
    _cleanup_needed=true

    # Get latest release information
    say "🔍 Fetching latest release information..."
    local _release_info_file="$_temp_dir/release.json"
    
    if ! curl --fail --location --silent --show-error \
             --connect-timeout "$CURL_TIMEOUT" \
             --max-time "$CURL_MAX_TIME" \
             --user-agent "xdev-installer/1.0" \
             "$GITHUB_API_URL" \
             --output "$_release_info_file"; then
        err "❌ Failed to fetch release information from GitHub API."
    fi

    parse_release_info "$_release_info_file" || return 1
    local _version="$RETVAL"
    
    local _asset_name="xdev-${_arch}"
    local _checksum=""
    # Extract architecture-specific checksum from digest field
    if grep -q "\"name\":\"${_asset_name}\"" "$_release_info_file"; then
        _checksum=$(grep -A 10 "\"name\":\"${_asset_name}\"" "$_release_info_file" | grep -o '"digest":"sha256:[^"]*"' | cut -d ':' -f 3 | tr -d '"' | head -1)
    fi

    assert_nz "$_version" "version"
    say "✅ Latest version: ${_version}"

    # Construct download URLs
    local _download_url="https://github.com/${GITHUB_REPO}/releases/download/${_version}/${_asset_name}"

    # Prepare installation directory
    say "📁 Preparing installation directory: $DEFAULT_INSTALL_DIR"
    if [ ! -d "$DEFAULT_INSTALL_DIR" ]; then
        ensure mkdir -p "$DEFAULT_INSTALL_DIR"
    fi

    # Download binary
    local _binary_path="$_temp_dir/$BINARY_NAME"
    say "⏳ Downloading xdev ${_version} for ${_arch}..."
    
    if ! curl --fail --location --progress-bar \
             --connect-timeout "$CURL_TIMEOUT" \
             --max-time "$CURL_MAX_TIME" \
             --user-agent "xdev-installer/1.0" \
             "$_download_url" \
             --output "$_binary_path"; then
        err "❌ Failed to download binary from ${_download_url}"
    fi

    # Verify checksum
    if [ -z "${_checksum:-}" ]; then
        err "❌ Could not find checksum for asset ${_asset_name} from GitHub API."
    fi

    say "🔒 Verifying checksum..."
    verify_checksum "$_binary_path" "$_checksum"
    say "✅ Checksum verification passed"

    # Install binary
    say "📦 Installing to ${_executable_path}..."
    ensure cp "$_binary_path" "$_executable_path"
    ensure chmod +x "$_executable_path"

    # Verify installation
    if ! "$_executable_path" version >/dev/null 2>&1; then
        err "❌ Installation verification failed. The binary may be corrupted."
    fi

    # Success message
    say ""
    say "🎉 xdev ${_version} installed successfully!"
    say "   Location: ${_executable_path}"
    say "💡 To use xdev, make sure $DEFAULT_INSTALL_DIR is in your PATH"

    return 0
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

parse_release_info() {
    local _release_file="$1"
    
    # Extract version tag using more precise regex for compressed JSON
    local _version
    _version=$(grep -o '"tag_name":"[^"]*"' "$_release_file" | cut -d '"' -f 4)
    
    if [ -z "$_version" ]; then
        err "❌ Could not parse version from GitHub API response"
    fi
    
    RETVAL="$_version"
}

verify_checksum() {
    local _file="$1"
    local _expected_checksum="$2"
    
    local _actual_checksum
    _actual_checksum=$(sha256sum "$_file" | cut -d ' ' -f 1)
    
    if [ "$_actual_checksum" != "$_expected_checksum" ]; then
        err "❌ Checksum verification failed! Expected: $_expected_checksum, Got: $_actual_checksum"
    fi
}

cleanup_on_exit() {
    if [ "$_cleanup_needed" = true ] && [ -n "$_temp_dir" ] && [ -d "$_temp_dir" ]; then
        rm -rf "$_temp_dir"
    fi
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
    command -v "$1" > /dev/null 2>&1
}

assert_nz() {
    if [ -z "$1" ]; then err "assert_nz $2"; fi
}

ensure() {
    if ! "$@"; then err "❌ Command failed: $*"; fi
}

# --- Run main ---
main "$@" || exit 1 
