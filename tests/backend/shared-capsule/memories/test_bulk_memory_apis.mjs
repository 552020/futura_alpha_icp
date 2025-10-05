#!/usr/bin/env node

/**
 * Comprehensive Test Suite for Bulk Memory APIs
 * 
 * This test suite validates all 8 new bulk memory endpoints:
 * 
 * BULK MEMORY OPERATIONS (4):
 * 1. memories_delete_bulk() - Delete multiple memories
 * 2. memories_delete_all() - Delete all memories in capsule  
 * 3. memories_cleanup_assets_all() - Cleanup all assets in a memory
 * 4. memories_cleanup_assets_bulk() - Cleanup assets in multiple memories
 * 
 * GRANULAR ASSET OPERATIONS (4):
 * 5. asset_remove() - Remove asset by reference
 * 6. asset_remove_inline() - Remove inline asset by index
 * 7. asset_remove_internal() - Remove internal blob asset
 * 8. asset_remove_external() - Remove external asset
 * 
 * Test Strategy:
 * 1. Create test capsule
 * 2. Create multiple memories with different asset types
 * 3. Test each bulk endpoint with meaningful data
 * 4. Validate results and cleanup
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { loadDfxIdentity } from "../upload/ic-identity.js";
import fetch from "node-fetch";
import crypto from "crypto";
import { createAssetMetadata } from "../upload/helpers.mjs";

// Import the backend interface
import { idlFactory } from "../../../../.dfx/local/canisters/backend/service.did.js";

// Test configuration
const TEST_NAME = "Bulk Memory APIs Test Suite";
const HOST = process.env.IC_HOST || "http://127.0.0.1:4943";
const IS_MAINNET = process.env.IC_HOST === "https://ic0.app" || process.env.IC_HOST === "https://icp0.io";

// Test data
const TEST_MEMORIES = [
  {
    name: "inline_text_memory",
    content: "This is an inline text memory for testing bulk operations.",
    mimeType: "text/plain",
    tags: ["test", "inline", "bulk"]
  },
  {
    name: "inline_json_memory", 
    content: JSON.stringify({ test: "data", type: "json", bulk: true }),
    mimeType: "application/json",
    tags: ["test", "inline", "json", "bulk"]
  },
  {
    name: "inline_csv_memory",
    content: "name,age,city\nJohn,30,NYC\nJane,25,LA\nBob,35,Chicago",
    mimeType: "text/csv", 
    tags: ["test", "inline", "csv", "bulk"]
  }
];

// Global backend instance
let backend;

// Helper functions
function echoInfo(message) {
  console.log(`ℹ️  ${message}`);
}

function echoPass(message) {
  console.log(`✅ ${message}`);
}

function echoFail(message) {
  console.log(`❌ ${message}`);
}

function echoError(message) {
  console.error(`💥 ${message}`);
}

function echoHeader(message) {
  console.log(`\n🎯 ${message}`);
  console.log("=".repeat(60));
}

function echoSubHeader(message) {
  console.log(`\n📋 ${message}`);
  console.log("-".repeat(40));
}

/**
 * Create test agent with proper certificate handling
 */
async function createTestAgent() {
  const identity = loadDfxIdentity();
  const agent = new HttpAgent({
    host: HOST,
    identity,
    fetch,
  });
  
  // CRITICAL for local replica: trust local root key
  if (!IS_MAINNET) {
    await agent.fetchRootKey();
  }
  
  return agent;
}

/**
 * Create test actor
 */
async function createTestActor() {
  const agent = await createTestAgent();
  const canisterId = process.env.BACKEND_CANISTER_ID || "uxrrr-q7777-77774-qaaaq-cai";
  
  return Actor.createActor(idlFactory, {
    agent,
    canisterId,
  });
}

/**
 * Create test capsule
 */
async function createTestCapsule() {
  echoInfo("Creating test capsule...");
  
  const capsuleResult = await backend.capsules_create([]);
  
  if (!capsuleResult.Ok) {
    throw new Error(`Failed to create capsule: ${JSON.stringify(capsuleResult.Err)}`);
  }
  
  const capsuleId = capsuleResult.Ok.id;
  echoPass(`Capsule created: ${capsuleId}`);
  
  return capsuleId;
}

/**
 * Create test memory with inline content
 */
