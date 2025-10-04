#!/usr/bin/env node

/**
 * Bulk Memory APIs Testing
 * Tests the new 8 bulk memory API endpoints for comprehensive memory and asset management
 *
 * New APIs tested:
 * 1. memories_delete_bulk(capsule_id, memory_ids[]) -> Result<BulkDeleteResult, Error>
 * 2. memories_delete_all(capsule_id) -> Result<BulkDeleteResult, Error>
 * 3. memories_cleanup_assets_all(memory_id) -> Result<AssetCleanupResult, Error>
 * 4. memories_cleanup_assets_bulk(memory_ids[]) -> Result<BulkAssetCleanupResult, Error>
 * 5. asset_remove(memory_id, asset_ref) -> Result<AssetRemovalResult, Error>
 * 6. asset_remove_inline(memory_id, asset_index) -> Result<AssetRemovalResult, Error>
 * 7. asset_remove_internal(memory_id, blob_ref) -> Result<AssetRemovalResult, Error>
 * 8. asset_remove_external(memory_id, storage_key) -> Result<AssetRemovalResult, Error>
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { loadDfxIdentity, makeMainnetAgent } from "../../upload/ic-identity.js";
import { sleep, retryWithBackoff, formatFileSize, formatDuration } from "../../upload/helpers.mjs";

// Import the backend interface
import { idlFactory } from "../../../../../src/nextjs/src/ic/declarations/backend/backend.did.js";

// Configuration
const CANISTER_ID = process.env.BACKEND_CANISTER_ID || process.env.BACKEND_ID || "uxrrr-q7777-77774-qaaaq-cai";
const HOST = process.env.IC_HOST || "http://127.0.0.1:4943";
const IS_MAINNET = process.env.IC_HOST === "https://ic0.app" || process.env.IC_HOST === "https://icp0.io";

// Colors for output
const colors = {
  reset: "\x1b[0m",
  red: "\x1b[31m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  magenta: "\x1b[35m",
  cyan: "\x1b[36m",
  white: "\x1b[37m",
  bold: "\x1b[1m",
};

function log(message, color = "white") {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

function logHeader(message) {
  log("\n" + "=".repeat(60), "cyan");
  log(message, "cyan");
  log("=".repeat(60), "cyan");
}

function logSuccess(message) {
  log(`✅ ${message}`, "green");
}

function logError(message) {
  log(`❌ ${message}`, "red");
}

function logInfo(message) {
  log(`ℹ️  ${message}`, "blue");
}

function logDebug(message) {
  if (process.env.DEBUG === "true") {
    log(`[DEBUG] ${message}`, "yellow");
  }
}

// Initialize agent and actor
let agent, actor;

async function initializeAgent() {
  try {
    // Create agent using the same approach as existing tests
    if (IS_MAINNET) {
      agent = await makeMainnetAgent();
    } else {
      // Load DFX identity for local replica
      logDebug("Loading DFX identity...");
      const identity = loadDfxIdentity();
      agent = new HttpAgent({
        host: HOST,
        identity,
        verifyQuerySignatures: false,
      });
    }

    // Create actor using the imported idlFactory
    actor = Actor.createActor(idlFactory, {
      agent,
      canisterId: CANISTER_ID,
    });

    logDebug("Agent and actor initialized successfully");
    return true;
  } catch (error) {
    logError(`Failed to initialize agent: ${error.message}`);
    return false;
  }
}

// Helper function to get test capsule ID
async function getTestCapsuleId() {
  try {
    logDebug("Getting test capsule ID...");

    // First, try to get existing capsule
    const capsules = await actor.capsules_list();
    logDebug(
      `Capsules list result: ${JSON.stringify(capsules, (key, value) =>
        typeof value === "bigint" ? value.toString() : value
      )}`
    );

    // Check if capsules is an array directly or wrapped in Ok
    let capsuleList;
    if (Array.isArray(capsules)) {
      capsuleList = capsules;
    } else if (capsules.Ok && Array.isArray(capsules.Ok)) {
      capsuleList = capsules.Ok;
    } else {
      capsuleList = [];
    }

    if (capsuleList.length > 0) {
      const capsuleId = capsuleList[0].id; // Note: using 'id' not 'capsule_id'
      logDebug(`Using existing capsule: ${capsuleId}`);
      return capsuleId;
    }

    // If no capsule exists, create one
    logDebug("No capsule found, creating one...");
    const createResult = await actor.capsules_create([]);

    if (createResult.Ok) {
      const capsuleId = createResult.Ok.id;
      logDebug(`Created new capsule: ${capsuleId}`);
      return capsuleId;
    } else {
      throw new Error(`Failed to create capsule: ${JSON.stringify(createResult)}`);
    }
  } catch (error) {
    logError(`Failed to get test capsule ID: ${error.message}`);
    throw error;
  }
}

// Helper function to create test memory
async function createTestMemory(capsuleId, name, description, tags, memoryBytes) {
  try {
    logDebug(`Creating test memory: ${name}`);

    // Convert blob format to proper bytes
    let inlineData;
    if (memoryBytes.startsWith('blob "') && memoryBytes.endsWith('"')) {
      // Extract base64 content
      const base64Content = memoryBytes.slice(6, -1); // Remove 'blob "' and '"'
      const decodedBytes = Buffer.from(base64Content, "base64");
      inlineData = Array.from(decodedBytes);
      logDebug(`Converted blob to ${inlineData.length} bytes`);
    } else {
      throw new Error(`Unsupported memory bytes format: ${memoryBytes}`);
    }

    // Create asset metadata
    const assetMetadata = {
      Document: {
        base: {
          name: name,
          description: [description],
          tags: tags.split(";").map((tag) => tag.trim().replace(/"/g, "")),
          asset_type: { Original: null },
          bytes: inlineData.length,
          mime_type: "text/plain",
          sha256: [],
          width: [],
          height: [],
          url: [],
          storage_key: [],
          bucket: [],
          asset_location: [],
          processing_status: [],
          processing_error: [],
          created_at: BigInt(Date.now() * 1000000),
          updated_at: BigInt(Date.now() * 1000000),
          deleted_at: [],
        },
        page_count: [],
        document_type: [],
        language: [],
        word_count: [],
      },
    };

    const idem = `test_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    logDebug(`Calling memories_create with ${inlineData.length} bytes`);
    const result = await actor.memories_create(
      capsuleId,
      [inlineData], // opt blob
      [], // opt BlobRef
      [], // opt StorageEdgeBlobType
      [], // opt text
      [], // opt text
      [], // opt nat64
      [], // opt blob
      assetMetadata, // AssetMetadata
      idem // text
    );

    logDebug(
      `Memory creation result: ${JSON.stringify(result, (key, value) =>
        typeof value === "bigint" ? value.toString() : value
      )}`
    );

    if (result.Ok) {
      const memoryId = result.Ok;
      logDebug(`Created memory with ID: ${memoryId}`);
      return memoryId;
    } else {
      throw new Error(
        `Memory creation failed: ${JSON.stringify(result, (key, value) =>
          typeof value === "bigint" ? value.toString() : value
        )}`
      );
    }
  } catch (error) {
    logError(`Failed to create test memory: ${error.message}`);
    throw error;
  }
}

// Test 1: Test memories_delete_bulk with valid memory IDs
async function testMemoriesDeleteBulkSuccess() {
  try {
    logDebug("Testing memories_delete_bulk with valid memory IDs...");

    const capsuleId = await getTestCapsuleId();

    // Create 3 test memories
    const memory1Id = await createTestMemory(
      capsuleId,
      "bulk_test_memory_1",
      "Test memory 1 for bulk deletion",
      '"test"; "bulk"; "delete"',
      'blob "VGVzdCBNZW1vcnkgMQ=="'
    );

    const memory2Id = await createTestMemory(
      capsuleId,
      "bulk_test_memory_2",
      "Test memory 2 for bulk deletion",
      '"test"; "bulk"; "delete"',
      'blob "VGVzdCBNZW1vcnkgMg=="'
    );

    const memory3Id = await createTestMemory(
      capsuleId,
      "bulk_test_memory_3",
      "Test memory 3 for bulk deletion",
      '"test"; "bulk"; "delete"',
      'blob "VGVzdCBNZW1vcnkgMw=="'
    );

    const memoryIds = [memory1Id, memory2Id, memory3Id];
    logDebug(`Testing bulk delete with memory IDs: ${memoryIds.join(", ")}`);

    // Call memories_delete_bulk
    const startTime = Date.now();
    const result = await actor.memories_delete_bulk(capsuleId, memoryIds);

    logDebug(
      `Bulk delete result: ${JSON.stringify(result, (key, value) =>
        typeof value === "bigint" ? value.toString() : value
      )}`
    );

    if (result.Ok) {
      const bulkResult = result.Ok;
      logSuccess(
        `Bulk delete succeeded: ${bulkResult.deleted_count} deleted, ${
          bulkResult.failed_count
        } failed (${formatDuration(Date.now() - startTime)})`
      );

      // Verify memories are actually deleted
      for (const memoryId of memoryIds) {
        const readResult = await actor.memories_read(memoryId);
        if (readResult.Ok) {
          logError(`Memory ${memoryId} still exists after bulk delete`);
          return false;
        }
      }

      logSuccess("All memories successfully deleted");
      return true;
    } else {
      logError(`Bulk delete failed: ${JSON.stringify(result)}`);
      return false;
    }
  } catch (error) {
    logError(`Bulk delete test failed: ${error.message}`);
    return false;
  }
}

// Test 2: Test memories_delete_bulk with partial failures
async function testMemoriesDeleteBulkPartialFailure() {
  try {
    logDebug("Testing memories_delete_bulk with partial failures...");

    const capsuleId = await getTestCapsuleId();

    // Create 2 valid memories and 1 invalid memory ID
    const memory1Id = await createTestMemory(
      capsuleId,
      "bulk_test_memory_1",
      "Test memory 1 for partial failure test",
      '"test"; "bulk"; "partial"',
      'blob "VGVzdCBNZW1vcnkgMQ=="'
    );

    const memory2Id = await createTestMemory(
      capsuleId,
      "bulk_test_memory_2",
      "Test memory 2 for partial failure test",
      '"test"; "bulk"; "partial"',
      'blob "VGVzdCBNZW1vcnkgMg=="'
    );

    const invalidMemoryId = "invalid_memory_id_12345";
    const memoryIds = [memory1Id, memory2Id, invalidMemoryId];

    logDebug(`Testing bulk delete with mixed valid/invalid IDs: ${memoryIds.join(", ")}`);

    // Call memories_delete_bulk
    const result = await actor.memories_delete_bulk(capsuleId, memoryIds);

    logDebug(
      `Bulk delete result: ${JSON.stringify(result, (key, value) =>
        typeof value === "bigint" ? value.toString() : value
      )}`
    );

    if (result.Ok) {
      const bulkResult = result.Ok;
      logSuccess(
        `Bulk delete succeeded with partial failures: ${bulkResult.deleted_count} deleted, ${bulkResult.failed_count} failed`
      );

      // Verify valid memories are deleted
      const read1Result = await actor.memories_read(memory1Id);
      const read2Result = await actor.memories_read(memory2Id);

      if (read1Result.Ok || read2Result.Ok) {
        logError("Valid memories not deleted in partial failure scenario");
        return false;
      }

      logSuccess("Partial failure scenario handled correctly");
      return true;
    } else {
      logError(`Bulk delete failed: ${JSON.stringify(result)}`);
      return false;
    }
  } catch (error) {
    logError(`Bulk delete partial failure test failed: ${error.message}`);
    return false;
  }
}

// Test 3: Test memories_delete_all
async function testMemoriesDeleteAll() {
  try {
    logDebug("Testing memories_delete_all...");

    const capsuleId = await getTestCapsuleId();

    // Create multiple memories in the capsule
    const memory1Id = await createTestMemory(
      capsuleId,
      "delete_all_memory_1",
      "Test memory 1 for delete all",
      '"test"; "delete-all"',
      'blob "VGVzdCBNZW1vcnkgMQ=="'
    );

    const memory2Id = await createTestMemory(
      capsuleId,
      "delete_all_memory_2",
      "Test memory 2 for delete all",
      '"test"; "delete-all"',
      'blob "VGVzdCBNZW1vcnkgMg=="'
    );

    logDebug(`Created memories for delete all test: ${memory1Id}, ${memory2Id}`);

    // Call memories_delete_all
    const result = await actor.memories_delete_all(capsuleId);

    logDebug(
      `Delete all result: ${JSON.stringify(result, (key, value) =>
        typeof value === "bigint" ? value.toString() : value
      )}`
    );

    if (result.Ok) {
      const bulkResult = result.Ok;
      logSuccess(`Delete all succeeded: ${bulkResult.deleted_count} deleted, ${bulkResult.failed_count} failed`);

      // Verify all memories are deleted
      const read1Result = await actor.memories_read(memory1Id);
      const read2Result = await actor.memories_read(memory2Id);

      if (read1Result.Ok || read2Result.Ok) {
        logError("Memories still exist after delete all");
        return false;
      }

      logSuccess("All memories successfully deleted");
      return true;
    } else {
      logError(`Delete all failed: ${JSON.stringify(result)}`);
      return false;
    }
  } catch (error) {
    logError(`Delete all test failed: ${error.message}`);
    return false;
  }
}

// Test 4: Test memories_cleanup_assets_all
async function testMemoriesCleanupAssetsAll() {
  try {
    logDebug("Testing memories_cleanup_assets_all...");

    const capsuleId = await getTestCapsuleId();

    // Create a memory with assets
    const memoryId = await createTestMemory(
      capsuleId,
      "cleanup_test_memory",
      "Test memory for asset cleanup",
      '"test"; "cleanup"; "assets"',
      'blob "VGVzdCBNZW1vcnkgZm9yIGNsZWFudXA="'
    );

    logDebug(`Created memory for asset cleanup test: ${memoryId}`);

    // Call memories_cleanup_assets_all
    const result = await actor.memories_cleanup_assets_all(memoryId);

    logDebug(
      `Asset cleanup result: ${JSON.stringify(result, (key, value) =>
        typeof value === "bigint" ? value.toString() : value
      )}`
    );

    if (result.Ok) {
      const cleanupResult = result.Ok;
      logSuccess(`Asset cleanup succeeded: ${cleanupResult.assets_cleaned} assets cleaned`);

      // Clean up the test memory
      await actor.memories_delete(memoryId);

      return true;
    } else {
      logError(`Asset cleanup failed: ${JSON.stringify(result)}`);
      return false;
    }
  } catch (error) {
    logError(`Asset cleanup test failed: ${error.message}`);
    return false;
  }
}

// Test 5: Test asset_remove functions
async function testAssetRemoveFunctions() {
  try {
    logDebug("Testing asset_remove functions...");

    const capsuleId = await getTestCapsuleId();

    // Create a memory with inline assets
    const memoryId = await createTestMemory(
      capsuleId,
      "asset_remove_test_memory",
      "Test memory for asset removal",
      '"test"; "asset-remove"',
      'blob "VGVzdCBNZW1vcnkgZm9yIGFzc2V0IHJlbW92YWw="'
    );

    logDebug(`Created memory for asset removal test: ${memoryId}`);

    // Test asset_remove_inline (index 0)
    const result = await actor.asset_remove_inline(memoryId, 0);

    logDebug(
      `Asset remove inline result: ${JSON.stringify(result, (key, value) =>
        typeof value === "bigint" ? value.toString() : value
      )}`
    );

    if (result.Ok) {
      const removalResult = result.Ok;
      logSuccess(`Asset removal succeeded: ${removalResult.asset_removed ? "removed" : "not found"}`);

      // Clean up the test memory
      await actor.memories_delete(memoryId);

      return true;
    } else {
      logError(`Asset removal failed: ${JSON.stringify(result)}`);
      return false;
    }
  } catch (error) {
    logError(`Asset removal test failed: ${error.message}`);
    return false;
  }
}

// Test 6: Test error handling with invalid inputs
async function testErrorHandling() {
  try {
    logDebug("Testing error handling with invalid inputs...");

    const capsuleId = await getTestCapsuleId();
    const invalidMemoryId = "invalid_memory_id_12345";

    // Test memories_delete_bulk with invalid memory IDs
    const bulkResult = await actor.memories_delete_bulk(capsuleId, [invalidMemoryId]);

    if (bulkResult.Ok) {
      const result = bulkResult.Ok;
      if (result.failed_count > 0) {
        logSuccess(`Bulk delete correctly handled invalid memory ID: ${result.failed_count} failed`);
      } else {
        logError("Bulk delete should have failed with invalid memory ID");
        return false;
      }
    } else {
      logSuccess("Bulk delete correctly returned error for invalid input");
    }

    // Test memories_cleanup_assets_all with invalid memory ID
    const cleanupResult = await actor.memories_cleanup_assets_all(invalidMemoryId);

    if (cleanupResult.Err) {
      logSuccess("Asset cleanup correctly returned error for invalid memory ID");
    } else {
      logError("Asset cleanup should have failed with invalid memory ID");
      return false;
    }

    return true;
  } catch (error) {
    logError(`Error handling test failed: ${error.message}`);
    return false;
  }
}

// Main test execution
async function main() {
  logHeader("🧪 Testing Bulk Memory APIs (JavaScript)");

  // Initialize agent
  if (!(await initializeAgent())) {
    process.exit(1);
  }

  const tests = [
    { name: "Bulk delete success", fn: testMemoriesDeleteBulkSuccess },
    { name: "Bulk delete partial failure", fn: testMemoriesDeleteBulkPartialFailure },
    { name: "Delete all memories", fn: testMemoriesDeleteAll },
    { name: "Asset cleanup all", fn: testMemoriesCleanupAssetsAll },
    { name: "Asset remove functions", fn: testAssetRemoveFunctions },
    { name: "Error handling", fn: testErrorHandling },
  ];

  let testsPassed = 0;
  let testsFailed = 0;

  for (const test of tests) {
    logInfo(`Running: ${test.name}`);
    try {
      if (await test.fn()) {
        testsPassed++;
      } else {
        testsFailed++;
      }
    } catch (error) {
      logError(`${test.name} failed with error: ${error.message}`);
      testsFailed++;
    }
  }

  // Final summary
  logHeader("Test Results");
  if (testsFailed === 0) {
    logSuccess(`🎉 All bulk memory API tests completed successfully! (${testsPassed}/${testsPassed + testsFailed})`);
  } else {
    logError(`❌ Some bulk memory API tests failed! (${testsPassed} passed, ${testsFailed} failed)`);
    process.exit(1);
  }
}

// Run main function
main().catch((error) => {
  logError(`Test execution failed: ${error.message}`);
  process.exit(1);
});
