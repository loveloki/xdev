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
    # Parse command line arguments
    local _install_dir="$DEFAULT_INSTALL_DIR"
    local _force_install=false
    local _skip_path_check=false
    
    while [ $# -gt 0 ]; do
        case $1 in
            --install-dir=*)
                _install_dir="${1#*=}"
                ;;
            --install-dir)
                shift
                _install_dir="$1"
                ;;
            --force)
                _force_install=true
                ;;
            --skip-path-check)
                _skip_path_check=true
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                err "❌ Unknown option: $1. Use --help for usage information."
                ;;
        esac
        shift
    done

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

    # Check if already installed and handle accordingly
    local _executable_path="$_install_dir/$BINARY_NAME"
    if [ -f "$_executable_path" ] && [ "$_force_install" = false ]; then
        local _current_version
        if _current_version=$("$_executable_path" --version 2>/dev/null | grep -o 'v[0-9.]*' | head -1); then
            say "ℹ️  xdev is already installed (version: ${_current_version})"
            say "   Use --force to reinstall"
            return 0
        fi
    fi

    # Get latest release information
    say "🔍 Fetching latest release information..."
    local _release_info_file="$_temp_dir/release.json"
    
    if ! curl --fail --location --silent --show-error \
             --connect-timeout "$CURL_TIMEOUT" \
             --max-time "$CURL_MAX_TIME" \
             --user-agent "xdev-installer/1.0" \
             "$GITHUB_API_URL" \
             --output "$_release_info_file"; then
        say "⚠️  GitHub API failed, trying fallback method..."
        get_version_fallback || return 1
        local _version="$RETVAL"
    else
        parse_release_info "$_release_info_file" || return 1
        local _version="$RETVAL"
        local _checksum="$RETVAL2"
    fi

    assert_nz "$_version" "version"
    say "✅ Latest version: ${_version}"

    # Construct download URLs
    local _asset_name="xdev-${_arch}"
    local _download_url="https://github.com/${GITHUB_REPO}/releases/download/${_version}/${_asset_name}"
    local _checksum_url="https://github.com/${GITHUB_REPO}/releases/download/${_version}/checksums.txt"

    # Prepare installation directory
    say "📁 Preparing installation directory: ${_install_dir}"
    if [ ! -d "$_install_dir" ]; then
        ensure mkdir -p "$_install_dir"
    fi

    # Check write permissions
    if [ ! -w "$_install_dir" ]; then
        err "❌ No write permission for ${_install_dir}. Try running with sudo or choose a different directory with --install-dir"
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

    # Verify checksum if available
    if [ -n "${_checksum:-}" ]; then
        say "🔒 Verifying checksum..."
        verify_checksum "$_binary_path" "$_checksum" || return 1
        say "✅ Checksum verification passed"
    else
        say "⚠️  Checksum verification skipped (not available from API)"
        # Try to download checksums file as fallback
        if curl --fail --location --silent --show-error \
               --connect-timeout "$CURL_TIMEOUT" \
               --max-time "$CURL_MAX_TIME" \
               "$_checksum_url" \
               --output "$_temp_dir/checksums.txt" 2>/dev/null; then
            if verify_checksum_from_file "$_binary_path" "$_asset_name" "$_temp_dir/checksums.txt"; then
                say "✅ Checksum verification passed (from checksums file)"
            else
                say "⚠️  Checksum verification failed, but continuing..."
            fi
        fi
    fi

    # Install binary
    say "📦 Installing to ${_executable_path}..."
    ensure cp "$_binary_path" "$_executable_path"
    ensure chmod +x "$_executable_path"

    # Verify installation
    if ! "$_executable_path" --version >/dev/null 2>&1; then
        err "❌ Installation verification failed. The binary may be corrupted."
    fi

    # Check PATH and provide guidance
    if [ "$_skip_path_check" = false ]; then
        check_path_and_advise "$_install_dir"
    fi

    # Success message
    say ""
    say "🎉 xdev ${_version} installed successfully!"
    say "   Location: ${_executable_path}"
    
    # Test if command is available
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        say "✅ xdev is ready to use. Try: xdev --help"
    else
        say "💡 To use xdev, make sure ${_install_dir} is in your PATH"
    fi

    return 0
}

# --- Helper functions ---

