#!/usr/bin/env bash
# ==========================================================================
# ezMerge CI Pipeline - Ebuild QA Verification
# ==========================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0;37m' # No Color

OVERLAY_DIR="ezmerge-overlay"
ERROR_COUNT=0

echo -e "${BLUE}==================================================${NC}"
echo -e "${BLUE}⚡ running ezMerge CI Overlay Validator...${NC}"
echo -e "${BLUE}==================================================${NC}"

# Check overlay directory exists
if [ ! -d "$OVERLAY_DIR" ]; then
    echo -e "${RED}✗ Error: Overlay directory '$OVERLAY_DIR' not found!${NC}"
    exit 1
fi

# 1. Verify layout.conf and repo_name
echo -n "Verifying repository structure... "
if [ -f "$OVERLAY_DIR/metadata/layout.conf" ] && [ -f "$OVERLAY_DIR/profiles/repo_name" ]; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAILED (Missing layout.conf or repo_name)${NC}"
    ERROR_COUNT=$((ERROR_COUNT + 1))
fi

# 2. Verify categories list matches subdirectories
echo "Verifying category alignment..."
if [ -f "$OVERLAY_DIR/profiles/categories" ]; then
    while IFS= read -r category || [ -n "$category" ]; do
        if [ -n "$category" ]; then
            if [ -d "$OVERLAY_DIR/$category" ]; then
                echo -e "  ├── Category ${GREEN}'$category'${NC} matches directory structure."
            else
                echo -e "  ├── ${YELLOW}⚠ Warning: Category '$category' declared but directory missing.${NC}"
            fi
        fi
    done < "$OVERLAY_DIR/profiles/categories"
else
    echo -e "  └── ${RED}✗ profiles/categories list is missing!${NC}"
    ERROR_COUNT=$((ERROR_COUNT + 1))
fi

# 3. Analyze Ebuild style & rules
echo "Auditing ebuild files..."
ebuild_count=0
while IFS= read -r -d '' ebuild_file; do
    ebuild_count=$((ebuild_count + 1))
    echo -e "  ├── Inspecting: ${BLUE}$ebuild_file${NC}"
    
    # Check for MIT License Header
    if ! grep -q "Distributed under the terms of the MIT License" "$ebuild_file"; then
        echo -e "  │   ${RED}✗ QA Fail: Missing MIT license header!${NC}"
        ERROR_COUNT=$((ERROR_COUNT + 1))
    fi
    
    # Check for trailing whitespaces
    if grep -q "[[:blank:]]$" "$ebuild_file"; then
        echo -e "  │   ${YELLOW}⚠ QA Warn: Trailing whitespace detected.${NC}"
    fi

    # Check for deprecated EAPI
    if grep -q "EAPI=[0-7]" "$ebuild_file"; then
        echo -e "  │   ${RED}✗ QA Fail: Deprecated EAPI detected (use EAPI=8).${NC}"
        ERROR_COUNT=$((ERROR_COUNT + 1))
    fi
done < <(find "$OVERLAY_DIR" -type f -name "*.ebuild" -print0)

if [ "$ebuild_count" -eq 0 ]; then
    echo -e "  └── ${YELLOW}⚠ Warning: No ebuild files found in the overlay.${NC}"
fi

# 4. Try running pkgcheck if available
echo -n "Checking for pkgcheck tool... "
if command -v pkgcheck &> /dev/null; then
    echo -e "${GREEN}Found! Running checks...${NC}"
    if pkgcheck scan "$OVERLAY_DIR"; then
        echo -e "${GREEN}✓ pkgcheck scanned overlay successfully without errors.${NC}"
    else
        echo -e "${YELLOW}⚠ pkgcheck flagged some QA warnings/errors.${NC}"
    fi
else
    echo -e "${YELLOW}Not found (Skipping system pkgcheck scanner)${NC}"
fi

echo -e "${BLUE}--------------------------------------------------${NC}"
if [ "$ERROR_COUNT" -eq 0 ]; then
    echo -e "${GREEN}🎉 CI BUILD PASSED! Overlay layout matches all QA requirements.${NC}"
    exit 0
else
    echo -e "${RED}✗ CI BUILD FAILED with $ERROR_COUNT errors.${NC}"
    exit 1
fi
