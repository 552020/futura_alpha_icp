#!/bin/bash

# Run all shared capsule tests (memories, galleries, and capsule operations)
# This script executes all test suites for shared capsule functionality

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOTAL_SUITES=0
PASSED_SUITES=0
FAILED_SUITES=0

# Helper function to run a test suite
run_test_suite() {
    local test_script="$1"
    local suite_name="$2"
    
    echo -e "${BLUE}=================================================${NC}"
    echo -e "${YELLOW}Running Test Suite: $suite_name${NC}"
    echo -e "${BLUE}=================================================${NC}"
    
    if [ -f "$test_script" ] && [ -x "$test_script" ]; then
        if "$test_script"; then
            echo -e "${GREEN}✅ $suite_name - ALL TESTS PASSED${NC}"
            PASSED_SUITES=$((PASSED_SUITES + 1))
        else
            echo -e "${RED}❌ $suite_name - SOME TESTS FAILED${NC}"
            FAILED_SUITES=$((FAILED_SUITES + 1))
        fi
    else
        echo -e "${RED}❌ $suite_name - SUITE NOT FOUND OR NOT EXECUTABLE${NC}"
        FAILED_SUITES=$((FAILED_SUITES + 1))
    fi
    
    TOTAL_SUITES=$((TOTAL_SUITES + 1))
    echo ""
}

# Helper function to run individual test file
run_individual_test() {
    local test_file="$1"
    local test_name="$2"
    
    echo -e "${BLUE}=================================================${NC}"
    echo -e "${YELLOW}Running Individual Test: $test_name${NC}"
    echo -e "${BLUE}=================================================${NC}"
    
    if [ -f "$test_file" ] && [ -x "$test_file" ]; then
        if "$test_file"; then
            echo -e "${GREEN}✅ $test_name - PASSED${NC}"
            PASSED_SUITES=$((PASSED_SUITES + 1))
        else
            echo -e "${RED}❌ $test_name - FAILED${NC}"
            FAILED_SUITES=$((FAILED_SUITES + 1))
        fi
    else
        echo -e "${RED}❌ $test_name - FILE NOT FOUND OR NOT EXECUTABLE${NC}"
        FAILED_SUITES=$((FAILED_SUITES + 1))
    fi
    
    TOTAL_SUITES=$((TOTAL_SUITES + 1))
    echo ""
}

# Main execution
main() {
    echo -e "${BLUE}=================================================${NC}"
    echo -e "${YELLOW}Shared Capsule Test Suite Runner${NC}"
    echo -e "${BLUE}=================================================${NC}"
    echo ""
    
    # Run capsule operations test
    run_individual_test "$SCRIPT_DIR/test_shared_capsule.sh" "Shared Capsule Operations"
    
    # Run memory test suite
    run_test_suite "$SCRIPT_DIR/memories/run_all_memory_tests.sh" "Memory Operations Suite"
    
    # Run gallery test suite
    run_test_suite "$SCRIPT_DIR/galleries/run_all_gallery_tests.sh" "Gallery Operations Suite"
    
    # Print final summary
    echo -e "${BLUE}=================================================${NC}"
    echo -e "${YELLOW}Final Test Summary - Shared Capsule${NC}"
    echo -e "${BLUE}=================================================${NC}"
    echo -e "Total test suites: $TOTAL_SUITES"
    echo -e "${GREEN}Passed suites: $PASSED_SUITES${NC}"
    echo -e "${RED}Failed suites: $FAILED_SUITES${NC}"
    echo ""
    
    if [ $FAILED_SUITES -eq 0 ]; then
        echo -e "${GREEN}🎉 All shared capsule tests passed!${NC}"
        echo -e "${GREEN}✅ Memories, galleries, and capsule operations all working correctly${NC}"
        exit 0
    else
        echo -e "${RED}💥 $FAILED_SUITES test suite(s) failed${NC}"
        echo -e "${RED}❌ Check individual test outputs above for details${NC}"
        exit 1
    fi
}

# Show usage if requested
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --help, -h    Show this help message"
    echo ""
    echo "This script runs all shared capsule tests including:"
    echo "  - Shared capsule operations (user registration, capsule CRUD)"
    echo "  - Memory operations (add, get, update, delete, list)"
    echo "  - Gallery operations (upload, CRUD)"
    echo ""
    echo "Individual test suites can be run separately:"
    echo "  ./memories/run_all_memory_tests.sh"
    echo "  ./galleries/run_all_gallery_tests.sh"
    echo "  ./test_shared_capsule.sh"
    exit 0
fi

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi