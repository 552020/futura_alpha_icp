#!/usr/bin/env node

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { readFileSync } from "fs";
import crypto from "crypto";
import { loadDfxIdentity } from "./ic-identity.js";

// Test configuration
const TEST_NAME = "Debug Blob Delete";
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

async function debugBlobDelete(backendCanisterId, filePath) {
  echoInfo(`Starting ${TEST_NAME}`);
  echoInfo(`Debugging blob deletion with file: ${filePath}`);

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

  // Upload file as blob
  const chunkCount = Math.ceil(fileSize / CHUNK_SIZE);
  const idempotencyKey = `debug-delete-test-${Date.now()}`;

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

  // ========================================
  // Debug: Check blob before deletion
  // ========================================
  echoInfo("=== Debug: Checking blob before deletion ===");

  // Check blob metadata
  echoInfo("Getting blob metadata...");
  const metaBeforeResult = await backend.blob_get_meta(blobId);
  if ("Ok" in metaBeforeResult) {
    echoPass(
      `Blob metadata before deletion: size=${metaBeforeResult.Ok.size}, chunks=${metaBeforeResult.Ok.chunk_count}`
    );
  } else {
    echoFail(`Failed to get blob metadata before deletion: ${JSON.stringify(metaBeforeResult.Err)}`);
  }

  // Try to read blob before deletion
  echoInfo("Attempting to read blob before deletion...");
  const readBeforeResult = await backend.blob_read(blobId);
  if ("Ok" in readBeforeResult) {
    echoPass(`Blob read before deletion successful: ${readBeforeResult.Ok.length} bytes`);
  } else {
    echoFail(`Blob read before deletion failed: ${JSON.stringify(readBeforeResult.Err)}`);
  }

  // ========================================
  // Debug: Delete blob
  // ========================================
  echoInfo("=== Debug: Deleting blob ===");

  const deleteResult = await backend.blob_delete(blobId);
  if ("Ok" in deleteResult) {
    echoPass(`Blob deletion successful: ${deleteResult.Ok}`);
  } else {
    echoFail(`Blob deletion failed: ${JSON.stringify(deleteResult.Err)}`);
    return;
  }

  // ========================================
  // Debug: Check blob after deletion
  // ========================================
  echoInfo("=== Debug: Checking blob after deletion ===");

  // Check blob metadata after deletion
  echoInfo("Getting blob metadata after deletion...");
  const metaAfterResult = await backend.blob_get_meta(blobId);
  if ("Err" in metaAfterResult) {
    if (metaAfterResult.Err.NotFound) {
      echoPass("Blob metadata correctly returns NotFound after deletion");
    } else {
      echoFail(`Blob metadata after deletion returned unexpected error: ${JSON.stringify(metaAfterResult.Err)}`);
    }
  } else {
    echoFail(`Blob metadata still exists after deletion: ${JSON.stringify(metaAfterResult.Ok)}`);
  }

  // Try to read blob after deletion
  echoInfo("Attempting to read blob after deletion...");
  const readAfterResult = await backend.blob_read(blobId);
  if ("Err" in readAfterResult) {
    if (readAfterResult.Err.NotFound) {
      echoPass("Blob read correctly returns NotFound after deletion");
    } else {
      echoFail(`Blob read after deletion returned unexpected error: ${JSON.stringify(readAfterResult.Err)}`);
    }
  } else {
    echoFail(`Blob read still works after deletion: ${readAfterResult.Ok.length} bytes`);
  }

  // Try to read blob chunk after deletion
  echoInfo("Attempting to read blob chunk after deletion...");
  const chunkAfterResult = await backend.blob_read_chunk(blobId, 0);
  if ("Err" in chunkAfterResult) {
    if (chunkAfterResult.Err.NotFound) {
      echoPass("Blob chunk read correctly returns NotFound after deletion");
    } else {
      echoFail(`Blob chunk read after deletion returned unexpected error: ${JSON.stringify(chunkAfterResult.Err)}`);
    }
  } else {
    echoFail(`Blob chunk read still works after deletion: ${chunkAfterResult.Ok.length} bytes`);
  }

  echoInfo("Debug completed!");
}

// Main execution
async function main() {
  const backendCanisterId = process.argv[2];
  const filePath = process.argv[3];

  if (!backendCanisterId || !filePath) {
    echoError("Usage: node debug_blob_delete.mjs <BACKEND_CANISTER_ID> <FILE_PATH>");
    process.exit(1);
  }

  try {
    await debugBlobDelete(backendCanisterId, filePath);
    echoPass("Debug completed successfully!");
  } catch (error) {
    echoError(`Debug failed: ${error.message}`);
    process.exit(1);
  }
}

main();
