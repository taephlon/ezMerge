#!/usr/bin/env bash
# ==========================================================================
# ezMerge Installer Script
# ==========================================================================

set -euo pipefail

# ANSI color codes for premium terminal UI
PURPLE='\033[1;35m'
TEAL='\033[1;36m'
GREEN='\033[1;32m'
YELLOW='\033[1;33m'
RED='\033[1;31m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${PURPLE}======================================================================${NC}"
echo -e "${TEAL}                    🚀 Welcome to the ezMerge Installer${NC}"
echo -e "${PURPLE}======================================================================${NC}"
echo -e "ezMerge is the modern companion for Gentoo Linux and Portage."
echo -e "This script will compile the CLI and install it onto your system.\n"

# ==========================================
# 1. Dependency Checks
# ==========================================
echo -e "${BOLD}[1/4] Checking dependencies...${NC}"
DEPENDENCIES_MET=true
WARNING_ISSUED=false

# Check Rust/Cargo
if command -v cargo >/dev/null 2>&1 && command -v rustc >/dev/null 2>&1; then
    RUST_VER=$(rustc --version | cut -d' ' -f2)
    echo -e "  ${GREEN}✓${NC} Rust Compiler (rustc) & Cargo found (v${RUST_VER})"
else
    echo -e "  ${RED}✗${NC} Rust & Cargo NOT found."
    echo -e "    ${YELLOW}Warning: Rust is required to build ezMerge from source.${NC}"
    echo -e "    👉 Install it via: ${BOLD}emerge dev-lang/rust${NC} (Gentoo) or visit ${TEAL}https://rustup.rs${NC}"
    DEPENDENCIES_MET=false
fi

# Check Python3 (for web search portal)
if command -v python3 >/dev/null 2>&1; then
    PY_VER=$(python3 --version | cut -d' ' -f2)
    echo -e "  ${GREEN}✓${NC} Python 3 found (v${PY_VER}) [Required for Web Portal]"
else
    echo -e "  ${YELLOW}⚠${NC} Python 3 NOT found."
    echo -e "    ${YELLOW}Note: Python 3 is required to run the Web Search Portal (server.py).${NC}"
    WARNING_ISSUED=true
fi

# Check Portage/Emerge environment (for live functionality)
if command -v emerge >/dev/null 2>&1; then
    echo -e "  ${GREEN}✓${NC} Gentoo Portage (emerge) detected."
else
    echo -e "  ${YELLOW}⚠${NC} Gentoo Portage (emerge) NOT detected."
    echo -e "    ${YELLOW}Warning: ezMerge is designed to manage Gentoo Portage overlays & USE flags.${NC}"
    echo -e "             On non-Gentoo systems, ezMerge will run in ${BOLD}mock/simulation mode${NC}."
    echo -e "             This is fully functional for demoing, debugging, or sandboxed use."
    WARNING_ISSUED=true
fi

# Check eselect repository (for overlay management)
if command -v eselect >/dev/null 2>&1 && eselect repository list >/dev/null 2>&1; then
    echo -e "  ${GREEN}✓${NC} eselect-repository module detected."
else
    if command -v emerge >/dev/null 2>&1; then
        echo -e "  ${YELLOW}⚠${NC} eselect-repository module NOT detected."
        echo -e "    ${YELLOW}Warning: ezMerge uses 'eselect repository' to add and enable overlays.${NC}"
        echo -e "             Please install it on Gentoo via: ${BOLD}emerge app-eselect/eselect-repository${NC}"
        WARNING_ISSUED=true
    fi
fi

# Check for conflict packages (like legacy layman)
if command -v layman >/dev/null 2>&1; then
    echo -e "  ${YELLOW}⚠${NC} Conflict Warning: Legacy 'layman' overlay manager detected."
    echo -e "    ${YELLOW}Warning: ezMerge manages overlays via eselect-repository (/etc/portage/repos.conf/).${NC}"
    echo -e "             Coexistence is supported, but to prevent duplicate overlay definitions or"
    echo -e "             make.conf profile corruption, we highly recommend migrating your overlays"
    echo -e "             to eselect-repository."
    WARNING_ISSUED=true
fi

