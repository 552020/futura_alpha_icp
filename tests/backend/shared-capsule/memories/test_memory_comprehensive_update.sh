#!/bin/bash

# Comprehensive Memory Update Test
# Tests updating each individual field of a memory to understand the data model

set -e

# Source test utilities
source "$(dirname "$0")/../../test_utils.sh"

# Configuration
CANISTER_ID="backend"
IDENTITY="default"
DEBUG="${DEBUG:-false}"  # Set DEBUG=true to enable debug output

echo_header "🧪 Testing Comprehensive Memory Updates"

# Helper function to get a capsule ID for testing
get_test_capsule_id() {
    local capsule_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_read_basic "(null)" 2>/dev/null)
    local capsule_id=""
    
    if [[ $capsule_result == *"null"* ]]; then
        [[ "$DEBUG" == "true" ]] && echo_debug "No capsule found, creating one first..."
        local create_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID capsules_create "(null)" 2>/dev/null)
        capsule_id=$(echo "$create_result" | grep -o 'capsule_id = opt "[^"]*"' | sed 's/capsule_id = opt "//' | sed 's/"//')
    else
        capsule_id=$(echo "$capsule_result" | grep -o 'capsule_id = "[^"]*"' | sed 's/capsule_id = "//' | sed 's/"//')
    fi
    
    if [[ -z "$capsule_id" ]]; then
        echo_error "Failed to get capsule ID for testing"
        return 1
    fi
    
    echo "$capsule_id"
}

# Helper function to create a test memory
create_test_memory() {
    local capsule_id="$1"
    local memory_name="$2"
    local memory_description="$3"
    local memory_tags="$4"
    local memory_bytes="$5"
    
    local asset_metadata='(variant {
      Document = record {
        base = record {
          name = "'$memory_name'";
          description = opt "'$memory_description'";
          tags = vec { '$memory_tags' };
          asset_type = variant { Original };
          bytes = 24;
          mime_type = "text/plain";
          sha256 = null;
          width = null;
          height = null;
          url = null;
          storage_key = null;
          bucket = null;
          asset_location = null;
          processing_status = null;
          processing_error = null;
          created_at = 0;
          updated_at = 0;
          deleted_at = null;
        };
        page_count = null;
        document_type = null;
        language = null;
        word_count = null;
      }
    })'
    
    local idem="test_memory_$(date +%s)_$$"
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_create \
        "(\"$capsule_id\", opt $memory_bytes, null, null, null, null, null, null, $asset_metadata, \"$idem\")" 2>/dev/null)
    
    if [[ $result == *"Ok ="* ]]; then
        local memory_id=$(echo "$result" | grep -o 'Ok = "[^"]*"' | sed 's/Ok = "//' | sed 's/"//')
        echo "$memory_id"
        return 0
    else
        echo_error "Failed to create test memory: $result"
        return 1
    fi
}

# Test 1: Update memory name/title
test_update_memory_name() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memory name update..."
    
    local capsule_id=$(get_test_capsule_id)
    local memory_id=$(create_test_memory "$capsule_id" "Original Name" "Original description" '"test"; "name"' 'blob "VGVzdCBtZW1vcnkgZGF0YQ=="')
    
    local update_data='(record {
      name = opt "Updated Memory Name";
      metadata = null;
      access = null;
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ Memory name update successful"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ Memory name update failed: $result"
        return 1
    fi
}

# Test 2: Update memory description
test_update_memory_description() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memory description update..."
    
    local capsule_id=$(get_test_capsule_id)
    local memory_id=$(create_test_memory "$capsule_id" "Test Memory" "Original description" '"test"; "description"' 'blob "VGVzdCBtZW1vcnkgZGF0YQ=="')
    
    local update_data='(record {
      name = null;
      metadata = opt (record {
        memory_type = variant { Document };
        title = opt "Test Memory";
        description = opt "Updated description with more details";
        content_type = "text/plain";
        created_at = 0;
        updated_at = 0;
        uploaded_at = 0;
        date_of_memory = null;
        file_created_at = null;
        parent_folder_id = null;
        tags = vec { "test"; "description" };
        deleted_at = null;
        people_in_memory = null;
        location = null;
        memory_notes = null;
        created_by = null;
        database_storage_edges = vec {};
      });
      access = null;
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ Memory description update successful"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ Memory description update failed: $result"
        return 1
    fi
}

