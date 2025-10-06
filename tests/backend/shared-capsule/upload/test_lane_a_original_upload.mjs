#!/usr/bin/env node

/**
 * Lane A: Original Upload Test
 *
 * Tests only the original file upload functionality (Lane A)
 * without image processing or memory creation.
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
const TEST_NAME = "Lane A: Original Upload Test";
const TEST_IMAGE_PATH = "./assets/input/avocado_big_21mb.jpg";
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

// Lane A: Upload original file to ICP (matches frontend uploadOriginalToS3)
async function uploadOriginalToICP(backend, fileBuffer, fileName) {
  const startTime = Date.now();

  echoInfo(`📤 Uploading: ${fileName} (${formatFileSize(fileBuffer.length)})`);

  // Validate file size using helper
  validateFileSize(fileBuffer.length);

  // Create a new capsule for this test
  const capsuleResult = await backend.capsules_create([]);
  if ("Err" in capsuleResult) {
    throw new Error(`Capsule creation failed: ${JSON.stringify(capsuleResult.Err)}`);
  }
  const capsuleId = capsuleResult.Ok.id;

  // Calculate chunk count and create chunks using helpers
  const chunkCount = Math.ceil(fileBuffer.length / CHUNK_SIZE);
  const chunks = createFileChunks(fileBuffer, CHUNK_SIZE);
  const idempotencyKey = generateFileId("upload");

  // Begin upload session (no assetMetadata needed anymore)
  const beginResult = await backend.uploads_begin(capsuleId, chunkCount, idempotencyKey);

  // Handle different response formats
  let sessionId;
  if (typeof beginResult === "number" || typeof beginResult === "string") {
    // Direct response (number or string) - this is the current backend behavior
    sessionId = beginResult;
    echoInfo(`✅ Upload session started: ${sessionId}`);
  } else if (beginResult && typeof beginResult === "object") {
    // Object response with Ok/Err structure
    try {
      validateUploadResponse(beginResult, ["Ok"]);
      sessionId = beginResult.Ok;
      echoInfo(`✅ Upload session started: ${sessionId}`);
    } catch (error) {
      throw handleUploadError(error, "Upload begin");
    }
  } else {
    throw new Error(`Unexpected response format: ${typeof beginResult} - ${JSON.stringify(beginResult)}`);
  }

  // Upload file in chunks with progress tracking
  echoInfo(`📦 Uploading ${chunks.length} chunks (${formatFileSize(CHUNK_SIZE)} each)...`);

  const progressCallback = createProgressCallback(chunks.length, (progress, completed, total) => {
    if (completed % 5 === 0 || completed === total) {
      echoInfo(`  📈 ${progress}% (${completed}/${total} chunks)`);
    }
  });

  for (let i = 0; i < chunks.length; i++) {
    const putChunkResult = await backend.uploads_put_chunk(sessionId, i, chunks[i]);

    // Handle different response formats for put_chunk
    if (typeof putChunkResult === "object" && putChunkResult !== null) {
      try {
        validateUploadResponse(putChunkResult);
      } catch (error) {
        throw handleUploadError(error, `Put chunk ${i}`);
      }
    } else {
      // Direct response (success) - no validation needed
      echoInfo(`✅ Chunk ${i} uploaded successfully`);
    }

    progressCallback(i);
  }

  // Calculate hash and total length for finish using helpers
  const hash = calculateFileHash(fileBuffer);
  const totalLen = BigInt(fileBuffer.length);

  // Finish upload
  const finishResult = await backend.uploads_finish(sessionId, Array.from(hash), totalLen);

  // Handle different response formats for finish
  let blobId;
  if (typeof finishResult === "string") {
    // Direct response - blob ID only (new format after refactoring)
    blobId = finishResult;
    echoInfo(`✅ Upload finished successfully: blob_id=${blobId}`);
  } else if (finishResult && typeof finishResult === "object") {
    // Object response with Ok/Err structure
    try {
      validateUploadResponse(finishResult, ["Ok"]);
      const result = finishResult.Ok;
      if (result && typeof result === "object" && "blob_id" in result) {
        // New format: UploadFinishResult with blob_id only
        blobId = result.blob_id;
        echoInfo(`✅ Upload finished successfully: blob_id=${blobId}`);
      } else {
        // Legacy format: direct string
        blobId = result;
        echoInfo(`✅ Upload finished successfully: blob_id=${blobId}`);
      }
    } catch (error) {
      throw handleUploadError(error, "Upload finish");
    }
  } else {
    throw new Error(`Unexpected finish response format: ${typeof finishResult} - ${JSON.stringify(finishResult)}`);
  }

  const duration = Date.now() - startTime;
  const uploadSpeed = formatUploadSpeed(fileBuffer.length, duration);

  echoInfo(
    `✅ Upload completed: ${fileName} (${formatFileSize(fileBuffer.length)}) in ${formatDuration(
      duration
    )} (${uploadSpeed})`
  );

  return blobId;
}

// Test function
async function testLaneAOriginalUpload() {
  const fileBuffer = fs.readFileSync(TEST_IMAGE_PATH);
  const fileName = path.basename(TEST_IMAGE_PATH);

  const blobId = await uploadOriginalToICP(backend, fileBuffer, fileName);

  // Verify blob was created
  const blobMeta = await backend.blob_get_meta(blobId);
  if ("Err" in blobMeta) {
    throw new Error(`Failed to get blob meta: ${JSON.stringify(blobMeta.Err)}`);
  }

  return blobMeta.Ok.size === BigInt(fileBuffer.length);
}

// Main test runner
async function main() {
  echoInfo(`Starting ${TEST_NAME}`);

  // Parse command line arguments
  const args = process.argv.slice(2);
  const backendCanisterId = args[0];
  const network = args[1] || "local"; // Default to local network

  if (!backendCanisterId) {
    echoFail("Usage: node test_lane_a_original_upload.mjs <CANISTER_ID> [mainnet|local]");
    echoFail("Example: node test_lane_a_original_upload.mjs uxrrr-q7777-77774-qaaaq-cai local");
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
    const result = await testLaneAOriginalUpload();
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