if [ "$DEPENDENCIES_MET" = false ]; then
    echo -e "\n${RED}✗ Error: Cannot proceed with installation due to missing build dependencies (Rust).${NC}"
    exit 1
fi

echo -e "  ${GREEN}✓ All compilation requirements met!${NC}"

# ==========================================
# 2. Installation Path Setup
# ==========================================
echo -e "\n${BOLD}[2/4] Configuration & Install Location...${NC}"

# Default directory prefix
INSTALL_DIR="/usr/local/bin"
USE_SUDO=false

if [ "$EUID" -eq 0 ]; then
    echo -e "  👉 Running as root. Binary will be installed globally to: ${TEAL}/usr/local/bin/ezmerge${NC}"
else
    echo -e "  Running as non-root user."
    echo -e "  Choose install destination:"
    echo -e "    1) Global installation to /usr/local/bin (requires sudo)"
    echo -e "    2) Local installation to your home directory (~/.local/bin)"
    
    read -rp "  Select option [1-2] (default 2): " choice
    choice=${choice:-2}

    if [ "$choice" -eq 1 ]; then
        INSTALL_DIR="/usr/local/bin"
        USE_SUDO=true
        echo -e "  👉 Will install globally using sudo."
    else
        INSTALL_DIR="$HOME/.local/bin"
        echo -e "  👉 Will install locally to: ${TEAL}$INSTALL_DIR/ezmerge${NC}"
        mkdir -p "$INSTALL_DIR"
        
        # Verify if ~/.local/bin is in PATH
        if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
            echo -e "  ${YELLOW}⚠ Note: $INSTALL_DIR is not in your current PATH.${NC}"
            echo -e "    You may need to add it to your ~/.bashrc or ~/.zshrc:"
            echo -e "    ${BOLD}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
        fi
    fi
fi

# ==========================================
# 3. Compiling Binary
# ==========================================
echo -e "\n${BOLD}[3/4] Compiling ezMerge CLI...${NC}"
echo -e "  Building release target using cargo..."
cargo build --release

# Double check compile output
CLI_SOURCE_BIN="target/release/ezmerge-cli"
if [ ! -f "$CLI_SOURCE_BIN" ]; then
    echo -e "${RED}✗ Error: Compiled binary not found at $CLI_SOURCE_BIN!${NC}"
    exit 1
fi
echo -e "  ${GREEN}✓ Build successful!${NC}"

# ==========================================
# 4. Installing Binary
# ==========================================
echo -e "\n${BOLD}[4/4] Copying binary to destination...${NC}"
TARGET_PATH="$INSTALL_DIR/ezmerge"

if [ "$USE_SUDO" = true ]; then
    echo -e "  Copying binary (using sudo)..."
    sudo cp "$CLI_SOURCE_BIN" "$TARGET_PATH"
    sudo chmod 755 "$TARGET_PATH"
else
    echo -e "  Copying binary..."
    cp "$CLI_SOURCE_BIN" "$TARGET_PATH"
    chmod 755 "$TARGET_PATH"
fi

echo -e "  ${GREEN}✓ Binary installed to: $TARGET_PATH${NC}"

# ==========================================
# Summary & Next Steps
# ==========================================
echo -e "\n${PURPLE}======================================================================${NC}"
echo -e "${GREEN}🎉 ezMerge CLI has been successfully installed!${NC}"
echo -e "${PURPLE}======================================================================${NC}"
echo -e "Verify the installation by running diagnostics:"
echo -e "  ${BOLD}ezmerge doctor${NC}"
echo -e ""
echo -e "Quick Usage Guide:"
echo -e "  - Search for packages:  ${TEAL}ezmerge search <query>${NC}"
echo -e "  - Inspect package info: ${TEAL}ezmerge info <package-name>${NC}"
echo -e "  - Install interactively: ${TEAL}ezmerge install <package-name>${NC}"
echo -e "  - List Gentoo overlays: ${TEAL}ezmerge overlay list${NC}"
echo -e "  - Revert config edits:  ${TEAL}ezmerge undo${NC}"
echo -e ""
echo -e "Web Package search portal:"
echo -e "  Start local API/UI server: ${BOLD}python3 server.py${NC}"
echo -e "  Then access: ${TEAL}http://localhost:8080${NC}"
echo -e "${PURPLE}======================================================================${NC}"