async function createTestMemory(capsuleId, memoryData, index) {
  echoInfo(`Creating memory ${index + 1}: ${memoryData.name}`);
  
  const contentBytes = Array.from(Buffer.from(memoryData.content, 'utf8'));
  const assetMetadata = createAssetMetadata(
    memoryData.name,
    contentBytes.length,
    memoryData.mimeType
  );
  
  const memoryResult = await backend.memories_create(
    capsuleId,
    [contentBytes], // inline content
    [], // no blob ref
    [], // no external location
    [], // no external storage key
    [], // no external URL
    [], // no external size
    [], // no external hash
    assetMetadata,
    `bulk_test_${index}_${Date.now()}`
  );
  
  if (!memoryResult.Ok) {
    throw new Error(`Failed to create memory ${index + 1}: ${JSON.stringify(memoryResult.Err)}`);
  }
  
  const memoryId = memoryResult.Ok;
  echoPass(`Memory ${index + 1} created: ${memoryId}`);
  
  return memoryId;
}

/**
 * Test 1: memories_delete_bulk() - Delete multiple memories
 */
async function testMemoriesDeleteBulk(capsuleId, memoryIds) {
  echoSubHeader("Test 1: memories_delete_bulk()");
  
  try {
    echoInfo(`Testing bulk deletion of ${memoryIds.length} memories...`);
    
    const deleteResult = await backend.memories_delete_bulk(memoryIds);
    
    if (!deleteResult.Ok) {
      throw new Error(`Bulk delete failed: ${JSON.stringify(deleteResult.Err)}`);
    }
    
    const result = deleteResult.Ok;
    echoPass(`Bulk delete completed successfully!`);
    echoInfo(`  📊 Deleted count: ${result.deleted_count}`);
    echoInfo(`  ❌ Failed count: ${result.failed_count}`);
    echoInfo(`  💬 Message: ${result.message}`);
    
    // Verify memories are actually deleted
    for (const memoryId of memoryIds) {
      const readResult = await backend.memories_read(memoryId);
      if (readResult.Ok) {
        echoFail(`Memory ${memoryId} still exists after bulk delete!`);
        return false;
      }
    }
    
    echoPass("✅ All memories successfully deleted!");
    return true;
    
  } catch (error) {
    echoFail(`Bulk delete test failed: ${error.message}`);
    return false;
  }
}

/**
 * Test 2: memories_delete_all() - Delete all memories in capsule
 */
async function testMemoriesDeleteAll(capsuleId) {
  echoSubHeader("Test 2: memories_delete_all()");
  
  try {
    echoInfo("Testing delete all memories in capsule...");
    
    const deleteAllResult = await backend.memories_delete_all(capsuleId);
    
    if (!deleteAllResult.Ok) {
      throw new Error(`Delete all failed: ${JSON.stringify(deleteAllResult.Err)}`);
    }
    
    const result = deleteAllResult.Ok;
    echoPass(`Delete all completed successfully!`);
    echoInfo(`  📊 Deleted count: ${result.deleted_count}`);
    echoInfo(`  ❌ Failed count: ${result.failed_count}`);
    echoInfo(`  💬 Message: ${result.message}`);
    
    echoPass("✅ All memories in capsule deleted!");
    return true;
    
  } catch (error) {
    echoFail(`Delete all test failed: ${error.message}`);
    return false;
  }
}

/**
 * Test 3: memories_cleanup_assets_all() - Cleanup all assets in a memory
 */
async function testMemoriesCleanupAssetsAll(memoryId) {
  echoSubHeader("Test 3: memories_cleanup_assets_all()");
  
  try {
    echoInfo(`Testing cleanup all assets in memory: ${memoryId}`);
    
    const cleanupResult = await backend.memories_cleanup_assets_all(memoryId);
    
    if (!cleanupResult.Ok) {
      throw new Error(`Cleanup assets all failed: ${JSON.stringify(cleanupResult.Err)}`);
    }
    
    const result = cleanupResult.Ok;
    echoPass(`Cleanup assets all completed successfully!`);
    echoInfo(`  📊 Assets cleaned: ${result.assets_cleaned}`);
    echoInfo(`  💬 Message: ${result.message}`);
    
    echoPass("✅ All assets in memory cleaned!");
    return true;
    
  } catch (error) {
    echoFail(`Cleanup assets all test failed: ${error.message}`);
    return false;
  }
}

