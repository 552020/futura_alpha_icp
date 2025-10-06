#!/usr/bin/env node

/**
 * Memory Creation Debug Test
 *
 * Tests the memory creation functionality with simplified metadata
 * to debug the "Variant has no data" error.
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
const TEST_NAME = "Memory Creation Debug Test";
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

// Test function with simplified metadata
async function testMemoryCreationDebug() {
  echoInfo(`🧪 Testing memory creation with simplified metadata`);

  // Create test blobs
  const testData1 = Buffer.from("Test data 1", "utf8");
  const testData2 = Buffer.from("Test data 2", "utf8");

  const blobId1 = await createTestBlob(backend, testData1, "test1");
  const blobId2 = await createTestBlob(backend, testData2, "test2");

  // Create a new capsule for memory creation
  const capsuleResult = await backend.capsules_create([]);
  if ("Err" in capsuleResult) {
    throw new Error(`Capsule creation failed: ${JSON.stringify(capsuleResult.Err)}`);
  }
  const capsuleId = capsuleResult.Ok.id;

  // Create simplified memory metadata
  const memoryMetadata = {
    title: ["Debug Test Memory"], // opt text - wrapped in array for Some(value)
    description: ["Memory creation debug test"], // opt text
    tags: ["test", "debug"],
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

  // Create simplified asset metadata
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
        name: "test-asset",
        storage_key: [],
        tags: ["test", "debug"],
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

  // Create memory with 2 assets
  const allAssets = [
    { blob_id: blobId1, metadata: assetMetadata },
    { blob_id: blobId2, metadata: assetMetadata },
  ];

  echoInfo(`📝 Creating memory with ${allAssets.length} assets...`);
  echoInfo(`  Asset 1: ${blobId1}`);
  echoInfo(`  Asset 2: ${blobId2}`);

  const memoryResult = await backend.memories_create_with_internal_blobs(
    capsuleId, // text - capsule ID
    memoryMetadata, // MemoryMetadata
    allAssets, // Vec<InternalBlobAssetInput> - 2 assets
    `debug-memory-${Date.now()}` // text - idempotency key
  );

  if ("Err" in memoryResult) {
    echoFail(`Memory creation failed: ${JSON.stringify(memoryResult.Err)}`);
    throw new Error(`Memory creation failed: ${JSON.stringify(memoryResult.Err)}`);
  }

  const memoryId = memoryResult.Ok;
  echoInfo(`✅ Memory created successfully: ${memoryId}`);

  // Verify memory was created
  const memoryRead = await backend.memories_read(memoryId);
  if ("Err" in memoryRead) {
    echoFail(`Memory read failed: ${JSON.stringify(memoryRead.Err)}`);
    throw new Error(`Memory read failed: ${JSON.stringify(memoryRead.Err)}`);
  }

  echoInfo(`✅ Memory verified: ${memoryRead.Ok.blob_internal_assets.length} internal assets`);
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
    echoFail("Usage: node test_memory_creation_debug.mjs <CANISTER_ID> [mainnet|local]");
    echoFail("Example: node test_memory_creation_debug.mjs uxrrr-q7777-77774-qaaaq-cai local");
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
    const result = await testMemoryCreationDebug();
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
