#!/bin/bash

# CI Scan Detection for Storage API Regression
# Scans codebase for potential regressions to direct HashMap usage

set -e

echo "🔍 Scanning for storage API regression patterns..."
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

FOUND_ISSUES=0

# Function to scan for problematic patterns
scan_pattern() {
    local pattern="$1"
    local description="$2"
    local exclude_files="$3"

    echo -e "\n${YELLOW}Scanning for: $description${NC}"
    echo "Pattern: $pattern"

    # Use ripgrep if available, otherwise fall back to grep
    if command -v rg &> /dev/null; then
        local results=$(rg "$pattern" --type rust --glob "!**/target/**" --glob "!**/node_modules/**" $exclude_files 2>/dev/null || true)
    else
        local results=$(find . -name "*.rs" -not -path "./target/*" -not -path "./node_modules/*" -exec grep -l "$pattern" {} \; 2>/dev/null | head -10 || true)
    fi

    if [ -n "$results" ]; then
        echo -e "${RED}❌ FOUND POTENTIAL ISSUES:${NC}"
        echo "$results" | while read -r line; do
            echo -e "  ${RED}•${NC} $line"
        done
        FOUND_ISSUES=$((FOUND_ISSUES + 1))
    else
        echo -e "${GREEN}✅ No issues found${NC}"
    fi
}

# Scan for .iter() and .values() calls on HashMap-like structures
scan_pattern "\.iter\(\)" "Direct .iter() calls (may bypass storage API)" "--glob !**/capsule_store/**"
scan_pattern "\.values\(\)" "Direct .values() calls (may bypass storage API)" "--glob !**/capsule_store/**"
scan_pattern "HashMap.*\.iter" "HashMap.iter() usage (should use storage API)" "--glob !**/capsule_store/**"
scan_pattern "HashMap.*\.values" "HashMap.values() usage (should use storage API)" "--glob !**/capsule_store/**"

# Scan for direct HashMap manipulation that should use storage API
scan_pattern "capsules\.insert" "Direct HashMap insert (should use store.upsert())" "--glob !**/capsule_store/**"
scan_pattern "capsules\.remove" "Direct HashMap remove (should use store.remove())" "--glob !**/capsule_store/**"
scan_pattern "capsules\.get_mut" "Direct HashMap get_mut (should use store.update())" "--glob !**/capsule_store/**"

# Scan for old with_capsules usage (excluding known administrative functions)
scan_pattern "with_capsules" "Usage of old with_capsules API (should use with_capsule_store)" "--glob !**/capsule_store/** --glob !**/import_capsules_from_upgrade*"

# Special check for .iter()/.values() in capsule-related files
echo -e "\n${YELLOW}Special check: Capsule-related files with .iter()/.values()${NC}"
if command -v rg &> /dev/null; then
    CAPSULE_ITER_ISSUES=$(rg "\.iter\(\)|\.values\(\)" --type rust --glob "**/capsule*" --glob "!**/capsule_store/**" --glob "!**/target/**" 2>/dev/null || true)
else
    CAPSULE_ITER_ISSUES=$(find . -name "*capsule*.rs" -not -path "./target/*" -not -path "./capsule_store/*" -exec grep -l "\.iter()\|\.values()" {} \; 2>/dev/null || true)
fi

if [ -n "$CAPSULE_ITER_ISSUES" ]; then
    echo -e "${RED}❌ FOUND CAPSULE-RELATED .iter()/.values() USAGE:${NC}"
    echo "$CAPSULE_ITER_ISSUES" | while read -r line; do
        echo -e "  ${RED}•${NC} $line"
    done
    FOUND_ISSUES=$((FOUND_ISSUES + 1))
else
    echo -e "${GREEN}✅ No capsule-related .iter()/.values() usage found${NC}"
fi

echo -e "\n=================================================="
echo -e "${YELLOW}SCAN COMPLETE${NC}"

if [ $FOUND_ISSUES -eq 0 ]; then
    echo -e "${GREEN}🎉 SUCCESS: No storage API regression patterns detected!${NC}"
    echo -e "${GREEN}The capsule storage foundation appears to be properly implemented.${NC}"
    exit 0
else
    echo -e "${RED}⚠️  WARNING: Found $FOUND_ISSUES potential regression patterns!${NC}"
    echo -e "${RED}Please review the flagged files and ensure they're using the new storage API.${NC}"
    echo -e "${YELLOW}Note: Some findings may be false positives in test files or legacy code.${NC}"
    exit 1
fi
