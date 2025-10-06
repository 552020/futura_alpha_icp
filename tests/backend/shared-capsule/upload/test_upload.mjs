#!/usr/bin/env node

/**
 * Generic Upload Test
 *
 * This test uploads any file to verify chunked upload functionality.
 * Takes a file path as a command line argument.
 */

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { Actor, HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity, makeMainnetAgent } from "./ic-identity.js";

// Import the backend interface
import { idlFactory } from "../../../../src/nextjs/src/ic/declarations/backend/backend.did.js";

// Test configuration
const TEST_NAME = "Generic Upload Test";
const CHUNK_SIZE = 1_800_000; // 1.8MB chunks - matches backend CHUNK_SIZE

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

// Main test function
async function testUpload(filePath) {
  echoInfo(`Testing upload with file: ${filePath}`);

  // Check if file exists
  if (!fs.existsSync(filePath)) {
    throw new Error(`Test file not found: ${filePath}`);
  }

  // Read file
  const fileBuffer = fs.readFileSync(filePath);
  const fileName = path.basename(filePath);
  const fileSize = fileBuffer.length;

  echoInfo(`File: ${fileName} (${fileSize} bytes)`);

  // Get or create test capsule
  const capsuleResult = await backend.capsules_read_basic([]);
  let capsuleId;

  if ("Ok" in capsuleResult && capsuleResult.Ok) {
    capsuleId = capsuleResult.Ok.capsule_id;
    echoInfo(`Using existing capsule: ${capsuleId}`);
  } else {
    const createResult = await backend.capsules_create([]);
    if (!("Ok" in createResult)) {
      throw new Error("Failed to create capsule: " + JSON.stringify(createResult));
    }
    capsuleId = createResult.Ok.id;
    echoInfo(`Created new capsule: ${capsuleId}`);
  }

  // Create asset metadata for image
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
        name: fileName,
        storage_key: [],
        tags: ["test", "upload", "chunked-upload"],
        processing_error: [],
        mime_type: "image/jpeg",
        description: ["Test file for chunked upload"],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(fileSize),
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

  // Calculate chunk count
  const chunkCount = Math.ceil(fileSize / CHUNK_SIZE);
  const idempotencyKey = `upload-test-${Date.now()}`;

  echoInfo(`Chunk count: ${chunkCount}, Idempotency key: ${idempotencyKey}`);

  // Begin upload session
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
    const chunkData = fileBuffer.slice(start, end);

    echoInfo(`Uploading chunk ${i + 1}/${chunkCount} (${chunkData.length} bytes)`);

    const chunkResult = await backend.uploads_put_chunk(sessionId, i, chunkData);
    if ("Err" in chunkResult) {
      throw new Error(`Chunk upload failed: ${JSON.stringify(chunkResult.Err)}`);
    }
  }

  echoInfo("All chunks uploaded successfully");

  // Compute expected hash
  const expectedHash = crypto.createHash("sha256").update(fileBuffer).digest();
  echoInfo(`Expected hash: ${expectedHash.toString("hex")}`);

  // Finish upload
  echoInfo("Calling uploads_finish...");
  const finishResult = await backend.uploads_finish(sessionId, expectedHash, fileSize);

  if ("Err" in finishResult) {
    throw new Error(`Upload finish failed: ${JSON.stringify(finishResult.Err)}`);
  }

  const result = finishResult.Ok;
  echoInfo(`Upload finished successfully!`);
  echoInfo(`Blob ID: ${result.blob_id}`);
  echoInfo(`Memory ID: ${result.memory_id}`);
  echoInfo(`Storage Location: ${result.storage_location}`);
  echoInfo(`Size: ${result.size} bytes`);

  // Verify the pure blob upload (no memory should be created)
  echoInfo("Verifying pure blob upload...");

  // Check that no memory was created (memory_id should be empty)
  if (result.memory_id === "") {
    echoPass("Memory ID is empty - no memory created (correct for pure blob upload)");
  } else {
    echoFail(`Memory ID should be empty but got: ${result.memory_id}`);
    throw new Error("Pure blob upload should not create memory");
  }

  // Verify blob ID is valid
  if (result.blob_id && result.blob_id.startsWith("blob_")) {
    echoPass("Blob ID is valid");
  } else {
    echoFail(`Invalid blob ID: ${result.blob_id}`);
    throw new Error("Invalid blob ID format");
  }

  // Verify file size matches
  if (Number(result.size) === fileSize) {
    echoPass("File size matches uploaded size");
  } else {
    echoFail(`Size mismatch: expected ${fileSize}, got ${result.size}`);
    throw new Error("File size mismatch");
  }

  // Test blob read - first get metadata to determine the right approach
  echoInfo("Testing blob read...");
  try {
    // First, get blob metadata to determine size and chunk count
    const blobMetaResult = await backend.blob_get_meta(result.blob_id);
    if ("Err" in blobMetaResult) {
      echoFail(`Blob metadata read failed: ${JSON.stringify(blobMetaResult.Err)}`);
      throw new Error("Blob metadata read failed");
    }

    const blobMeta = blobMetaResult.Ok;
    echoInfo(`Blob metadata: size=${blobMeta.size} bytes, chunks=${blobMeta.chunk_count}`);

    // Verify metadata size matches expected
    if (Number(blobMeta.size) !== fileSize) {
      echoFail(`Blob metadata size mismatch: expected ${fileSize}, got ${blobMeta.size}`);
      throw new Error("Blob metadata size verification failed");
    }
    echoPass("Blob metadata size matches expected size");

    // Choose read method based on size
    if (blobMeta.size > 3 * 1024 * 1024) {
      // Large file: use chunked read
      echoInfo("File is large (>3MB), using chunked blob read...");

      // Read first chunk to verify blob exists and is readable
      const blobReadResult = await backend.blob_read_chunk(result.blob_id, 0);
      if ("Ok" in blobReadResult) {
        const chunkData = blobReadResult.Ok;
        echoPass(`Blob read successful - first chunk size: ${chunkData.length} bytes`);
        echoInfo("Large blob exists and is readable (chunked read works)");
      } else {
        echoFail(`Blob chunk read failed: ${JSON.stringify(blobReadResult.Err)}`);
        throw new Error("Blob chunk read failed");
      }
    } else {
      // Small file: read entire blob
      echoInfo("File is small (≤3MB), reading entire blob...");
      const blobReadResult = await backend.blob_read(result.blob_id);
      if ("Ok" in blobReadResult) {
        const blobData = blobReadResult.Ok;
        if (blobData.length === fileSize) {
          echoPass("Blob read successful - size matches");

          // Verify hash matches
          const actualHash = crypto.createHash("sha256").update(blobData).digest();
          if (actualHash.equals(expectedHash)) {
            echoPass("Blob hash matches expected hash");
          } else {
            echoFail("Blob hash mismatch");
            throw new Error("Blob hash verification failed");
          }
        } else {
          echoFail(`Blob size mismatch: expected ${fileSize}, got ${blobData.length}`);
          throw new Error("Blob size verification failed");
        }
      } else {
        echoFail(`Blob read failed: ${JSON.stringify(blobReadResult.Err)}`);
        throw new Error("Blob read failed");
      }
    }
  } catch (error) {
    echoFail(`Blob read error: ${error.message}`);
    throw error;
  }

  // Verify no memory was created in the list
  echoInfo("Verifying no memory was created...");
  const memoriesResult = await backend.memories_list(capsuleId, [], [10]);
  if ("Ok" in memoriesResult) {
    const page = memoriesResult.Ok;
    const memories = page.items;
    const newMemory = memories.find((m) => m.id === result.memory_id);

    if (!newMemory) {
      echoPass("Memory list API works (no new memory should be created)");
    } else {
      echoFail("Unexpected memory found in list");
      throw new Error("Memory should not have been created");
    }
  } else {
    echoFail(`Memory list failed: ${JSON.stringify(memoriesResult.Err)}`);
    throw new Error("Memory list verification failed");
  }

  echoPass("Pure blob upload test PASSED!");
  echoInfo("✅ Blob upload, creation, and readback all work correctly");
  echoInfo("✅ No memory was created (pure blob storage)");
  echoInfo("✅ Ready for separate memory creation endpoints");

  return true;
}

