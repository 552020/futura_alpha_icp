#!/bin/bash

# Test Registry for Backend Integration Tests
# This file defines all available tests with their numbers and metadata

# Test definitions using indexed arrays (compatible with older bash versions)

# Test names (indexed by test number)
TEST_INFO=(
    ""  # 0-index placeholder
    "test_capsules_list.sh"
    "test_capsules_read.sh"
    "test_memories_ping.sh"
    "test_galleries_list.sh"
    "test_galleries_create.sh"
    "test_galleries_delete.sh"
    "test_galleries_update.sh"
    "test_capsules_create.sh"
    "test_capsules_bind_neon.sh"
    "test_store_gallery_forever_with_memories.sh"
    "test_sync_gallery_memories.sh"
    "test_gallery_crud.sh"
    "test_gallery_upload.sh"
    "test_uuid_mapping.sh"
    "test_authorization.sh"
    "test_chunked_upload.sh"
    "test_memories_advanced.sh"
    "test_memories_create.sh"
    "test_memories_delete.sh"
    "test_memories_list.sh"
    "test_memories_read.sh"
    "test_memories_update.sh"
    "test_memory_crud.sh"
    "test_shared_capsule.sh"
    "test_canister_capsule.sh"
)

# Test paths (indexed by test number)
TEST_PATHS=(
    ""  # 0-index placeholder
    "general/test_capsules_list.sh"
    "general/test_capsules_read.sh"
    "general/test_memories_ping.sh"
    "general/test_galleries_list.sh"
    "general/test_galleries_create.sh"
    "general/test_galleries_delete.sh"
    "general/test_galleries_update.sh"
    "general/test_capsules_create.sh"
    "general/test_capsules_bind_neon.sh"
    "general/test_store_gallery_forever_with_memories.sh"
    "general/test_sync_gallery_memories.sh"
    "shared-capsule/galleries/test_gallery_crud.sh"
    "shared-capsule/galleries/test_gallery_upload.sh"
    "shared-capsule/galleries/test_uuid_mapping.sh"
    "shared-capsule/memories/test_authorization.sh"
    "shared-capsule/memories/test_chunked_upload.sh"
    "shared-capsule/memories/test_memories_advanced.sh"
    "shared-capsule/memories/test_memories_create.sh"
    "shared-capsule/memories/test_memories_delete.sh"
    "shared-capsule/memories/test_memories_list.sh"
    "shared-capsule/memories/test_memories_read.sh"
    "shared-capsule/memories/test_memories_update.sh"
    "shared-capsule/memories/test_memory_crud.sh"
    "shared-capsule/test_shared_capsule.sh"
    "canister-capsule/test_canister_capsule.sh"
)

# Test descriptions (indexed by test number)
TEST_DESCRIPTIONS=(
    ""  # 0-index placeholder
    "Test capsules_list endpoint functionality"
    "Test capsules_read_basic and capsules_read_full"
    "Test memories_ping endpoint (memory presence)"
    "Test galleries_list endpoint"
    "Test galleries_create endpoint"
    "Test galleries_delete endpoint"
    "Test galleries_update endpoint"
    "Test capsules_create endpoint"
    "Test capsules_bind_neon endpoint"
    "Test store_gallery_forever_with_memories (legacy)"
    "Test sync_gallery_memories endpoint"
    "Test gallery CRUD operations"
    "Test gallery upload functionality"
    "Test UUID mapping for galleries"
    "Test memory authorization"
    "Test chunked memory upload"
    "Test advanced memory operations"
    "Test memory creation"
    "Test memory deletion"
    "Test memory listing"
    "Test memory reading"
    "Test memory updating"
    "Test memory CRUD operations"
    "Test shared capsule functionality"
    "Test canister-capsule integration"
)

# Test categories (indexed by test number)
TEST_CATEGORIES=(
    ""  # 0-index placeholder
    "general"
    "general"
    "general"
    "general"
    "general"
    "general"
    "general"
    "general"
    "general"
    "general"
    "general"
    "shared-capsule"
    "shared-capsule"
    "shared-capsule"
    "shared-capsule"
    "shared-capsule"
    "shared-capsule"
    "shared-capsule"
    "shared-capsule"
    "shared-capsule"
    "shared-capsule"
    "shared-capsule"
    "shared-capsule"
    "shared-capsule"
    "canister-capsule"
)

# Total number of tests
TOTAL_TESTS=25

# Function to get test info
get_test_info() {
    local test_num=$1
    echo "${TEST_INFO[$test_num]}"
}

get_test_path() {
    local test_num=$1
    echo "${TEST_PATHS[$test_num]}"
}

get_test_description() {
    local test_num=$1
    echo "${TEST_DESCRIPTIONS[$test_num]}"
}

get_test_category() {
    local test_num=$1
    echo "${TEST_CATEGORIES[$test_num]}"
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
export -f get_test_info
export -f get_test_path
export -f get_test_description
export -f get_test_category
export -f list_all_tests
export -f is_valid_test
export TOTAL_TESTS
