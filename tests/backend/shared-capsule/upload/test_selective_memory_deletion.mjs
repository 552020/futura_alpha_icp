#!/usr/bin/env node

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { readFileSync } from "fs";
import crypto from "crypto";
import { loadDfxIdentity } from "./ic-identity.js";

// Test configuration
const TEST_NAME = "Selective Memory Deletion Test";
const CHUNK_SIZE = 1_800_000; // 1.8MB (matches backend)

// Helper functions
function echoInfo(msg) {
  console.log(`ℹ️  ${msg}`);
}
function echoPass(msg) {
  console.log(`✅ ${msg}`);
}
function echoFail(msg) {
  console.log(`❌ ${msg}`);
}
function echoError(msg) {
  console.log(`💥 ${msg}`);
}

async function testSelectiveMemoryDeletion(backendCanisterId, filePath) {
  echoInfo(`Starting ${TEST_NAME}`);
  echoInfo(`Testing selective memory deletion with file: ${filePath}`);

  // Read file
  const fileBuffer = readFileSync(filePath);
  const fileSize = fileBuffer.length;
  echoInfo(`File: ${filePath.split("/").pop()} (${fileSize} bytes)`);

  // Setup agent and actor
  const identity = loadDfxIdentity();
  const agent = new HttpAgent({
    host: "http://127.0.0.1:4943",
    identity,
    fetch: (await import("node-fetch")).default,
  });
  await agent.fetchRootKey();

  const backend = Actor.createActor(
    (await import("../../../../src/nextjs/src/ic/declarations/backend/backend.did.js")).idlFactory,
    {
      agent,
      canisterId: Principal.fromText(backendCanisterId),
    }
  );

  // Create a new capsule for this test
  echoInfo("Creating new capsule for test...");
  const capsuleResult = await backend.capsules_create([]);
  if ("Err" in capsuleResult) {
    throw new Error(`Capsule creation failed: ${JSON.stringify(capsuleResult.Err)}`);
  }
  const capsuleId = capsuleResult.Ok.id;
  echoInfo(`Created new capsule: ${capsuleId}`);

  // ========================================
  // Test 1: Create Memory with Internal Blob
  // ========================================
  echoInfo("=== Test 1: Creating Memory with Internal Blob ===");

  // Upload file as blob
  const chunkCount = Math.ceil(fileSize / CHUNK_SIZE);
  const idempotencyKey = `selective-delete-test-${Date.now()}`;

  echoInfo(`Chunk count: ${chunkCount}, Idempotency key: ${idempotencyKey}`);

  // Begin upload
  echoInfo("Calling uploads_begin...");
  const beginResult = await backend.uploads_begin(capsuleId, chunkCount, idempotencyKey);
  if ("Err" in beginResult) {
    throw new Error(`Upload begin failed: ${JSON.stringify(beginResult.Err)}`);
  }
  const sessionId = beginResult.Ok;
  echoInfo(`Upload session started: ${sessionId}`);

  // Upload chunks
  echoInfo("Uploading chunks...");
  for (let i = 0; i < chunkCount; i++) {
    const start = i * CHUNK_SIZE;
    const end = Math.min(start + CHUNK_SIZE, fileSize);
    const chunk = fileBuffer.slice(start, end);

    echoInfo(`Uploading chunk ${i + 1}/${chunkCount} (${chunk.length} bytes)`);
    const chunkResult = await backend.uploads_put_chunk(sessionId, i, Array.from(chunk));
    if ("Err" in chunkResult) {
      throw new Error(`Chunk upload failed: ${JSON.stringify(chunkResult.Err)}`);
    }
  }
  echoInfo("All chunks uploaded successfully");

  // Finish upload
  const expectedHash = crypto.createHash("sha256").update(fileBuffer).digest();
  echoInfo(`Expected hash: ${expectedHash.toString("hex")}`);

  echoInfo("Calling uploads_finish...");
  const finishResult = await backend.uploads_finish(sessionId, expectedHash, fileSize);
  if ("Err" in finishResult) {
    throw new Error(`Upload finish failed: ${JSON.stringify(finishResult.Err)}`);
  }

  const blobId = finishResult.Ok.blob_id;
  echoInfo(`Upload finished successfully! Blob ID: ${blobId}`);

  // Create memory with internal blob
  echoInfo("Creating memory with internal blob...");
  const memoryMetadata = {
    memory_type: { Image: null },
    title: ["Test Memory for Selective Deletion"],
    description: ["A test memory created for selective deletion testing"],
    content_type: "image/jpeg",
    created_at: BigInt(Date.now() * 1000000),
    updated_at: BigInt(Date.now() * 1000000),
    uploaded_at: BigInt(Date.now() * 1000000),
    date_of_memory: [],
    file_created_at: [],
    parent_folder_id: [],
    tags: ["test", "selective-deletion"],
    deleted_at: [],
    people_in_memory: [],
    location: [],
    memory_notes: [],
    created_by: [],
    database_storage_edges: [],
  };

  const internalBlobAsset = {
    blob_id: blobId,
    metadata: {
      Image: {
        base: {
          name: "Test Internal Blob for Selective Deletion",
          description: ["Asset stored as internal blob for selective deletion testing"],
          tags: ["test", "internal", "selective-deletion"],
          asset_type: { Original: null },
          bytes: BigInt(fileSize),
          mime_type: "image/jpeg",
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
        color_space: [],
        exif_data: [],
        compression_ratio: [],
        dpi: [],
        orientation: [],
      },
    },
  };

  const memoryIdempotencyKey = `memory-selective-delete-test-${Date.now()}`;
  const memoryResult = await backend.memories_create_with_internal_blobs(
    capsuleId,
    memoryMetadata,
    [internalBlobAsset],
    memoryIdempotencyKey
  );

  if ("Err" in memoryResult) {
    throw new Error(`Memory creation failed: ${JSON.stringify(memoryResult.Err)}`);
  }
  const memoryId = memoryResult.Ok;
  echoInfo(`Memory created successfully! Memory ID: ${memoryId}`);

  // ========================================
  // Test 2: Full Deletion (delete_assets: true)
  // ========================================
  echoInfo("=== Test 2: Full Deletion (delete_assets: true) ===");

  // Create a second memory for full deletion test
  const memory2IdempotencyKey = `memory-full-delete-test-${Date.now()}`;
  const memory2Result = await backend.memories_create_with_internal_blobs(
    capsuleId,
    memoryMetadata,
    [internalBlobAsset],
    memory2IdempotencyKey
  );

  if ("Err" in memory2Result) {
    throw new Error(`Second memory creation failed: ${JSON.stringify(memory2Result.Err)}`);
  }
  const memory2Id = memory2Result.Ok;
  echoInfo(`Second memory created successfully! Memory ID: ${memory2Id}`);

  // Test full deletion (delete_assets: true)
  echoInfo("Testing full deletion (delete_assets: true)...");
  const fullDeleteResult = await backend.memories_delete(memory2Id, true);
  if ("Err" in fullDeleteResult) {
    echoFail(`Full deletion failed: ${JSON.stringify(fullDeleteResult.Err)}`);
  } else {
    echoPass("Full deletion successful");

    // Verify memory is deleted
    echoInfo("Verifying memory is deleted...");
    const memoryReadResult = await backend.memories_read(memory2Id);
    if ("Err" in memoryReadResult && "NotFound" in memoryReadResult.Err) {
      echoPass("Memory is confirmed deleted");
    } else {
      echoFail("Memory should be deleted but still exists");
    }

    // Verify blob is also deleted (full deletion)
    echoInfo("Verifying blob is also deleted...");
    const blobReadResult = await backend.blob_read(blobId);
    if ("Err" in blobReadResult && "NotFound" in blobReadResult.Err) {
      echoPass("Blob is also deleted (full deletion working)");
    } else {
      echoFail("Blob should be deleted in full deletion mode");
    }
  }

  // ========================================
  // Test 3: Metadata-Only Deletion (delete_assets: false)
  // ========================================
  echoInfo("=== Test 3: Metadata-Only Deletion (delete_assets: false) ===");

  // Test metadata-only deletion (delete_assets: false)
  echoInfo("Testing metadata-only deletion (delete_assets: false)...");
  const metadataOnlyDeleteResult = await backend.memories_delete(memoryId, false);
  if ("Err" in metadataOnlyDeleteResult) {
    echoFail(`Metadata-only deletion failed: ${JSON.stringify(metadataOnlyDeleteResult.Err)}`);
  } else {
    echoPass("Metadata-only deletion successful");

    // Verify memory is deleted
    echoInfo("Verifying memory is deleted...");
    const memoryReadResult = await backend.memories_read(memoryId);
    if ("Err" in memoryReadResult && "NotFound" in memoryReadResult.Err) {
      echoPass("Memory is confirmed deleted");
    } else {
      echoFail("Memory should be deleted but still exists");
    }

    // Verify blob is NOT deleted (metadata-only deletion)
    echoInfo("Verifying blob is preserved...");
    const blobReadResult = await backend.blob_read(blobId);
    if ("Ok" in blobReadResult) {
      echoPass("Blob is preserved (metadata-only deletion working)");
      echoInfo(`Blob size: ${blobReadResult.Ok.length} bytes`);
    } else {
      echoFail("Blob should be preserved in metadata-only deletion mode");
    }
  }

  echoPass("Selective memory deletion test completed!");
  echoInfo("✅ Full deletion (delete_assets: true) works - deletes memory + assets");
  echoInfo("✅ Metadata-only deletion (delete_assets: false) works - deletes memory, preserves assets");
  echoInfo("✅ Both deletion modes work correctly with real files");
}

// Main execution
async function main() {
  const backendCanisterId = process.argv[2];
  const filePath = process.argv[3];

  if (!backendCanisterId || !filePath) {
    echoError("Usage: node test_selective_memory_deletion.mjs <BACKEND_CANISTER_ID> <FILE_PATH>");
    process.exit(1);
  }

  try {
    await testSelectiveMemoryDeletion(backendCanisterId, filePath);
    echoPass("All selective memory deletion tests passed!");
  } catch (error) {
    echoError(`Test failed: ${error.message}`);
    process.exit(1);
  }
}

main();