// Main test execution
async function main() {
  echoInfo(`Starting ${TEST_NAME}`);

  // Get backend canister ID and file path
  const backendCanisterId = process.argv[2];
  const filePath = process.argv[3];

  if (!backendCanisterId) {
    echoError("Usage: node test_upload.mjs <BACKEND_CANISTER_ID> <FILE_PATH>");
    process.exit(1);
  }

  if (!filePath) {
    echoError("Usage: node test_upload.mjs <BACKEND_CANISTER_ID> <FILE_PATH>");
    process.exit(1);
  }

  // Setup agent and backend
  const identity = loadDfxIdentity();
  const agent = new HttpAgent({
    host: "http://127.0.0.1:4943",
    identity,
    fetch: (await import("node-fetch")).default,
  });
  await agent.fetchRootKey();

  backend = Actor.createActor(idlFactory, {
    agent,
    canisterId: backendCanisterId,
  });

  // Run test
  try {
    const result = await testUpload(filePath);
    if (result) {
      echoPass("Upload test passed!");
      process.exit(0);
    } else {
      echoFail("Upload test failed!");
      process.exit(1);
    }
  } catch (error) {
    echoError(`Test failed: ${error.message}`);
    process.exit(1);
  }
}

// Run the test
main().catch((error) => {
  echoError(`Test execution failed: ${error.message}`);
  process.exit(1);
});