# Test 3: Update memory tags
test_update_memory_tags() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memory tags update..."
    
    local capsule_id=$(get_test_capsule_id)
    local memory_id=$(create_test_memory "$capsule_id" "Test Memory" "Test description" '"original"; "tags"' 'blob "VGVzdCBtZW1vcnkgZGF0YQ=="')
    
    local update_data='(record {
      name = null;
      metadata = opt (record {
        memory_type = variant { Document };
        title = opt "Test Memory";
        description = opt "Test description";
        content_type = "text/plain";
        created_at = 0;
        updated_at = 0;
        uploaded_at = 0;
        date_of_memory = null;
        file_created_at = null;
        parent_folder_id = null;
        tags = vec { "updated"; "tags"; "with"; "more"; "items" };
        deleted_at = null;
        people_in_memory = null;
        location = null;
        memory_notes = null;
        created_by = null;
        database_storage_edges = vec {};
      });
      access = null;
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ Memory tags update successful"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ Memory tags update failed: $result"
        return 1
    fi
}

# Test 4: Update memory type
test_update_memory_type() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memory type update..."
    
    local capsule_id=$(get_test_capsule_id)
    local memory_id=$(create_test_memory "$capsule_id" "Test Memory" "Test description" '"test"; "type"' 'blob "VGVzdCBtZW1vcnkgZGF0YQ=="')
    
    local update_data='(record {
      name = null;
      metadata = opt (record {
        memory_type = variant { Note };
        title = opt "Test Memory";
        description = opt "Test description";
        content_type = "text/plain";
        created_at = 0;
        updated_at = 0;
        uploaded_at = 0;
        date_of_memory = null;
        file_created_at = null;
        parent_folder_id = null;
        tags = vec { "test"; "type" };
        deleted_at = null;
        people_in_memory = null;
        location = null;
        memory_notes = null;
        created_by = null;
        database_storage_edges = vec {};
      });
      access = null;
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ Memory type update successful"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ Memory type update failed: $result"
        return 1
    fi
}

# Test 5: Update memory location
test_update_memory_location() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memory location update..."
    
    local capsule_id=$(get_test_capsule_id)
    local memory_id=$(create_test_memory "$capsule_id" "Test Memory" "Test description" '"test"; "location"' 'blob "VGVzdCBtZW1vcnkgZGF0YQ=="')
    
    local update_data='(record {
      name = null;
      metadata = opt (record {
        memory_type = variant { Document };
        title = opt "Test Memory";
        description = opt "Test description";
        content_type = "text/plain";
        created_at = 0;
        updated_at = 0;
        uploaded_at = 0;
        date_of_memory = null;
        file_created_at = null;
        parent_folder_id = null;
        tags = vec { "test"; "location" };
        deleted_at = null;
        people_in_memory = null;
        location = opt "Paris, France";
        memory_notes = null;
        created_by = null;
        database_storage_edges = vec {};
      });
      access = null;
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ Memory location update successful"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ Memory location update failed: $result"
        return 1
    fi
}

# Test 6: Update memory notes
test_update_memory_notes() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memory notes update..."
    
    local capsule_id=$(get_test_capsule_id)
    local memory_id=$(create_test_memory "$capsule_id" "Test Memory" "Test description" '"test"; "notes"' 'blob "VGVzdCBtZW1vcnkgZGF0YQ=="')
    
    local update_data='(record {
      name = null;
      metadata = opt (record {
        memory_type = variant { Document };
        title = opt "Test Memory";
        description = opt "Test description";
        content_type = "text/plain";
        created_at = 0;
        updated_at = 0;
        uploaded_at = 0;
        date_of_memory = null;
        file_created_at = null;
        parent_folder_id = null;
        tags = vec { "test"; "notes" };
        deleted_at = null;
        people_in_memory = null;
        location = null;
        memory_notes = opt "This is a test note with additional information about the memory";
        created_by = null;
        database_storage_edges = vec {};
      });
      access = null;
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ Memory notes update successful"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ Memory notes update failed: $result"
        return 1
    fi
}

