#!/bin/bash

# Run all gallery tests for shared capsule functionality
# This script executes all gallery-related test files in sequence

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper function to run a test file
run_test_file() {
    local test_file="$1"
    local test_name="$2"
    
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${YELLOW}Running: $test_name${NC}"
    echo -e "${BLUE}=========================================${NC}"
    
    if [ -f "$test_file" ] && [ -x "$test_file" ]; then
        if "$test_file"; then
            echo -e "${GREEN}✅ $test_name - PASSED${NC}"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            echo -e "${RED}❌ $test_name - FAILED${NC}"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    else
        echo -e "${RED}❌ $test_name - FILE NOT FOUND OR NOT EXECUTABLE${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo ""
}

# Main execution
main() {
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${YELLOW}Gallery Tests Suite - Shared Capsule${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo ""
    
    # Run all gallery test files
    run_test_file "$SCRIPT_DIR/test_gallery_upload.sh" "Gallery Upload Test"
    run_test_file "$SCRIPT_DIR/test_gallery_crud.sh" "Gallery CRUD Test"
    
    # Print summary
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${YELLOW}Gallery Tests Summary${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo -e "Total tests: $TOTAL_TESTS"
    echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}🎉 All gallery tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}💥 $FAILED_TESTS gallery test(s) failed${NC}"
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi