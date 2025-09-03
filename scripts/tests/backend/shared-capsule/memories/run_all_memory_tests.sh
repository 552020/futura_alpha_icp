#!/bin/bash

# Run all memory tests for shared capsule functionality
# This script executes all memory-related test files in sequence

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
    echo -e "${YELLOW}Memory Tests Suite - Shared Capsule${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo ""
    
    # Run all memory test files
    run_test_file "$SCRIPT_DIR/test_memories_create.sh" "Add Memory Test"
    run_test_file "$SCRIPT_DIR/test_memories_read.sh" "Get Memory Test"
    run_test_file "$SCRIPT_DIR/test_memories_update.sh" "Update Memory Test"
    run_test_file "$SCRIPT_DIR/test_memories_delete.sh" "Delete Memory Test"
    run_test_file "$SCRIPT_DIR/test_memories_list.sh" "List Memories Test"
    run_test_file "$SCRIPT_DIR/test_memory_upload.sh" "Memory Upload Test"
    run_test_file "$SCRIPT_DIR/test_memory_crud.sh" "Memory CRUD Test"
    
    # Print summary
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${YELLOW}Memory Tests Summary${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo -e "Total tests: $TOTAL_TESTS"
    echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}🎉 All memory tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}💥 $FAILED_TESTS memory test(s) failed${NC}"
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi