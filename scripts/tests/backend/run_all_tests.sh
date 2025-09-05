#!/bin/bash

# Master Test Runner for Backend Integration Tests
# Features:
# - Runs all tests with progress tracking
# - Resume capability from last failed test
# - State persistence across runs
# - Detailed reporting

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOGS_DIR="$SCRIPT_DIR/logs"
STATE_FILE="$SCRIPT_DIR/.test_state"
LOG_FILE="$LOGS_DIR/test_run_$(date +%Y%m%d_%H%M%S).log"

# Ensure logs directory exists
mkdir -p "$LOGS_DIR"

# Source test registry
source "$SCRIPT_DIR/test_registry.sh"
source "$SCRIPT_DIR/test_config.sh"
source "$SCRIPT_DIR/test_utils.sh"

# State management functions
save_state() {
    local last_test=$1
    local passed_tests=$2
    local total_tests=$3

    cat > "$STATE_FILE" << EOF
LAST_TEST=$last_test
PASSED_TESTS="$passed_tests"
TOTAL_TESTS=$total_tests
LAST_RUN=$(date)
EOF
}

load_state() {
    if [ -f "$STATE_FILE" ]; then
        source "$STATE_FILE"
        echo -e "${BLUE}📁 Found previous state - Last test: $LAST_TEST, Passed: $PASSED_TESTS${NC}"
        return 0
    else
        echo -e "${YELLOW}📝 No previous state found - starting fresh${NC}"
        return 1
    fi
}

# Logging functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}❌ $1${NC}" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}" | tee -a "$LOG_FILE"
}

# Progress display
show_progress() {
    local current=$1
    local total=$2
    local percentage=$((current * 100 / total))
    local progress_bar=""

    # Create progress bar
    for i in {1..50}; do
        if [ $i -le $((percentage / 2)) ]; then
            progress_bar="${progress_bar}█"
        else
            progress_bar="${progress_bar}░"
        fi
    done

    printf "\r${CYAN}Progress: [${progress_bar}] %d/%d (%d%%)${NC}" $current $total $percentage
}

# Test runner function
run_test() {
    local test_num=$1
    local test_name=$(get_test_info $test_num)
    local test_path=$(get_test_path $test_num)
    local test_desc=$(get_test_description $test_num)

    echo ""
    log_info "🚀 Running Test $test_num: $test_name"
    log_info "   $test_desc"
    log_info "   Path: $test_path"

    local start_time=$(date +%s)

    # Run the test
    if [ -f "$SCRIPT_DIR/$test_path" ]; then
        if (cd "$SCRIPT_DIR/$(dirname "$test_path")" && ./$(basename "$test_path") 2>&1); then
            local end_time=$(date +%s)
            local duration=$((end_time - start_time))
            log_success "Test $test_num PASSED (took ${duration}s)"
            return 0
        else
            local end_time=$(date +%s)
            local duration=$((end_time - start_time))
            log_error "Test $test_num FAILED (took ${duration}s)"
            return 1
        fi
    else
        log_error "Test file not found: $SCRIPT_DIR/$test_path"
        return 1
    fi
}

# Main execution
main() {
    local start_from=1
    local passed_tests=""
    local resume_mode=false

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --resume)
                resume_mode=true
                shift
                ;;
            --start-from)
                start_from="$2"
                shift
                shift
                ;;
            --list)
                list_all_tests
                exit 0
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --resume          Resume from last failed test"
                echo "  --start-from N    Start from test number N"
                echo "  --list           List all available tests"
                echo "  --help           Show this help"
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done

    echo "==========================================" | tee "$LOG_FILE"
    echo "🚀 BACKEND INTEGRATION TEST SUITE" | tee -a "$LOG_FILE"
    echo "==========================================" | tee -a "$LOG_FILE"
    echo "" | tee -a "$LOG_FILE"

    # Load previous state if resuming
    if $resume_mode && load_state; then
        if [ -n "$LAST_TEST" ] && [ "$LAST_TEST" -lt "$TOTAL_TESTS" ]; then
            start_from=$((LAST_TEST + 1))
            passed_tests="$PASSED_TESTS"
            log_info "Resuming from test $start_from (last completed: $LAST_TEST)"
        else
            log_warning "Cannot resume - all tests completed or invalid state"
            start_from=1
        fi
    fi

    log_info "Starting test run from test $start_from to $TOTAL_TESTS"
    log_info "Total tests to run: $((TOTAL_TESTS - start_from + 1))"

    local passed=0
    local failed=0
    local current_test

    # Run tests
    for current_test in $(seq $start_from $TOTAL_TESTS); do
        show_progress $((current_test - 1)) $TOTAL_TESTS

        if run_test $current_test; then
            passed=$((passed + 1))
            passed_tests="${passed_tests}${passed_tests:+,}$current_test"

            # Save state after each successful test
            save_state $current_test "$passed_tests" $TOTAL_TESTS
        else
            failed=$((failed + 1))

            # Save state on failure
            save_state $((current_test - 1)) "$passed_tests" $TOTAL_TESTS

            echo ""
            log_error "Test $current_test failed - stopping execution"
            log_info "To resume later, run: $0 --resume"
            log_info "To start from specific test, run: $0 --start-from $current_test"
            break
        fi
    done

    show_progress $TOTAL_TESTS $TOTAL_TESTS
    echo ""

    # Final summary
    echo ""
    echo "==========================================" | tee -a "$LOG_FILE"
    echo "📊 TEST SUMMARY" | tee -a "$LOG_FILE"
    echo "==========================================" | tee -a "$LOG_FILE"
    echo "" | tee -a "$LOG_FILE"

    if [ $failed -eq 0 ]; then
        log_success "🎉 ALL TESTS PASSED!"
        log_success "Passed: $passed, Failed: $failed"
        log_info "State saved to: $STATE_FILE"
        log_info "Log saved to: $LOG_FILE"
    else
        log_error "❌ SOME TESTS FAILED"
        log_info "Passed: $passed, Failed: $failed"
        log_info "Last completed test: $((current_test - 1))"
        log_info "State saved to: $STATE_FILE"
        log_info "Log saved to: $LOG_FILE"
        log_info "To resume: $0 --resume"
        exit 1
    fi

    echo "" | tee -a "$LOG_FILE"
    log_info "Total tests run: $((passed + failed))"
    log_info "Passed tests: $passed_tests"
}

# Run main function
main "$@"
