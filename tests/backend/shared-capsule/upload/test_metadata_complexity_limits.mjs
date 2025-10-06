#!/usr/bin/env node

/**
 * Metadata Complexity Limits Test
 *
 * This test demonstrates the exact point where AssetMetadata
 * becomes too complex for Candid serialization.
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity, makeMainnetAgent } from "./ic-identity.js";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { idlFactory } from "../../../../src/nextjs/src/ic/declarations/backend/backend.did.js";
import {
  validateFileSize,
  validateImageType,
  calculateFileHash,
  generateFileId,
  calculateDerivativeDimensions,
  calculateDerivativeSizes,
  createFileChunks,
  createProgressCallback,
  createAssetMetadata,
  createBlobReference,
  handleUploadError,
  validateUploadResponse,
  formatFileSize,
  formatUploadSpeed,
  formatDuration,
} from "./helpers.mjs";

// Test configuration
const TEST_NAME = "Metadata Complexity Limits Test";
const CHUNK_SIZE = 1_800_000; // 1.8MB - matches backend CHUNK_SIZE in types.rs

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

// Create a simple blob for testing
async function createTestBlob(backend, data, name) {
  echoInfo(`📤 Creating test blob: ${name} (${data.length} bytes)`);

  // Create a new capsule for this test
  const capsuleResult = await backend.capsules_create([]);
  if ("Err" in capsuleResult) {
    throw new Error(`Capsule creation failed: ${JSON.stringify(capsuleResult.Err)}`);
  }
  const capsuleId = capsuleResult.Ok.id;

  // Calculate chunk count and create chunks
  const chunkCount = Math.ceil(data.length / CHUNK_SIZE);
  const chunks = createFileChunks(data, CHUNK_SIZE);
  const idempotencyKey = generateFileId("test");

  // Begin upload session
  const beginResult = await backend.uploads_begin(capsuleId, chunkCount, idempotencyKey);
  let sessionId;
  if (typeof beginResult === "number" || typeof beginResult === "string") {
    sessionId = beginResult;
  } else if (beginResult && typeof beginResult === "object") {
    sessionId = beginResult.Ok;
  } else {
    throw new Error(`Unexpected response format: ${typeof beginResult}`);
  }

  // Upload chunks
  for (let i = 0; i < chunks.length; i++) {
    await backend.uploads_put_chunk(sessionId, i, chunks[i]);
  }

  // Finish upload
  const hash = calculateFileHash(data);
  const totalLen = BigInt(data.length);
  const finishResult = await backend.uploads_finish(sessionId, Array.from(hash), totalLen);

  let blobId;
  if (typeof finishResult === "string") {
    blobId = finishResult;
  } else if (finishResult && typeof finishResult === "object") {
    const result = finishResult.Ok;
    if (result && typeof result === "object" && "blob_id" in result) {
      blobId = result.blob_id;
    } else {
      blobId = result;
    }
  } else {
    throw new Error(`Unexpected finish response format: ${typeof finishResult}`);
  }

  echoInfo(`✅ Test blob created: ${blobId}`);
  return blobId;
}

// Test with minimal metadata (should work)
async function testMinimalMetadata() {
  echoInfo(`🧪 Testing with minimal metadata (should work)`);

  const testData = Buffer.from("Test data", "utf8");
  const blobId = await createTestBlob(backend, testData, "minimal");

  const capsuleResult = await backend.capsules_create([]);
  if ("Err" in capsuleResult) {
    throw new Error(`Capsule creation failed: ${JSON.stringify(capsuleResult.Err)}`);
  }
  const capsuleId = capsuleResult.Ok.id;

  // Minimal memory metadata
  const memoryMetadata = {
    title: ["Minimal Test"],
    description: ["Minimal metadata test"],
    tags: ["test", "minimal"],
    created_at: BigInt(Date.now() * 1000000),
    updated_at: BigInt(Date.now() * 1000000),
    date_of_memory: [],
    memory_type: { Image: null },
    content_type: "text/plain",
    people_in_memory: [],
    database_storage_edges: [],
    created_by: [],
    parent_folder_id: [],
    deleted_at: [],
    file_created_at: [],
    location: [],
    memory_notes: [],
    uploaded_at: BigInt(Date.now() * 1000000),
  };

  // Minimal asset metadata
  const assetMetadata = {
    Image: {
      dpi: [],
      color_space: [],
      base: {
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000),
        asset_type: { Original: null },
        sha256: [],
        name: "minimal-asset",
        storage_key: [],
        tags: ["test", "minimal"],
        processing_error: [],
        mime_type: "text/plain",
        description: [],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(0),
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      exif_data: [],
      compression_ratio: [],
      orientation: [],
    },
  };

  const allAssets = [{ blob_id: blobId, metadata: assetMetadata }];

  const memoryResult = await backend.memories_create_with_internal_blobs(
    capsuleId,
    memoryMetadata,
    allAssets,
    `minimal-test-${Date.now()}`
  );

  if ("Err" in memoryResult) {
    echoFail(`Minimal metadata failed: ${JSON.stringify(memoryResult.Err)}`);
    return false;
  }

  echoPass(`Minimal metadata worked: ${memoryResult.Ok}`);
  return true;
}

// Test with complex metadata (should fail)
async function testComplexMetadata() {
  echoInfo(`🧪 Testing with complex metadata (should fail)`);

  const testData = Buffer.from("Test data", "utf8");
  const blobId = await createTestBlob(backend, testData, "complex");

  const capsuleResult = await backend.capsules_create([]);
  if ("Err" in capsuleResult) {
    throw new Error(`Capsule creation failed: ${JSON.stringify(capsuleResult.Err)}`);
  }
  const capsuleId = capsuleResult.Ok.id;

  // Complex memory metadata
  const memoryMetadata = {
    title: ["Complex Test Memory with Very Long Title"],
    description: [
      "Complex metadata test with very long description that contains lots of information about the memory",
    ],
    tags: ["test", "complex", "metadata", "serialization", "candid", "limitation", "frontend", "production"],
    created_at: BigInt(Date.now() * 1000000),
    updated_at: BigInt(Date.now() * 1000000),
    date_of_memory: [BigInt(Date.now() * 1000000)],
    memory_type: { Image: null },
    content_type: "image/jpeg",
    people_in_memory: [["person1", "person2", "person3"]],
    database_storage_edges: [],
    created_by: ["user123"],
    parent_folder_id: ["folder456"],
    deleted_at: [],
    file_created_at: [BigInt(Date.now() * 1000000)],
    location: ["San Francisco, CA, USA"],
    memory_notes: ["This is a complex memory with lots of metadata for testing serialization limits"],
    uploaded_at: BigInt(Date.now() * 1000000),
  };

  // Complex asset metadata with all fields populated
  const assetMetadata = {
    Image: {
      dpi: [300],
      color_space: ["sRGB"],
      base: {
        url: ["https://example.com/image.jpg"],
        height: [1080],
        updated_at: BigInt(Date.now() * 1000000),
        asset_type: { Original: null },
        sha256: [Array.from(crypto.createHash("sha256").update("test").digest())],
        name: "complex-asset-with-very-long-name",
        storage_key: ["complex-storage-key-12345"],
        tags: [
          "test",
          "complex",
          "metadata",
          "serialization",
          "candid",
          "limitation",
          "frontend",
          "production",
          "image",
          "jpeg",
        ],
        processing_error: [],
        mime_type: "image/jpeg",
        description: ["Complex asset with lots of metadata for testing serialization limits"],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(1024000),
        asset_location: ["icp"],
        width: [1920],
        processing_status: ["completed"],
        bucket: ["main-bucket"],
      },
      exif_data: ["Complex EXIF data with lots of information about the image"],
      compression_ratio: [0.85],
      orientation: [1],
    },
  };

  const allAssets = [{ blob_id: blobId, metadata: assetMetadata }];

  try {
    const memoryResult = await backend.memories_create_with_internal_blobs(
      capsuleId,
      memoryMetadata,
      allAssets,
      `complex-test-${Date.now()}`
    );

    if ("Err" in memoryResult) {
      echoFail(`Complex metadata failed as expected: ${JSON.stringify(memoryResult.Err)}`);
      return true; // Expected to fail
    }

    echoInfo(`Complex metadata unexpectedly worked: ${memoryResult.Ok}`);
    return false; // Unexpected success
  } catch (error) {
    echoFail(`Complex metadata failed with error: ${error.message}`);
    return true; // Expected to fail
  }
}

// Main test function
async function testMetadataComplexityLimits() {
  echoInfo(`🧪 Testing metadata complexity limits`);

  // Test 1: Minimal metadata (should work)
  const minimalResult = await testMinimalMetadata();
  if (!minimalResult) {
    throw new Error("Minimal metadata test failed");
  }

  // Test 2: Complex metadata (should fail)
  const complexResult = await testComplexMetadata();
  if (!complexResult) {
    throw new Error("Complex metadata test unexpectedly succeeded");
  }

  echoPass("Metadata complexity limits test completed successfully");
  return true;
}

// Main test runner
async function main() {
  echoInfo(`Starting ${TEST_NAME}`);

  // Parse command line arguments
  const args = process.argv.slice(2);
  const backendCanisterId = args[0];
  const network = args[1] || "local"; // Default to local network

  if (!backendCanisterId) {
    echoFail("Usage: node test_metadata_complexity_limits.mjs <CANISTER_ID> [mainnet|local]");
    echoFail("Example: node test_metadata_complexity_limits.mjs uxrrr-q7777-77774-qaaaq-cai local");
    process.exit(1);
  }

  // Setup agent and backend based on network
  const identity = loadDfxIdentity();
  let agent;

  if (network === "mainnet") {
    echoInfo(`🌐 Connecting to mainnet (ic0.app)`);
    agent = makeMainnetAgent(identity);
  } else {
    echoInfo(`🏠 Connecting to local network (127.0.0.1:4943)`);
    agent = new HttpAgent({
      host: "http://127.0.0.1:4943",
      identity,
      fetch: (await import("node-fetch")).default,
    });
  }

  await agent.fetchRootKey();

  backend = Actor.createActor(idlFactory, {
    agent,
    canisterId: backendCanisterId,
  });

  // Run test
  try {
    echoInfo(`Running: ${TEST_NAME}`);
    const result = await testMetadataComplexityLimits();
    if (result) {
      echoPass(TEST_NAME);
    } else {
      echoFail(TEST_NAME);
      process.exit(1);
    }
  } catch (error) {
    echoFail(`${TEST_NAME}: ${error.message}`);
    process.exit(1);
  }

  echoPass("Test completed successfully! ✅");
}

// Run the test
main().catch((error) => {
  echoFail(`Test execution failed: ${error.message}`);
  process.exit(1);
});