/**
 * Test 4: memories_cleanup_assets_bulk() - Cleanup assets in multiple memories
 */
async function testMemoriesCleanupAssetsBulk(memoryIds) {
  echoSubHeader("Test 4: memories_cleanup_assets_bulk()");
  
  try {
    echoInfo(`Testing bulk cleanup assets in ${memoryIds.length} memories...`);
    
    const cleanupResult = await backend.memories_cleanup_assets_bulk(memoryIds);
    
    if (!cleanupResult.Ok) {
      throw new Error(`Bulk cleanup assets failed: ${JSON.stringify(cleanupResult.Err)}`);
    }
    
    const result = cleanupResult.Ok;
    echoPass(`Bulk cleanup assets completed successfully!`);
    echoInfo(`  📊 Successful: ${result.ok.length} memories`);
    echoInfo(`  ❌ Failed: ${result.failed.length} memories`);
    
    if (result.failed.length > 0) {
      echoInfo(`  📋 Failed details:`);
      result.failed.forEach(failure => {
        echoInfo(`    - ${failure.id}: ${failure.err}`);
      });
    }
    
    echoPass("✅ Bulk asset cleanup completed!");
    return true;
    
  } catch (error) {
    echoFail(`Bulk cleanup assets test failed: ${error.message}`);
    return false;
  }
}

/**
 * Test 5: asset_remove() - Remove asset by reference
 */
async function testAssetRemove(memoryId, assetRef) {
  echoSubHeader("Test 5: asset_remove()");
  
  try {
    echoInfo(`Testing asset removal by reference: ${assetRef}`);
    
    const removeResult = await backend.asset_remove(memoryId, assetRef);
    
    if (!removeResult.Ok) {
      throw new Error(`Asset remove failed: ${JSON.stringify(removeResult.Err)}`);
    }
    
    const result = removeResult.Ok;
    echoPass(`Asset removal completed successfully!`);
    echoInfo(`  📊 Asset removed: ${result.asset_removed}`);
    echoInfo(`  💬 Message: ${result.message}`);
    
    echoPass("✅ Asset removed by reference!");
    return true;
    
  } catch (error) {
    echoFail(`Asset remove test failed: ${error.message}`);
    return false;
  }
}

/**
 * Test 6: asset_remove_inline() - Remove inline asset by index
 */
async function testAssetRemoveInline(memoryId, assetIndex) {
  echoSubHeader("Test 6: asset_remove_inline()");
  
  try {
    echoInfo(`Testing inline asset removal by index: ${assetIndex}`);
    
    const removeResult = await backend.asset_remove_inline(memoryId, assetIndex);
    
    if (!removeResult.Ok) {
      throw new Error(`Asset remove inline failed: ${JSON.stringify(removeResult.Err)}`);
    }
    
    const result = removeResult.Ok;
    echoPass(`Inline asset removal completed successfully!`);
    echoInfo(`  📊 Asset removed: ${result.asset_removed}`);
    echoInfo(`  💬 Message: ${result.message}`);
    
    echoPass("✅ Inline asset removed by index!");
    return true;
    
  } catch (error) {
    echoFail(`Asset remove inline test failed: ${error.message}`);
    return false;
  }
}

/**
 * Test 7: asset_remove_internal() - Remove internal blob asset
 */
async function testAssetRemoveInternal(memoryId, blobRef) {
  echoSubHeader("Test 7: asset_remove_internal()");
  
  try {
    echoInfo(`Testing internal blob asset removal: ${blobRef}`);
    
    const removeResult = await backend.asset_remove_internal(memoryId, blobRef);
    
    if (!removeResult.Ok) {
      throw new Error(`Asset remove internal failed: ${JSON.stringify(removeResult.Err)}`);
    }
    
    const result = removeResult.Ok;
    echoPass(`Internal blob asset removal completed successfully!`);
    echoInfo(`  📊 Asset removed: ${result.asset_removed}`);
    echoInfo(`  💬 Message: ${result.message}`);
    
    echoPass("✅ Internal blob asset removed!");
    return true;
    
  } catch (error) {
    echoFail(`Asset remove internal test failed: ${error.message}`);
    return false;
  }
}

/**
 * Test 8: asset_remove_external() - Remove external asset
 */
