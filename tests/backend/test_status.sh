#!/bin/bash

# How far the last run of run_all_tests.sh got: passed tests per category.
# Reads the registry and the runner's state file; changes nothing.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_registry.sh"

STATE_FILE="$SCRIPT_DIR/.test_state"
PASSED_TESTS=""
LAST_TEST=0
if [ -f "$STATE_FILE" ]; then
    source "$STATE_FILE"
fi

declare -A total passed
for i in $(seq 1 "$TOTAL_TESTS"); do
    category=$(get_test_category "$i")
    total[$category]=$(( ${total[$category]:-0} + 1 ))
done

passed_count=0
IFS=',' read -ra PASSED_ARRAY <<< "${PASSED_TESTS:-}"
for test_num in "${PASSED_ARRAY[@]}"; do
    if is_valid_test "$test_num"; then
        category=$(get_test_category "$test_num")
        passed[$category]=$(( ${passed[$category]:-0} + 1 ))
        passed_count=$((passed_count + 1))
    fi
done

echo "Backend integration tests: $passed_count of $TOTAL_TESTS passed in the last run"
if [ "${LAST_TEST:-0}" -gt 0 ]; then
    echo "Last test run: $LAST_TEST ($(get_test_info "$LAST_TEST"))"
fi
echo ""
for category in $(printf '%s\n' "${!total[@]}" | sort); do
    printf "  %-16s %2d of %2d\n" "$category" "${passed[$category]:-0}" "${total[$category]}"
done
echo ""
echo "Commands:"
echo "  ./run_all_tests.sh --list           list all tests"
echo "  ./run_all_tests.sh --resume         continue after the last failure"
echo "  ./run_all_tests.sh --start-from N   start at test N"
echo ""
echo "State: .test_state. Logs: logs/test_run_*.log (not in git)."
