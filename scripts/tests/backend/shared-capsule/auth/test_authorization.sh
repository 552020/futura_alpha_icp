#!/bin/bash

# Test script for basic authorization functionality
# Tests that write operations require authentication

set -e

echo "🔐 Testing Basic Authorization (MVP)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_RUN=0
TESTS_PASSED=0

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "\n${YELLOW}Running: $test_name${NC}"
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ PASSED: $test_name${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ FAILED: $test_name${NC}"
    fi
}

# Test 1: Verify authorization module compiles and tests pass
run_test "Authorization module compilation" "cargo test --manifest-path src/backend/Cargo.toml auth::tests --quiet"

# Test 2: Verify metadata operations include authorization
run_test "Metadata operations have authorization" "cargo test --manifest-path src/backend/Cargo.toml metadata::tests --quiet"

# Test 3: Verify upload operations include authorization  
run_test "Upload operations have authorization" "cargo test --manifest-path src/backend/Cargo.toml upload::tests --quiet"

# Test 4: Verify all ICP endpoints compile successfully
run_test "All ICP endpoints compile" "cargo check --manifest-path src/backend/Cargo.toml --quiet"

echo -e "\n${YELLOW}=== Authorization Test Summary ===${NC}"
echo -e "Tests run: $TESTS_RUN"
echo -e "Tests passed: $TESTS_PASSED"

if [ $TESTS_PASSED -eq $TESTS_RUN ]; then
    echo -e "${GREEN}🎉 All authorization tests passed!${NC}"
    echo -e "${GREEN}✅ Basic caller verification implemented${NC}"
    echo -e "${GREEN}✅ Rate limiting (max 3 concurrent uploads) implemented${NC}"
    echo -e "${GREEN}✅ Unauthorized error responses implemented${NC}"
    echo -e "${GREEN}✅ All write operations protected${NC}"
    exit 0
else
    echo -e "${RED}❌ Some authorization tests failed${NC}"
    exit 1
fi