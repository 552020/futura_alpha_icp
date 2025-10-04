/**
 * Helper functions for bulk memory API testing
 * Provides utilities for creating test data, validating results, and managing test state
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity, makeMainnetAgent } from "../../upload/ic-identity.js";
import { sleep, retryWithBackoff, formatFileSize, formatDuration } from "../../upload/helpers.mjs";
import { idlFactory } from "../../../../../.dfx/local/canisters/backend/service.did.js";

// Configuration
const CANISTER_ID = process.env.BACKEND_CANISTER_ID || process.env.BACKEND_ID || "uxrrr-q7777-77774-qaaaq-cai";
const HOST = process.env.IC_HOST || "http://127.0.0.1:4943";
const IS_MAINNET = process.env.IC_HOST === "https://ic0.app" || process.env.IC_HOST === "https://icp0.io";

/**
 * Initialize agent and actor for testing
 */
export async function initializeTestEnvironment() {
  try {
    let agent;

    if (IS_MAINNET) {
      agent = await makeMainnetAgent();
    } else {
      const identity = loadDfxIdentity();
      agent = new HttpAgent({
        host: HOST,
        identity,
        verifyQuerySignatures: false,
      });
    }

    const actor = Actor.createActor(idlFactory, {
      agent,
      canisterId: CANISTER_ID,
    });

    return { agent, actor };
  } catch (error) {
    throw new Error(`Failed to initialize test environment: ${error.message}`);
  }
}

/**
 * Get or create a test capsule
 */
export async function getOrCreateTestCapsule(actor) {
  try {
    // First, try to get existing capsules
    const capsules = await actor.capsules_list();

    let capsuleList;
    if (Array.isArray(capsules)) {
      capsuleList = capsules;
    } else if (capsules.Ok && Array.isArray(capsules.Ok)) {
      capsuleList = capsules.Ok;
    } else {
      capsuleList = [];
    }

    if (capsuleList.length > 0) {
      return capsuleList[0].id;
    }

    // Create a new capsule if none exists
    const createResult = await actor.capsules_create([]);

    if (createResult.Ok) {
      return createResult.Ok.id;
    } else {
      throw new Error(`Failed to create capsule: ${JSON.stringify(createResult)}`);
    }
  } catch (error) {
    throw new Error(`Failed to get or create test capsule: ${error.message}`);
  }
}

/**
 * Create a test memory with specified parameters
 */
export async function createTestMemory(actor, capsuleId, name, description, tags, memoryBytes) {
  try {
    // Convert blob format to proper bytes
    let inlineData;
    if (memoryBytes.startsWith('blob "') && memoryBytes.endsWith('"')) {
      const base64Content = memoryBytes.slice(6, -1);
      const decodedBytes = Buffer.from(base64Content, "base64");
      inlineData = Array.from(decodedBytes);
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

    if (result.Ok) {
      return result.Ok;
    } else {
      throw new Error(`Memory creation failed: ${JSON.stringify(result)}`);
    }
  } catch (error) {
    throw new Error(`Failed to create test memory: ${error.message}`);
  }
}

/**
 * Create multiple test memories for bulk operations
 */
export async function createTestMemoriesBatch(actor, capsuleId, count, prefix = "bulk_test") {
  const memoryIds = [];

  for (let i = 1; i <= count; i++) {
    const memoryId = await createTestMemory(
      actor,
      capsuleId,
      `${prefix}_memory_${i}`,
      `Test memory ${i} for bulk operations`,
      '"test"; "bulk"',
      `blob "VGVzdCBNZW1vcnkg${i}=="`
    );
    memoryIds.push(memoryId);
  }

  return memoryIds;
}

/**
 * Verify that memories are deleted
 */
export async function verifyMemoriesDeleted(actor, memoryIds) {
  for (const memoryId of memoryIds) {
    const readResult = await actor.memories_read(memoryId);
    if (readResult.Ok) {
      return false; // Memory still exists
    }
  }
  return true; // All memories are deleted
}

/**
 * Verify that memories exist
 */
export async function verifyMemoriesExist(actor, memoryIds) {
  for (const memoryId of memoryIds) {
    const readResult = await actor.memories_read(memoryId);
    if (!readResult.Ok) {
      return false; // Memory doesn't exist
    }
  }
  return true; // All memories exist
}

/**
 * Clean up test memories
 */
export async function cleanupTestMemories(actor, memoryIds) {
  for (const memoryId of memoryIds) {
    try {
      await actor.memories_delete(memoryId);
    } catch (error) {
      // Ignore cleanup errors
      console.warn(`Failed to cleanup memory ${memoryId}: ${error.message}`);
    }
  }
}

/**
 * Validate bulk delete result
 */
export function validateBulkDeleteResult(result, expectedDeleted, expectedFailed = 0) {
  if (!result.Ok) {
    return { valid: false, error: `Expected Ok result, got: ${JSON.stringify(result)}` };
  }

  const bulkResult = result.Ok;

  if (bulkResult.deleted_count !== expectedDeleted) {
    return {
      valid: false,
      error: `Expected ${expectedDeleted} deleted, got ${bulkResult.deleted_count}`,
    };
  }

  if (bulkResult.failed_count !== expectedFailed) {
    return {
      valid: false,
      error: `Expected ${expectedFailed} failed, got ${bulkResult.failed_count}`,
    };
  }

  return { valid: true };
}

/**
 * Validate asset cleanup result
 */
export function validateAssetCleanupResult(result, expectedCleaned) {
  if (!result.Ok) {
    return { valid: false, error: `Expected Ok result, got: ${JSON.stringify(result)}` };
  }

  const cleanupResult = result.Ok;

  if (cleanupResult.assets_cleaned !== expectedCleaned) {
    return {
      valid: false,
      error: `Expected ${expectedCleaned} assets cleaned, got ${cleanupResult.assets_cleaned}`,
    };
  }

  return { valid: true };
}

/**
 * Validate asset removal result
 */
export function validateAssetRemovalResult(result, expectedRemoved) {
  if (!result.Ok) {
    return { valid: false, error: `Expected Ok result, got: ${JSON.stringify(result)}` };
  }

  const removalResult = result.Ok;

  if (removalResult.asset_removed !== expectedRemoved) {
    return {
      valid: false,
      error: `Expected asset_removed=${expectedRemoved}, got ${removalResult.asset_removed}`,
    };
  }

  return { valid: true };
}

/**
 * Create test data for different asset types
 */
export function createTestAssetData(assetType, size = 100) {
  const testData = {
    inline: `blob "${Buffer.from("A".repeat(size)).toString("base64")}"`,
    internal: {
      blobRef: "test_blob_ref_123",
      hash: Buffer.from("test_hash_123").toString("base64"),
    },
    external: {
      storageKey: "s3://bucket/test_file.jpg",
      url: "https://s3.amazonaws.com/bucket/test_file.jpg",
      size: size,
    },
  };

  return testData[assetType] || testData.inline;
}

/**
 * Generate test memory IDs for testing
 */
export function generateTestMemoryIds(count, prefix = "test_memory") {
  return Array.from({ length: count }, (_, i) => `${prefix}_${i + 1}_${Date.now()}`);
}

// Utility functions are now imported from ../../upload/helpers.mjs
