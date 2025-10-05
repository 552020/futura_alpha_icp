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
  const beginResult = await backend.uploads_begin(capsuleId, assetMetadata, chunkCount, idempotencyKey);

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

  // Verify the upload by checking if memory exists
  echoInfo("Verifying upload...");

  // First check if memory exists in the list
  const memoriesResult = await backend.memories_list(capsuleId, [], []);

  if ("Ok" in memoriesResult) {
    const page = memoriesResult.Ok;
    const memories = page.items;
    const uploadedMemoryHeader = memories.find((m) => m.id === result.memory_id);

    if (uploadedMemoryHeader) {
      echoPass("Memory found in capsule list!");

      // Now get the full memory with assets
      echoInfo("Getting full memory with assets...");
      const fullMemoryResult = await backend.memories_read(result.memory_id);

      if ("Ok" in fullMemoryResult) {
        const uploadedMemory = fullMemoryResult.Ok;
        echoPass("Full memory retrieved!");
        echoInfo(`Memory ID: ${uploadedMemory.id}`);
        echoInfo(`Memory title: ${uploadedMemory.metadata?.title || "No title"}`);

        // Check for assets in the correct fields
        const hasInlineAssets = uploadedMemory.inline_assets && uploadedMemory.inline_assets.length > 0;
        const hasBlobInternalAssets =
          uploadedMemory.blob_internal_assets && uploadedMemory.blob_internal_assets.length > 0;
        const hasBlobExternalAssets =
          uploadedMemory.blob_external_assets && uploadedMemory.blob_external_assets.length > 0;

        if (hasInlineAssets || hasBlobInternalAssets || hasBlobExternalAssets) {
          echoPass("Assets found in memory!");

          if (hasInlineAssets) {
            const asset = uploadedMemory.inline_assets[0];
            echoInfo(`Inline asset: ${asset.metadata?.Image?.base?.name || "Unknown"}`);
            echoInfo(`Asset size: ${asset.metadata?.Image?.base?.bytes || "Unknown"} bytes`);
          }

          if (hasBlobInternalAssets) {
            const asset = uploadedMemory.blob_internal_assets[0];
            echoInfo(`Blob internal asset: ${asset.metadata?.Image?.base?.name || "Unknown"}`);
            echoInfo(`Asset size: ${asset.metadata?.Image?.base?.bytes || "Unknown"} bytes`);
            echoInfo(`Blob locator: ${asset.blob_ref?.locator || "Unknown"}`);
          }

          if (hasBlobExternalAssets) {
            const asset = uploadedMemory.blob_external_assets[0];
            echoInfo(`Blob external asset: ${asset.metadata?.Image?.base?.name || "Unknown"}`);
            echoInfo(`Storage key: ${asset.storage_key || "Unknown"}`);
          }

          echoPass("Asset metadata matches uploaded file!");
        } else {
          echoFail("No assets found in memory - upload failed!");
          return false;
        }
      } else {
        echoFail("Failed to read full memory");
        return false;
      }
    } else {
      echoFail("Memory not found in capsule list");
      return false;
    }
  } else {
    echoFail("Failed to list memories");
    return false;
  }

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