show_help() {
    cat << 'EOF'
xdev installer

USAGE:
    install.sh [OPTIONS]

OPTIONS:
    --install-dir <DIR>    Install directory (default: ~/.local/bin)
    --force               Force reinstall even if already installed
    --skip-path-check     Skip PATH checking and advice
    -h, --help            Show this help message

EXAMPLES:
    # Install to default location
    ./install.sh

    # Install to custom directory
    ./install.sh --install-dir=/usr/local/bin

    # Force reinstall
    ./install.sh --force
EOF
}

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
    
    # Extract version tag
    local _version
    _version=$(grep '"tag_name"' "$_release_file" | cut -d '"' -f 4)
    
    if [ -z "$_version" ]; then
        err "❌ Could not parse version from GitHub API response"
    fi

    # Try to extract checksum if available in release assets
    local _checksum=""
    if grep -q '"name".*"checksums\|sha256' "$_release_file"; then
        # This is a simplified approach - in a real scenario, you'd parse the JSON more carefully
        _checksum=$(grep -A 5 -B 5 '"name".*"checksums\|sha256' "$_release_file" | grep '"browser_download_url"' | cut -d '"' -f 4 || true)
    fi
    
    RETVAL="$_version"
    RETVAL2="$_checksum"
}

get_version_fallback() {
    say "🔄 Using fallback method to get version..."
    local _latest_url
    _latest_url=$(curl -s -L -o /dev/null -w '%{url_effective}' \
                      --connect-timeout "$CURL_TIMEOUT" \
                      --max-time "$CURL_MAX_TIME" \
                      "https://github.com/${GITHUB_REPO}/releases/latest")
    
    local _version
    _version=$(echo "$_latest_url" | grep -o 'v[0-9][0-9.]*[0-9]\|v[0-9]' | head -1)

    if [ -z "$_version" ]; then
        err "❌ Could not resolve the latest version. Please check https://github.com/${GITHUB_REPO}/releases"
    fi
    
    RETVAL="$_version"
}

verify_checksum() {
    local _file="$1"
    local _expected_checksum="$2"
    
    if [ -z "$_expected_checksum" ]; then
        return 1
    fi
    
    local _actual_checksum
    _actual_checksum=$(sha256sum "$_file" | cut -d ' ' -f 1)
    
    if [ "$_actual_checksum" != "$_expected_checksum" ]; then
        err "❌ Checksum verification failed! Expected: $_expected_checksum, Got: $_actual_checksum"
    fi
}

verify_checksum_from_file() {
    local _file="$1"
    local _asset_name="$2"
    local _checksums_file="$3"
    
    if [ ! -f "$_checksums_file" ]; then
        return 1
    fi
    
    local _expected_checksum
    _expected_checksum=$(grep "$_asset_name" "$_checksums_file" | cut -d ' ' -f 1)
    
    if [ -z "$_expected_checksum" ]; then
        return 1
    fi
    
    verify_checksum "$_file" "$_expected_checksum"
}

check_path_and_advise() {
    local _install_dir="$1"
    
    say "🔍 Checking PATH configuration..."
    
    # Check if install directory is in PATH
    case ":$PATH:" in
        *":$_install_dir:"*)
            say "✅ ${_install_dir} is already in your PATH"
            return 0
            ;;
    esac
    
    say "💡 ${_install_dir} is not in your PATH"
    say ""
    say "To add it permanently, add this line to your shell profile:"
    
    # Detect shell and provide specific advice
    local _shell_profile=""
    case "${SHELL:-}" in
        */bash)
            _shell_profile="~/.bashrc or ~/.bash_profile"
            ;;
        */zsh)
            _shell_profile="~/.zshrc"
            ;;
        */fish)
            _shell_profile="~/.config/fish/config.fish"
            say "  set -gx PATH $_install_dir \$PATH"
            say ""
            say "Or run this command now:"
            say "  fish_add_path $_install_dir"
            return 0
            ;;
        *)
            _shell_profile="your shell profile"
            ;;
    esac
    
    say "  export PATH=\"$_install_dir:\$PATH\""
    say ""
    say "Add it to: $_shell_profile"
    say ""
    say "Or run this command for the current session:"
    say "  export PATH=\"$_install_dir:\$PATH\""
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
