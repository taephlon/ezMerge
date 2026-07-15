#!/usr/bin/env bash
# ==========================================================================
# ezMerge Quick Demonstration Script
# ==========================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0;37m' # No Color

echo -e "${BLUE}==================================================${NC}"
echo -e "${BLUE}🚀 ezMerge Ecosystem CLI Demo Launcher${NC}"
echo -e "${BLUE}==================================================${NC}"

# 1. Build the Rust project
echo -e "\n${YELLOW}Step 1: Compiling ezmerge-cli in debug mode...${NC}"
cargo build

# Check if binary exists
CLI_BIN="./target/debug/ezmerge-cli"
if [ ! -f "$CLI_BIN" ]; then

    echo -e "${RED}✗ Error: ezmerge binary not found at $CLI_BIN!${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Compilation successful!${NC}"

# 2. Run Diagnostics (Doctor)
echo -e "\n${YELLOW}Step 2: Running System Diagnostics (doctor)...${NC}"
$CLI_BIN doctor

# 3. Search for packages
echo -e "\n${YELLOW}Step 3: Searching for packages matching 'obs' (search)...${NC}"
$CLI_BIN search obs

# 4. View package info
echo -e "\n${YELLOW}Step 4: Displaying metadata for 'hyprland' (info)...${NC}"
$CLI_BIN info hyprland

# 5. List overlays
echo -e "\n${YELLOW}Step 5: Listing curated overlay index (overlay list)...${NC}"
$CLI_BIN overlay list

echo -e "\n${BLUE}==================================================${NC}"
echo -e "${GREEN}🎉 Demo Run Completed!${NC}"
echo -e "You can run more interactive commands manually:"
echo -e "  - Try installing a package: ${GREEN}$CLI_BIN install obs-vkcapture${NC}"
echo -e "  - Rollback mock configurations: ${GREEN}$CLI_BIN undo${NC}"
echo -e "  - Start the web portal: ${GREEN}python3 server.py${NC}"
echo -e "${BLUE}==================================================${NC}"