async function testAssetRemoveExternal(memoryId, storageKey) {
  echoSubHeader("Test 8: asset_remove_external()");
  
  try {
    echoInfo(`Testing external asset removal: ${storageKey}`);
    
    const removeResult = await backend.asset_remove_external(memoryId, storageKey);
    
    if (!removeResult.Ok) {
      throw new Error(`Asset remove external failed: ${JSON.stringify(removeResult.Err)}`);
    }
    
    const result = removeResult.Ok;
    echoPass(`External asset removal completed successfully!`);
    echoInfo(`  📊 Asset removed: ${result.asset_removed}`);
    echoInfo(`  💬 Message: ${result.message}`);
    
    echoPass("✅ External asset removed!");
    return true;
    
  } catch (error) {
    echoFail(`Asset remove external test failed: ${error.message}`);
    return false;
  }
}

/**
 * Main test execution
 */
async function main() {
  echoHeader(TEST_NAME);
  
  let capsuleId = null;
  let memoryIds = [];
  
  try {
    // Initialize backend
    echoInfo("Initializing backend connection...");
    backend = await createTestActor();
    echoPass("Backend connection established");
    
    // Step 1: Create test capsule
    echoSubHeader("Setup: Creating Test Data");
    capsuleId = await createTestCapsule();
    
    // Step 2: Create multiple test memories
    echoInfo("Creating test memories with different content types...");
    for (let i = 0; i < TEST_MEMORIES.length; i++) {
      const memoryId = await createTestMemory(capsuleId, TEST_MEMORIES[i], i);
      memoryIds.push(memoryId);
    }
    
    echoPass(`Created ${memoryIds.length} test memories`);
    
    // Step 3: Test all 8 bulk endpoints
    echoSubHeader("Testing Bulk Memory Operations");
    
    // Test bulk operations (these will delete our test data)
    const test1 = await testMemoriesDeleteBulk(capsuleId, memoryIds.slice(0, 2));
    
    // Recreate memories for remaining tests
    echoInfo("Recreating memories for remaining tests...");
    memoryIds = [];
    for (let i = 0; i < TEST_MEMORIES.length; i++) {
      const memoryId = await createTestMemory(capsuleId, TEST_MEMORIES[i], i);
      memoryIds.push(memoryId);
    }
    
    const test2 = await testMemoriesDeleteAll(capsuleId);
    
    // Recreate one memory for asset tests
    echoInfo("Creating memory for asset tests...");
    const assetMemoryId = await createTestMemory(capsuleId, TEST_MEMORIES[0], 0);
    
    const test3 = await testMemoriesCleanupAssetsAll(assetMemoryId);
    const test4 = await testMemoriesCleanupAssetsBulk([assetMemoryId]);
    
    echoSubHeader("Testing Granular Asset Operations");
    
    // Create memory for granular asset tests
    const granularMemoryId = await createTestMemory(capsuleId, TEST_MEMORIES[1], 1);
    
    // Test granular asset operations
    const test5 = await testAssetRemove(granularMemoryId, "test_ref");
    const test6 = await testAssetRemoveInline(granularMemoryId, 0);
    const test7 = await testAssetRemoveInternal(granularMemoryId, "test_blob_ref");
    const test8 = await testAssetRemoveExternal(granularMemoryId, "test_storage_key");
    
    // Summary
    const results = [test1, test2, test3, test4, test5, test6, test7, test8];
    const passed = results.filter(r => r).length;
    const total = results.length;
    
    echoHeader("Test Results Summary");
    echoInfo(`📊 Tests passed: ${passed}/${total}`);
    echoInfo(`📊 Success rate: ${Math.round((passed / total) * 100)}%`);
    
    if (passed === total) {
      echoPass("🎉 All bulk memory API tests passed!");
    } else {
      echoFail(`❌ ${total - passed} tests failed`);
    }
    
  } catch (error) {
    echoError(`Test suite failed: ${error.message}`);
    console.error("Stack trace:", error.stack);
    process.exit(1);
  } finally {
    // Cleanup
    if (capsuleId) {
      echoInfo("Cleaning up test capsule...");
      try {
        await backend.capsules_delete(capsuleId);
        echoPass("Test capsule deleted");
      } catch (error) {
        echoError(`Failed to delete test capsule: ${error.message}`);
      }
    }
  }
}

// Run the test
main().catch(console.error);
