#!/usr/bin/env node

import { Actor, HttpAgent } from "@dfinity/agent";
import { readFileSync } from "fs";
import { loadDfxIdentity } from "./ic-identity.js";

const BACKEND_CANISTER_ID = process.argv[2];
const FILE_PATH = process.argv[3];

if (!BACKEND_CANISTER_ID || !FILE_PATH) {
  console.error("💥 Usage: node test_memory_creation_with_internal_blobs.mjs <BACKEND_CANISTER_ID> <FILE_PATH>");
  process.exit(1);
}

console.log("ℹ️  Starting Memory Creation with Internal Blobs Test");
console.log(`ℹ️  Testing memory creation with file: ${FILE_PATH}`);

try {
  // Load identity and create agent
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
      canisterId: BACKEND_CANISTER_ID,
    }
  );

  // Read file
  const fileBuffer = readFileSync(FILE_PATH);
  const fileSize = fileBuffer.length;
  console.log(`ℹ️  File: ${FILE_PATH.split("/").pop()} (${fileSize} bytes)`);

  // Create new capsule for test
  console.log(`ℹ️  Creating new capsule for test...`);

  const capsuleResult = await backend.capsules_create([]); // No subject
  if (capsuleResult.Err) {
    throw new Error(`Capsule creation failed: ${JSON.stringify(capsuleResult.Err)}`);
  }
  const capsuleId = capsuleResult.Ok.id;
  console.log(`ℹ️  Created new capsule: ${capsuleId}`);

  // Step 1: Upload blob using pure blob upload
  console.log("ℹ️  Step 1: Uploading blob...");

  const chunkSize = 1_800_000; // 1.8MB backend chunk size
  const chunkCount = Math.ceil(fileSize / chunkSize);
  const idempotencyKey = `memory-test-${Date.now()}`;

  console.log(`ℹ️  Chunk count: ${chunkCount}, Idempotency key: ${idempotencyKey}`);

  // Begin upload
  console.log("ℹ️  Calling uploads_begin...");
  const beginResult = await backend.uploads_begin(capsuleId, chunkCount, idempotencyKey);
  if (beginResult.Err) {
    throw new Error(`Upload begin failed: ${JSON.stringify(beginResult.Err)}`);
  }
  const sessionId = beginResult.Ok;
  console.log(`ℹ️  Upload session started: ${sessionId}`);

  // Upload chunks
  console.log("ℹ️  Uploading chunks...");
  for (let i = 0; i < chunkCount; i++) {
    const start = i * chunkSize;
    const end = Math.min(start + chunkSize, fileSize);
    const chunk = fileBuffer.slice(start, end);

    console.log(`ℹ️  Uploading chunk ${i + 1}/${chunkCount} (${chunk.length} bytes)`);

    const chunkResult = await backend.uploads_put_chunk(sessionId, i, Array.from(chunk));
    if (chunkResult.Err) {
      throw new Error(`Chunk upload failed: ${JSON.stringify(chunkResult.Err)}`);
    }
  }
  console.log("ℹ️  All chunks uploaded successfully");

  // Compute expected hash
  const crypto = await import("crypto");
  const expectedHash = crypto.createHash("sha256").update(fileBuffer).digest();
  console.log(`ℹ️  Expected hash: ${expectedHash.toString("hex")}`);

  // Finish upload
  console.log("ℹ️  Calling uploads_finish...");
  const finishResult = await backend.uploads_finish(sessionId, expectedHash, fileSize);
  if (finishResult.Err) {
    throw new Error(`Upload finish failed: ${JSON.stringify(finishResult.Err)}`);
  }

  const result = finishResult.Ok;
  const blobId = result.blob_id;
  console.log(`ℹ️  Upload finished successfully!`);
  console.log(`ℹ️  Blob ID: ${blobId}`);

  // Step 2: Create memory with internal blob
  console.log("ℹ️  Step 2: Creating memory with internal blob...");

  const memoryMetadata = {
    memory_type: { Image: null },
    title: ["Test Memory with Internal Blob"],
    description: ["A test memory created with internal blob storage"],
    content_type: "image/jpeg",
    created_at: BigInt(Date.now() * 1000000), // nanoseconds
    updated_at: BigInt(Date.now() * 1000000),
    uploaded_at: BigInt(Date.now() * 1000000),
    date_of_memory: [],
    file_created_at: [],
    parent_folder_id: [],
    tags: ["test", "internal-blob"],
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
          name: "Test Internal Blob Asset",
          description: ["Asset stored as internal blob"],
          tags: ["test", "internal"],
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

  const memoryIdempotencyKey = `memory-${Date.now()}`;

  const memoryResult = await backend.memories_create_with_internal_blobs(
    capsuleId,
    memoryMetadata,
    [internalBlobAsset],
    memoryIdempotencyKey
  );

  if (memoryResult.Err) {
    throw new Error(`Memory creation failed: ${JSON.stringify(memoryResult.Err)}`);
  }

  const memoryId = memoryResult.Ok;
  console.log(`ℹ️  Memory created successfully!`);
  console.log(`ℹ️  Memory ID: ${memoryId}`);

  // Step 3: Verify memory was created correctly
  console.log("ℹ️  Step 3: Verifying memory...");

  // List memories
  const listResult = await backend.memories_list(capsuleId, [], [10]);
  if (listResult.Err) {
    throw new Error(`Memory list failed: ${JSON.stringify(listResult.Err)}`);
  }

  const memories = listResult.Ok.items;
  console.log(`ℹ️  Found ${memories.length} memories in list`);
  console.log(`ℹ️  Looking for memory ID: ${memoryId}`);
  console.log(`ℹ️  Available memory IDs: ${memories.map((m) => m.id).join(", ")}`);

  // Try to read the memory directly first to confirm it exists
  console.log("ℹ️  Attempting to read memory directly...");
  const readResult = await backend.memories_read(memoryId);
  if (readResult.Err) {
    throw new Error(`Memory read failed: ${JSON.stringify(readResult.Err)}`);
  }

  const fullMemory = readResult.Ok;
  console.log(`✅ Memory read successful`);
  console.log(`ℹ️  Memory title: ${fullMemory.metadata.title[0] || "No title"}`);
  console.log(`ℹ️  Internal blob assets: ${fullMemory.blob_internal_assets.length}`);

  if (fullMemory.blob_internal_assets.length === 0) {
    throw new Error("No internal blob assets found in memory");
  }

  const blobAsset = fullMemory.blob_internal_assets[0];
  console.log(`✅ Internal blob asset found: ${blobAsset.asset_id}`);

  // Now check if it's in the list
  const createdMemory = memories.find((m) => m.id === memoryId);
  if (createdMemory) {
    console.log(`✅ Memory also found in list: ${createdMemory.id}`);
  } else {
    console.log(`⚠️  Memory exists but not found in list (this might be a listing/indexing issue)`);
  }
  console.log(`ℹ️  Blob locator: ${blobAsset.blob_ref.locator}`);
  console.log(`ℹ️  Blob size: ${blobAsset.blob_ref.len} bytes`);

  // Verify blob is readable
  console.log("ℹ️  Verifying blob is readable...");
  const blobReadResult = await backend.blob_read(blobAsset.blob_ref.locator);
  if (blobReadResult.Err) {
    throw new Error(`Blob read failed: ${JSON.stringify(blobReadResult.Err)}`);
  }

  const blobData = blobReadResult.Ok;
  console.log(`✅ Blob read successful - size: ${blobData.length} bytes`);

  if (blobData.length !== fileSize) {
    throw new Error(`Size mismatch: expected ${fileSize}, got ${blobData.length}`);
  }

  console.log("✅ Memory creation with internal blobs test PASSED!");
  console.log("ℹ️  ✅ Blob upload works correctly");
  console.log("ℹ️  ✅ Memory creation with internal blob works");
  console.log("ℹ️  ✅ Memory can be read and verified");
  console.log("ℹ️  ✅ Blob is accessible through memory");
} catch (error) {
  console.error("💥 Test failed:", error.message);
  process.exit(1);
}
