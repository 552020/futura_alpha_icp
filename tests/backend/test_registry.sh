#!/bin/bash

# Test registry for the backend integration tests (shell, against a replica).
# One row per test: "category|path under tests/backend|description".
# The row number is the test number the runner uses, so add new tests at the end.

TESTS=(
    ""  # test numbers start at 1
    "general|general/test_capsules_list.sh|Test capsules_list endpoint functionality"
    "general|general/test_capsules_read.sh|Test capsules_read_basic and capsules_read_full"
    "shared-capsule|shared-capsule/memories/test_memories_ping.sh|Test memories_ping endpoint (memory presence)"
    "shared-capsule|shared-capsule/galleries/test_galleries_list.sh|Test galleries_list endpoint"
    "shared-capsule|shared-capsule/galleries/test_galleries_create.sh|Test galleries_create endpoint"
    "shared-capsule|shared-capsule/galleries/test_galleries_delete.sh|Test galleries_delete endpoint"
    "shared-capsule|shared-capsule/galleries/test_galleries_update.sh|Test galleries_update endpoint"
    "general|general/test_capsules_create.sh|Test capsules_create endpoint"
    "general|general/test_capsules_bind_neon.sh|Test capsules_bind_neon endpoint"
    "shared-capsule|shared-capsule/galleries/test_gallery_crud.sh|Test gallery CRUD operations"
    "shared-capsule|shared-capsule/galleries/test_gallery_upload.sh|Test gallery upload functionality"
    "shared-capsule|shared-capsule/auth/test_authorization.sh|Test memory authorization"
    "shared-capsule|shared-capsule/memories/test_memories_advanced.sh|Test advanced memory operations"
    "shared-capsule|shared-capsule/memories/test_memories_create.sh|Test memory creation"
    "shared-capsule|shared-capsule/memories/test_memories_delete.sh|Test memory deletion"
    "shared-capsule|shared-capsule/memories/test_memories_list.sh|Test memory listing"
    "shared-capsule|shared-capsule/memories/test_memories_read.sh|Test memory reading"
    "shared-capsule|shared-capsule/memories/test_memories_update.sh|Test memory updating"
    "legacy|legacy/test_legacy_inheritance.sh|Test inheritance: trustees, death confirmation, heir access"
    "general|general/test_upgrade_keeps_files.sh|Test that an upgrade keeps uploaded files"
    "personal|personal/test_personal_canister.sh|Test personal canisters: factory creates, hands off, owner upgrades"
    "personal|personal/test_top_up.sh|Test cycles top-up of personal canisters: floor, monthly cap"
)

# Total number of tests
TOTAL_TESTS=$((${#TESTS[@]} - 1))

# Function to get test info
# One field of a test's row: 1 category, 2 path, 3 description
test_field() {
    echo "${TESTS[$1]}" | cut -d'|' -f"$2"
}

# The script's file name
get_test_info() {
    basename "$(test_field "$1" 2)"
}

get_test_path() {
    test_field "$1" 2
}

get_test_description() {
    test_field "$1" 3
}

get_test_category() {
    test_field "$1" 1
}

# Function to list all tests
list_all_tests() {
    echo "=========================================="
    echo "📋 ALL AVAILABLE TESTS"
    echo "=========================================="
    echo ""

    for i in $(seq 1 $TOTAL_TESTS); do
        local name=$(get_test_info $i)
        local desc=$(get_test_description $i)
        local category=$(get_test_category $i)
        printf "%2d. %-40s [%s]\n" $i "$name" "$category"
        echo "    $desc"
        echo ""
    done

    echo "=========================================="
    echo "Total: $TOTAL_TESTS tests"
    echo "=========================================="
}

# Function to validate test number
is_valid_test() {
    local test_num=$1
    [[ $test_num =~ ^[0-9]+$ ]] && [ $test_num -ge 1 ] && [ $test_num -le $TOTAL_TESTS ]
}

# Export functions for use in other scripts
export -f test_field
export -f get_test_info
export -f get_test_path
export -f get_test_description
export -f get_test_category
export -f list_all_tests
export -f is_valid_test
export TOTAL_TESTS