# Test 7: Update memory people
test_update_memory_people() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memory people update..."
    
    local capsule_id=$(get_test_capsule_id)
    local memory_id=$(create_test_memory "$capsule_id" "Test Memory" "Test description" '"test"; "people"' 'blob "VGVzdCBtZW1vcnkgZGF0YQ=="')
    
    local update_data='(record {
      name = null;
      metadata = opt (record {
        memory_type = variant { Document };
        title = opt "Test Memory";
        description = opt "Test description";
        content_type = "text/plain";
        created_at = 0;
        updated_at = 0;
        uploaded_at = 0;
        date_of_memory = null;
        file_created_at = null;
        parent_folder_id = null;
        tags = vec { "test"; "people" };
        deleted_at = null;
        people_in_memory = opt vec { "Alice"; "Bob"; "Charlie" };
        location = null;
        memory_notes = null;
        created_by = null;
        database_storage_edges = vec {};
      });
      access = null;
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ Memory people update successful"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ Memory people update failed: $result"
        return 1
    fi
}

# Test 8: Update memory access from Private to Public
test_update_memory_access_private_to_public() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memory access update (Private to Public)..."
    
    local capsule_id=$(get_test_capsule_id)
    local memory_id=$(create_test_memory "$capsule_id" "Test Memory" "Test description" '"test"; "access"' 'blob "VGVzdCBtZW1vcnkgZGF0YQ=="')
    
    local update_data='(record {
      name = null;
      metadata = null;
      access = opt (variant {
        Public = record {
          owner_secure_code = "new_secure_code_123";
        }
      });
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ Memory access update (Private to Public) successful"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ Memory access update (Private to Public) failed: $result"
        return 1
    fi
}

# Test 9: Update memory access to Custom
test_update_memory_access_to_custom() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memory access update to Custom..."
    
    local capsule_id=$(get_test_capsule_id)
    local memory_id=$(create_test_memory "$capsule_id" "Test Memory" "Test description" '"test"; "custom"' 'blob "VGVzdCBtZW1vcnkgZGF0YQ=="')
    
    local update_data='(record {
      name = null;
      metadata = null;
      access = opt (variant {
        Custom = record {
          individuals = vec {
            variant { Principal = principal "aaaaa-aa" };
            variant { Opaque = "user123" };
          };
          groups = vec { "family"; "friends" };
          owner_secure_code = "custom_secure_code_456";
        }
      });
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ Memory access update to Custom successful"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ Memory access update to Custom failed: $result"
        return 1
    fi
}

# Test 10: Update memory access to Scheduled
test_update_memory_access_to_scheduled() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing memory access update to Scheduled..."
    
    local capsule_id=$(get_test_capsule_id)
    local memory_id=$(create_test_memory "$capsule_id" "Test Memory" "Test description" '"test"; "scheduled"' 'blob "VGVzdCBtZW1vcnkgZGF0YQ=="')
    
    # Schedule to be public after 1 year from now
    local future_time=$(( $(date +%s) + 31536000 ))000000000  # 1 year in nanoseconds
    
    local update_data='(record {
      name = null;
      metadata = null;
      access = opt (variant {
        Scheduled = record {
          accessible_after = '$future_time' : nat64;
          access = variant {
            Public = record {
              owner_secure_code = "scheduled_secure_code_789";
            }
          };
          owner_secure_code = "current_secure_code_789";
        }
      });
    })'
    
    local result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>/dev/null)
    
    if [[ $result == *"success = true"* ]]; then
        echo_success "✅ Memory access update to Scheduled successful"
        [[ "$DEBUG" == "true" ]] && echo_debug "Result: $result"
        return 0
    else
        echo_error "❌ Memory access update to Scheduled failed: $result"
        return 1
    fi
}

