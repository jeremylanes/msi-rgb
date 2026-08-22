#!/usr/bin/env bash
# ==============================================================================
# MSI Mystic Light Linux RGB Controller - Pro Automated One-Line Installer
# Supports standalone `curl -fsSL https://.../install.sh | bash` as well as local execution.
# Supported OS: Fedora / RHEL / Ubuntu / Debian / Arch / Manjaro / openSUSE
# ==============================================================================
set -euo pipefail

# Color formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${CYAN}${BOLD}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}${BOLD}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}${BOLD}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}${BOLD}[ERROR]${NC} $1"
}

print_banner() {
    echo -e "${RED}${BOLD}"
    cat << "EOF"
  __  __ ____ ___   __  __           _   _        _     _       _     _   
 |  \/  / ___|_ _| |  \/  |_   _ ___| |_(_) ___  | |   (_) __ _| |__ | |_ 
 | |\/| \___ \| |  | |\/| | | | / __| __| |/ __| | |   | |/ _` | '_ \| __|
 | |  | |___) | |  | |  | | |_| \__ \ |_| | (__  | |___| | (_| | | | | |_ 
 |_|  |_|____/___| |_|  |_|\__, |___/\__|_|\___| |_____|_|\__, |_| |_|\__|
                           |___/                          |___/           
            Professional Linux RGB Keyboard Studio Installer
EOF
    echo -e "${NC}"
}

# Check command line flags
INSTALL_MODE="auto" # auto, system, user, uninstall
SKIP_DEPS=false
REPO_URL="https://github.com/jeremylanes/msi-rgb"

for arg in "$@"; do
    case "$arg" in
        --user)
            INSTALL_MODE="user"
            ;;
        --system)
            INSTALL_MODE="system"
            ;;
        --no-deps)
            SKIP_DEPS=true
            ;;
        --uninstall)
            INSTALL_MODE="uninstall"
            ;;
        --help|-h)
            echo "Usage: curl -fsSL https://raw.githubusercontent.com/jeremylanes/msi-rgb/main/bin/install.sh | bash -s -- [OPTIONS]"
            echo "   or: ./bin/install.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --system     Install system-wide to /usr/local/bin (requires sudo)"
            echo "  --user       Install to ~/.local/bin (no root required)"
            echo "  --no-deps    Skip system package manager dependency installation"
            echo "  --uninstall  Remove installed binary, desktop file, icon, and autostart units"
            echo "  --help, -h   Show this help message"
            exit 0
            ;;
        *)
            log_warn "Unknown option: $arg"
            ;;
    esac
done

# Resolve Project Root (Detect if running locally or via remote curl pipe)
TMP_DIR=""
cleanup() {
    if [ -n "$TMP_DIR" ] && [ -d "$TMP_DIR" ]; then
        rm -rf "$TMP_DIR"
    fi
}
trap cleanup EXIT INT TERM

detect_project_source() {
    # Check if executed inside existing clone
    if [ -f "Cargo.toml" ] && grep -q 'name = "msi-rgb"' "Cargo.toml" 2>/dev/null; then
        PROJECT_ROOT="$(pwd)"
        log_info "Running from local repository: ${PROJECT_ROOT}"
    elif [ -n "${BASH_SOURCE[0]:-}" ] && [ -f "${BASH_SOURCE[0]}" ]; then
        local SCRIPT_DIR
        SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        if [ -f "${SCRIPT_DIR}/Cargo.toml" ]; then
            PROJECT_ROOT="${SCRIPT_DIR}"
        elif [ -f "${SCRIPT_DIR}/../Cargo.toml" ]; then
            PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
        else
            PROJECT_ROOT=""
        fi
    else
        PROJECT_ROOT=""
    fi

    # If running from remote curl pipe without local repository, fetch archive
    if [ -z "${PROJECT_ROOT:-}" ] || [ ! -f "${PROJECT_ROOT}/Cargo.toml" ]; then
        log_info "Remote execution detected. Fetching latest source archive from GitHub..."
        TMP_DIR=$(mktemp -d /tmp/msi-rgb-installer.XXXXXX)
        local TAR_URL="${REPO_URL}/archive/refs/heads/main.tar.gz"
        
        if command -v curl >/dev/null 2>&1; then
            curl -fsSL "$TAR_URL" | tar -xz -C "$TMP_DIR" --strip-components=1
        elif command -v wget >/dev/null 2>&1; then
            wget -qO- "$TAR_URL" | tar -xz -C "$TMP_DIR" --strip-components=1
        else
            log_error "Neither curl nor wget found to download repository files."
            exit 1
        fi
        PROJECT_ROOT="$TMP_DIR"
    fi
}

detect_distro() {
    if [ -f /etc/os-release ]; then
        # shellcheck source=/dev/null
        . /etc/os-release
        DISTRO_ID="${ID:-unknown}"
        DISTRO_LIKE="${ID_LIKE:-}"
    else
        DISTRO_ID="unknown"
        DISTRO_LIKE=""
    fi
}

install_system_dependencies() {
    if [ "$SKIP_DEPS" = true ]; then
        log_info "Skipping system package manager dependencies (--no-deps specified)."
        return 0
    fi

    # Check if dependencies are already satisfied
    local HAS_GTK=false
    local HAS_CAIRO=false
    local HAS_OPENRGB=false

    if command -v pkg-config >/dev/null 2>&1; then
        if pkg-config --exists gtk4; then HAS_GTK=true; fi
        if pkg-config --exists cairo; then HAS_CAIRO=true; fi
    fi
    if command -v openrgb >/dev/null 2>&1; then HAS_OPENRGB=true; fi

    if [ "$HAS_GTK" = true ] && [ "$HAS_CAIRO" = true ] && [ "$HAS_OPENRGB" = true ]; then
        log_success "Core build & runtime dependencies (GTK4, Cairo, OpenRGB) are already satisfied."
        return 0
    fi

    detect_distro
    log_info "Detected Linux distribution: ${DISTRO_ID} (${DISTRO_LIKE})"

    SUDO_CMD=""
    if [ "$(id -u)" -ne 0 ]; then
        if command -v sudo >/dev/null 2>&1; then
            SUDO_CMD="sudo"
        else
            log_warn "sudo not found; attempting direct package manager call."
        fi
    fi

    if command -v dnf >/dev/null 2>&1; then
        log_info "Installing build and runtime dependencies via DNF..."
        $SUDO_CMD dnf install -y \
            gtk4-devel \
            cairo-gobject-devel \
            glib2-devel \
            pkgconf-pkg-config \
            gcc \
            openrgb \
            cargo \
            rust || true
    elif command -v apt-get >/dev/null 2>&1; then
        log_info "Installing build and runtime dependencies via APT..."
        $SUDO_CMD apt-get update -y || true
        $SUDO_CMD apt-get install -y \
            libgtk-4-dev \
            libcairo2-dev \
            libglib2.0-dev \
            pkg-config \
            build-essential \
            openrgb \
            cargo \
            rustc || true
    elif command -v pacman >/dev/null 2>&1; then
        log_info "Installing build and runtime dependencies via Pacman..."
        $SUDO_CMD pacman -Sy --noconfirm \
            gtk4 \
            cairo \
            glib2 \
            pkgconf \
            base-devel \
            openrgb \
            rust || true
    elif command -v zypper >/dev/null 2>&1; then
        log_info "Installing build and runtime dependencies via Zypper..."
        $SUDO_CMD zypper --non-interactive install \
            gtk4-devel \
            cairo-devel \
            glib2-devel \
            pkg-config \
            gcc \
            openrgb \
            cargo \
            rust || true
    else
        log_warn "No supported package manager detected (dnf, apt, pacman, zypper)."
        log_warn "Please ensure GTK4, Cairo, GLib, pkg-config, Rust, and OpenRGB are installed."
    fi
}

check_rust_toolchain() {
    if ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then
        log_warn "Rust toolchain (cargo/rustc) was not found in PATH."
        if command -v rustup >/dev/null 2>&1; then
            log_info "Installing stable toolchain via rustup..."
            rustup default stable
        else
            log_info "Installing Rust toolchain via https://sh.rustup.rs..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            # shellcheck source=/dev/null
            if [ -f "$HOME/.cargo/env" ]; then
                source "$HOME/.cargo/env"
            fi
        fi
    fi

    log_success "Rust compiler ready: $(rustc --version)"
}

build_release_binary() {
    log_info "Compiling optimized release binary with Cargo..."
    cd "${PROJECT_ROOT}"
    cargo build --release
    if [ ! -f "${PROJECT_ROOT}/target/release/msi-rgb" ]; then
        log_error "Build failed: target/release/msi-rgb binary not found."
        exit 1
    fi
    log_success "Release binary compiled successfully."
}

uninstall_msi_rgb() {
    log_info "Uninstalling MSI Mystic Light RGB Controller..."

    # Remove system paths
    if command -v sudo >/dev/null 2>&1 && [ "$(id -u)" -ne 0 ]; then
        sudo rm -f /usr/local/bin/msi-rgb
        sudo rm -f /usr/share/applications/com.msi.mysticlight.rgb.desktop
        sudo rm -f /usr/share/icons/hicolor/scalable/apps/com.msi.mysticlight.rgb.svg
    else
        rm -f /usr/local/bin/msi-rgb || true
        rm -f /usr/share/applications/com.msi.mysticlight.rgb.desktop || true
        rm -f /usr/share/icons/hicolor/scalable/apps/com.msi.mysticlight.rgb.svg || true
    fi

    # Remove user paths
    rm -f "${HOME}/.local/bin/msi-rgb"
    rm -f "${HOME}/.local/share/applications/com.msi.mysticlight.rgb.desktop"
    rm -f "${HOME}/.local/share/icons/hicolor/scalable/apps/com.msi.mysticlight.rgb.svg"
    rm -f "${HOME}/.config/autostart/msi-rgb.desktop"
    rm -f "${HOME}/.config/systemd/user/msi-rgb.service"

    if command -v systemctl >/dev/null 2>&1; then
        systemctl --user daemon-reload || true
    fi

    log_success "MSI Mystic Light RGB Controller has been uninstalled."
    exit 0
}

perform_installation() {
    local BIN_DIR
    local APP_DIR
    local ICON_DIR
    local USE_SUDO=false

    if [ "$INSTALL_MODE" = "system" ] || { [ "$INSTALL_MODE" = "auto" ] && [ "$(id -u)" -eq 0 ]; }; then
        BIN_DIR="/usr/local/bin"
        APP_DIR="/usr/share/applications"
        ICON_DIR="/usr/share/icons/hicolor/scalable/apps"
        USE_SUDO=false
    elif [ "$INSTALL_MODE" = "auto" ] && command -v sudo >/dev/null 2>&1; then
        BIN_DIR="/usr/local/bin"
        APP_DIR="/usr/share/applications"
        ICON_DIR="/usr/share/icons/hicolor/scalable/apps"
        USE_SUDO=true
    else
        BIN_DIR="${HOME}/.local/bin"
        APP_DIR="${HOME}/.local/share/applications"
        ICON_DIR="${HOME}/.local/share/icons/hicolor/scalable/apps"
        USE_SUDO=false
    fi

    log_info "Installing executable to: ${BIN_DIR}/msi-rgb"
    log_info "Installing Desktop entry to: ${APP_DIR}/com.msi.mysticlight.rgb.desktop"
    log_info "Installing Icon to: ${ICON_DIR}/com.msi.mysticlight.rgb.svg"

    if [ "$USE_SUDO" = true ]; then
        sudo mkdir -p "${BIN_DIR}" "${APP_DIR}" "${ICON_DIR}"
        sudo install -m 755 "${PROJECT_ROOT}/target/release/msi-rgb" "${BIN_DIR}/msi-rgb"
        sudo install -m 644 "${PROJECT_ROOT}/packaging/com.msi.mysticlight.rgb.desktop" "${APP_DIR}/com.msi.mysticlight.rgb.desktop"
        sudo install -m 644 "${PROJECT_ROOT}/packaging/com.msi.mysticlight.rgb.svg" "${ICON_DIR}/com.msi.mysticlight.rgb.svg"
        if command -v update-desktop-database >/dev/null 2>&1; then
            sudo update-desktop-database "${APP_DIR}" >/dev/null 2>&1 || true
        fi
        if command -v gtk-update-icon-cache >/dev/null 2>&1; then
            sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor >/dev/null 2>&1 || true
        fi
    else
        mkdir -p "${BIN_DIR}" "${APP_DIR}" "${ICON_DIR}"
        install -m 755 "${PROJECT_ROOT}/target/release/msi-rgb" "${BIN_DIR}/msi-rgb"
        install -m 644 "${PROJECT_ROOT}/packaging/com.msi.mysticlight.rgb.desktop" "${APP_DIR}/com.msi.mysticlight.rgb.desktop"
        install -m 644 "${PROJECT_ROOT}/packaging/com.msi.mysticlight.rgb.svg" "${ICON_DIR}/com.msi.mysticlight.rgb.svg"
        if command -v update-desktop-database >/dev/null 2>&1; then
            update-desktop-database "${APP_DIR}" >/dev/null 2>&1 || true
        fi
    fi

    # Configure session autostart & systemd user service
    mkdir -p "${HOME}/.config/autostart"
    cat << EOF > "${HOME}/.config/autostart/msi-rgb.desktop"
[Desktop Entry]
Type=Application
Name=MSI Mystic Light RGB Daemon
Comment=Auto-restore RGB lighting state on login
Exec=msi-rgb --apply-default --daemon
Hidden=false
NoDisplay=true
X-GNOME-Autostart-enabled=true
EOF

    mkdir -p "${HOME}/.config/systemd/user"
    cat << EOF > "${HOME}/.config/systemd/user/msi-rgb.service"
[Unit]
Description=MSI Mystic Light RGB Keyboard Lighting Controller
After=graphical-session.target openrgb.service
Wants=openrgb.service

[Service]
Type=simple
ExecStart=${BIN_DIR}/msi-rgb --daemon --apply-default
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=default.target
EOF

    if command -v systemctl >/dev/null 2>&1; then
        systemctl --user daemon-reload >/dev/null 2>&1 || true
        systemctl --user enable msi-rgb.service >/dev/null 2>&1 || true
    fi

    log_success "Installation completed successfully!"
}

# Main Execution Flow
print_banner

if [ "$INSTALL_MODE" = "uninstall" ]; then
    uninstall_msi_rgb
fi

log_info "Step 1/5: Locating and verifying project sources..."
detect_project_source

log_info "Step 2/5: Checking and installing system dependencies..."
install_system_dependencies

log_info "Step 3/5: Verifying Rust toolchain..."
check_rust_toolchain

log_info "Step 4/5: Building release binary..."
build_release_binary

log_info "Step 5/5: Deploying binary, desktop launcher, icons, and autostart..."
perform_installation

echo ""
echo -e "${GREEN}${BOLD}================================================================${NC}"
echo -e "${GREEN}${BOLD}  MSI Mystic Light Linux RGB Controller is Ready!               ${NC}"
echo -e "${GREEN}${BOLD}================================================================${NC}"
echo -e "  • Launch GUI from Application Menu or Terminal:"
echo -e "      ${CYAN}${BOLD}msi-rgb${NC}"
echo -e "  • Apply a profile from CLI or Hyprland keybind:"
echo -e "      ${CYAN}${BOLD}msi-rgb --profile \"Cyberpunk Neon\"${NC}"
echo -e "  • List all configured profiles:"
echo -e "      ${CYAN}${BOLD}msi-rgb --list-profiles${NC}"
echo ""
