#!/bin/bash

# Test Status Summary for Backend Integration Tests
# Shows current progress and status of all tests (dynamic)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_registry.sh"

# Load state file if it exists
STATE_FILE="$SCRIPT_DIR/.test_state"
PASSED_TESTS=""
LAST_TEST=0

if [ -f "$STATE_FILE" ]; then
    source "$STATE_FILE"
    PASSED_TESTS="${PASSED_TESTS:-}"
    LAST_TEST="${LAST_TEST:-0}"
fi

# Count completed tests dynamically
COMPLETED_GENERAL=0
COMPLETED_SHARED=0
COMPLETED_CANISTER=0

# Parse passed tests
IFS=',' read -ra PASSED_ARRAY <<< "$PASSED_TESTS"
for test_num in "${PASSED_ARRAY[@]}"; do
    if [[ $test_num =~ ^[0-9]+$ ]] && [ $test_num -ge 1 ] && [ $test_num -le $TOTAL_TESTS ]; then
        category=$(get_test_category $test_num)
        case $category in
            "general") COMPLETED_GENERAL=$((COMPLETED_GENERAL + 1)) ;;
            "shared-capsule") COMPLETED_SHARED=$((COMPLETED_SHARED + 1)) ;;
            "canister-capsule") COMPLETED_CANISTER=$((COMPLETED_CANISTER + 1)) ;;
        esac
    fi
done

# Calculate totals
TOTAL_COMPLETED=$((COMPLETED_GENERAL + COMPLETED_SHARED + COMPLETED_CANISTER))
PERCENTAGE=$((TOTAL_COMPLETED * 100 / TOTAL_TESTS))

echo "=========================================="
echo "📊 BACKEND INTEGRATION TEST STATUS"
echo "=========================================="
echo ""

echo "✅ COMPLETED TESTS: $TOTAL_COMPLETED/$TOTAL_TESTS ($PERCENTAGE%)"
if [ -n "$PASSED_TESTS" ]; then
    echo "   📝 Passed tests: $PASSED_TESTS"
fi
if [ $LAST_TEST -gt 0 ]; then
    echo "   🎯 Last completed: Test $LAST_TEST"
fi
echo ""

echo "📁 GENERAL TESTS ($COMPLETED_GENERAL/11 completed):"
if [ $COMPLETED_GENERAL -eq 11 ]; then
    echo "   ✅ All general tests completed"
elif [ $COMPLETED_GENERAL -gt 0 ]; then
    echo "   ✅ $COMPLETED_GENERAL tests completed"
fi
if [ $COMPLETED_GENERAL -lt 11 ]; then
    echo "   ❓ Remaining: $(get_test_info $((COMPLETED_GENERAL + 1))) and others"
fi
echo ""

echo "📁 SHARED-CAPSULE TESTS ($COMPLETED_SHARED/12 completed):"
if [ $COMPLETED_SHARED -eq 12 ]; then
    echo "   ✅ All shared-capsule tests completed"
elif [ $COMPLETED_SHARED -gt 0 ]; then
    echo "   ✅ $COMPLETED_SHARED tests completed"
else
    echo "   ❓ Not yet tested"
fi
echo ""

echo "📁 CANISTER-CAPSULE TESTS ($COMPLETED_CANISTER/1 completed):"
if [ $COMPLETED_CANISTER -eq 1 ]; then
    echo "   ✅ Canister-capsule test completed"
else
    echo "   ❓ Not yet tested"
fi
echo ""

echo "🔧 AVAILABLE COMMANDS:"
echo "   ./run_all_tests.sh --list           # List all tests"
echo "   ./run_all_tests.sh --resume         # Resume from last failure"
echo "   ./run_all_tests.sh --start-from N   # Start from test N"
echo "   ./run_all_tests.sh --help           # Show help"
echo ""

echo "📝 STATE FILES:"
echo "   .test_state       - Current progress state"

echo "   logs/test_run_*.log - Detailed execution logs"
echo ""

echo "=========================================="
echo "🎯 NEXT STEPS:"
echo "1. Run: ./run_all_tests.sh --start-from 11"
echo "2. Test shared-capsule tests (12-24)"
echo "3. Test canister-capsule test (25)"
echo "=========================================="