# Test 11: Read and display complete memory structure (simplified)
test_read_complete_memory_structure() {
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing complete memory structure read..."
    
    local capsule_id=$(get_test_capsule_id)
    local memory_id=$(create_test_memory "$capsule_id" "Complete Test Memory" "Complete test description" '"complete"; "test"; "structure"' 'blob "VGVzdCBtZW1vcnkgZGF0YQ=="')
    
    # Update with simpler comprehensive data (avoiding complex nested structures)
    local update_data='(record {
      name = opt "Complete Test Memory";
      metadata = opt (record {
        memory_type = variant { Note };
        title = opt "Complete Test Memory";
        description = opt "This is a comprehensive test memory with all fields populated";
        content_type = "text/plain";
        created_at = 0;
        updated_at = 0;
        uploaded_at = 0;
        date_of_memory = opt (1609459200000000000 : nat64);
        file_created_at = opt (1609459200000000000 : nat64);
        parent_folder_id = opt "folder_123";
        tags = vec { "complete"; "test"; "structure"; "comprehensive" };
        deleted_at = null;
        people_in_memory = opt vec { "Alice"; "Bob"; "Charlie"; "Diana" };
        location = opt "San Francisco, CA, USA";
        memory_notes = opt "This is a comprehensive test note with detailed information";
        created_by = opt "test_user_principal";
        database_storage_edges = vec {};
      });
      access = opt (variant {
        Public = record {
          owner_secure_code = "comprehensive_secure_code_999";
        }
      });
    })'
    
    # Test the update command first
    [[ "$DEBUG" == "true" ]] && echo_debug "Testing update command..."
    local update_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_update "(\"$memory_id\", $update_data)" 2>&1)
    
    [[ "$DEBUG" == "true" ]] && echo_debug "Raw update result: '$update_result'"
    
    if [[ $update_result == *"success = true"* ]]; then
        echo_success "✅ Comprehensive memory update successful"
        
        # Now read the complete memory structure
        [[ "$DEBUG" == "true" ]] && echo_debug "Testing read command..."
        local read_result=$(dfx canister call --identity $IDENTITY $CANISTER_ID memories_read "(\"$memory_id\")" 2>&1)
        
        [[ "$DEBUG" == "true" ]] && echo_debug "Raw read result: '$read_result'"
        
        if [[ $read_result == *"Ok ="* ]]; then
            echo_success "✅ Complete memory structure read successful"
            [[ "$DEBUG" == "true" ]] && echo_debug "Complete Memory Structure:"
            [[ "$DEBUG" == "true" ]] && echo "$read_result" | head -50
            return 0
        else
            echo_error "❌ Complete memory structure read failed: $read_result"
            return 1
        fi
    else
        echo_error "❌ Comprehensive memory update failed: $update_result"
        return 1
    fi
}

# Main test execution
main() {
    echo_header "🚀 Starting Comprehensive Memory Update Tests"
    
    run_test "Memory name update" test_update_memory_name
    run_test "Memory description update" test_update_memory_description
    run_test "Memory tags update" test_update_memory_tags
    run_test "Memory type update" test_update_memory_type
    run_test "Memory location update" test_update_memory_location
    run_test "Memory notes update" test_update_memory_notes
    run_test "Memory people update" test_update_memory_people
    run_test "Memory access update (Private to Public)" test_update_memory_access_private_to_public
    run_test "Memory access update to Custom" test_update_memory_access_to_custom
    run_test "Memory access update to Scheduled" test_update_memory_access_to_scheduled
    run_test "Complete memory structure read" test_read_complete_memory_structure
    
    echo_header "🎉 All comprehensive memory update tests completed!"
}

# Run main function
main "$@"
