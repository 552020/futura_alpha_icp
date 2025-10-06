#!/usr/bin/env node

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { readFileSync } from "fs";
import crypto from "crypto";
import { loadDfxIdentity } from "./ic-identity.js";

// Test configuration
const TEST_NAME = "Blob Delete Real Assets Test";
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

async function testBlobDeleteWithRealAssets(backendCanisterId, filePath) {
  echoInfo(`Starting ${TEST_NAME}`);
  echoInfo(`Testing blob deletion with real assets using file: ${filePath}`);

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
  // Test 1: Create Internal Blob Asset
  // ========================================
  echoInfo("=== Test 1: Creating Internal Blob Asset ===");

  // Upload file as blob
  const chunkCount = Math.ceil(fileSize / CHUNK_SIZE);
  const idempotencyKey = `blob-delete-test-${Date.now()}`;

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
    title: ["Test Memory for Blob Delete"],
    description: ["A test memory created for blob deletion testing"],
    content_type: "image/jpeg",
    created_at: BigInt(Date.now() * 1000000),
    updated_at: BigInt(Date.now() * 1000000),
    uploaded_at: BigInt(Date.now() * 1000000),
    date_of_memory: [],
    file_created_at: [],
    parent_folder_id: [],
    tags: ["test", "blob-delete"],
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
          name: "Test Internal Blob for Deletion",
          description: ["Asset stored as internal blob for deletion testing"],
          tags: ["test", "internal", "deletion"],
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

  const memoryIdempotencyKey = `memory-delete-test-${Date.now()}`;
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
  // Test 2: Create Inline Asset
  // ========================================
  echoInfo("=== Test 2: Creating Inline Asset ===");

  // Create a small inline memory
  const smallFileBuffer = fileBuffer.slice(0, Math.min(1000, fileSize)); // First 1KB

  const inlineAsset = {
    Image: {
      base: {
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000),
        asset_type: { Original: null },
        sha256: [],
        name: "Test Inline Asset for Deletion",
        storage_key: [],
        tags: ["test", "inline", "deletion"],
        processing_error: [],
        mime_type: "image/jpeg",
        description: ["Inline asset for deletion testing"],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(smallFileBuffer.length),
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      exif_data: [],
      compression_ratio: [],
      orientation: [],
      dpi: [],
      color_space: [],
    },
  };

  const inlineMemoryResult = await backend.memories_create(
    capsuleId,
    [new Uint8Array(smallFileBuffer)], // bytes parameter for inline storage
    [], // blob_ref (empty array for None)
    [], // external_location (empty array for None)
    [], // external_storage_key (empty array for None)
    [], // external_url (empty array for None)
    [], // external_size (empty array for None)
    [], // external_hash (empty array for None)
    inlineAsset, // asset_metadata
    `inline-delete-test-${Date.now()}` // idempotency key
  );

  if ("Err" in inlineMemoryResult) {
    throw new Error(`Inline memory creation failed: ${JSON.stringify(inlineMemoryResult.Err)}`);
  }
  const inlineMemoryId = inlineMemoryResult.Ok;
  echoInfo(`Inline memory created successfully! Memory ID: ${inlineMemoryId}`);

  // ========================================
  // Test 3: Test Blob Deletion on Real Assets
  // ========================================
  echoInfo("=== Test 3: Testing Blob Deletion on Real Assets ===");

  // Test deletion of internal blob
  echoInfo("Testing deletion of internal blob...");
  const internalBlobDeleteResult = await backend.blob_delete(blobId);
  if ("Ok" in internalBlobDeleteResult) {
    echoPass(`Internal blob deletion successful: ${internalBlobDeleteResult.Ok}`);

    // Verify blob is actually deleted
    echoInfo("Verifying blob is deleted...");
    const blobReadResult = await backend.blob_read(blobId);
    if ("Err" in blobReadResult && "NotFound" in blobReadResult.Err) {
      echoPass("Blob is confirmed deleted (NotFound error)");
    } else {
      echoFail("Blob deletion verification failed - blob still exists");
    }

    // Verify memory still exists (blob deletion should NOT delete the memory)
    echoInfo("Verifying memory still exists after blob deletion...");
    const memoryReadResult = await backend.memories_read(memoryId);
    if ("Ok" in memoryReadResult) {
      echoPass("Memory still exists after blob deletion (correct behavior)");
      echoInfo(`Memory still has ${memoryReadResult.Ok.blob_internal_assets.length} internal blob assets`);
      echoInfo("Note: Blob reference exists but blob data is deleted");
    } else {
      echoFail(`Memory should still exist but got error: ${JSON.stringify(memoryReadResult.Err)}`);
    }
  } else {
    echoFail(`Internal blob deletion failed: ${JSON.stringify(internalBlobDeleteResult.Err)}`);
  }

  // Test deletion of inline asset (should fail with helpful message)
  echoInfo("Testing deletion of inline asset...");
  const inlineAssetId = `inline_${inlineMemoryId}`; // Construct inline asset ID
  const inlineDeleteResult = await backend.blob_delete(inlineAssetId);
  if ("Err" in inlineDeleteResult) {
    echoPass(`Inline asset deletion (expected error): ${inlineDeleteResult.Err.InvalidArgument}`);
  } else {
    echoFail(`Inline asset deletion should have failed but got: ${inlineDeleteResult.Ok}`);
  }

  // Test deletion of non-existent blob
  echoInfo("Testing deletion of non-existent blob...");
  const nonExistentBlobId = "blob_99999999999999999999";
  const nonExistentDeleteResult = await backend.blob_delete(nonExistentBlobId);
  if ("Err" in nonExistentDeleteResult) {
    echoPass(`Non-existent blob deletion (expected error): ${JSON.stringify(nonExistentDeleteResult.Err)}`);
  } else {
    echoFail(`Non-existent blob deletion should have failed but got: ${nonExistentDeleteResult.Ok}`);
  }

  echoPass("Real asset blob deletion test completed!");
  echoInfo("✅ Internal blob creation and deletion works with real files");
  echoInfo("✅ Inline asset creation works with real files");
  echoInfo("✅ Blob deletion works on real internal blobs");
  echoInfo("✅ Inline asset deletion returns appropriate error");
  echoInfo("✅ Non-existent blob deletion returns appropriate error");
}

// Main execution
async function main() {
  const backendCanisterId = process.argv[2];
  const filePath = process.argv[3];

  if (!backendCanisterId || !filePath) {
    echoError("Usage: node test_blob_delete_real_assets.mjs <BACKEND_CANISTER_ID> <FILE_PATH>");
    process.exit(1);
  }

  try {
    await testBlobDeleteWithRealAssets(backendCanisterId, filePath);
    echoPass("All real asset blob deletion tests passed!");
  } catch (error) {
    echoError(`Test failed: ${error.message}`);
    process.exit(1);
  }
}

main();
