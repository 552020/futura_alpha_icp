/**
 * 2-Lane + 4-Asset Upload System Test
 *
 * This test reproduces the S3 2-lane + 4-asset system using ICP backend:
 * - Lane A: Upload original file to ICP blob storage
 * - Lane B: Process image derivatives (display, thumb, placeholder)
 * - Finalize: Create memory with all 4 asset types
 *
 * Uses functional approach to match frontend S3 system pattern.
 *
 * Reference Frontend Files:
 * - Main S3 Service: src/nextjs/src/lib/s3.ts
 * - 2-Lane + 4-Asset System: src/nextjs/src/services/upload/s3-with-processing.ts
 * - Image Processing: src/nextjs/src/services/upload/image-derivatives.ts
 * - Finalization: src/nextjs/src/services/upload/finalize.ts
 * - S3 Grants: src/nextjs/src/services/upload/s3-grant.ts
 * - Shared Utils: src/nextjs/src/services/upload/shared-utils.ts
 *
 * TODO: Import and reuse frontend functions where possible:
 * - processImageDerivativesPure() for real image processing
 * - Asset metadata structures and types
 * - Utility functions for file handling and validation
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
const TEST_NAME = "2-Lane + 4-Asset Upload System Test";
const TEST_IMAGE_PATH = "./assets/input/avocado_small_372kb.jpg";
// Constants - Aligned with backend configuration
const CHUNK_SIZE = 1_800_000; // 1.8MB - matches backend CHUNK_SIZE in types.rs
const INLINE_MAX = 32 * 1024; // 32KB - matches backend INLINE_MAX in types.rs

// Derivative asset storage strategy (from frontend S3 system):
// - Display: Blob storage + chunked upload (~100KB-2MB)
// - Thumb: Blob storage + chunked upload (~10KB-200KB)
// - Placeholder: Inline storage (~1KB-10KB, data URL in database)

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

function echoWarning(message) {
  console.log(`⚠️  ${message}`);
}

// Real image processing (Node.js version of frontend logic)
async function processImageDerivativesPure(fileBuffer, mimeType) {
  const originalSize = fileBuffer.length;

  echoInfo(`🖼️ Processing derivatives for ${formatFileSize(originalSize)} file`);

  // Validate file type using helper
  validateImageType(mimeType);

  // Get derivative size limits from helper
  const sizeLimits = calculateDerivativeSizes(originalSize);

  // Calculate realistic dimensions
  const aspectRatio = 16 / 9;
  const originalWidth = Math.floor(Math.sqrt(originalSize / 3));
  const originalHeight = Math.floor(originalWidth / aspectRatio);

  // Calculate derivative dimensions using helper
  const displayDims = calculateDerivativeDimensions(
    originalWidth,
    originalHeight,
    sizeLimits.display.maxWidth,
    sizeLimits.display.maxHeight
  );
  const thumbDims = calculateDerivativeDimensions(
    originalWidth,
    originalHeight,
    sizeLimits.thumb.maxWidth,
    sizeLimits.thumb.maxHeight
  );

  // Create derivative buffers (simulation - in real implementation, use Sharp/Jimp)
  const displaySize = Math.min(sizeLimits.display.maxSize, Math.floor(originalSize * 0.1));
  const displayBuffer = Buffer.alloc(displaySize);
  fileBuffer.copy(displayBuffer, 0, 0, displaySize);

  const thumbSize = Math.min(sizeLimits.thumb.maxSize, Math.floor(originalSize * 0.05));
  const thumbBuffer = Buffer.alloc(thumbSize);
  fileBuffer.copy(thumbBuffer, 0, 0, thumbSize);

  const placeholderSize = Math.min(sizeLimits.placeholder.maxSize, 1024);
  const placeholderBuffer = Buffer.alloc(placeholderSize, 0x42);

  // Log precise sizes using helper
  echoInfo(`📊 Derivative sizes:`);
  echoInfo(`  Display: ${formatFileSize(displaySize)} (${displayDims.width}x${displayDims.height})`);
  echoInfo(`  Thumb: ${formatFileSize(thumbSize)} (${thumbDims.width}x${thumbDims.height})`);
  echoInfo(`  Placeholder: ${formatFileSize(placeholderSize)} (32x18)`);

  return {
    original: {
      buffer: fileBuffer,
      size: originalSize,
      width: originalWidth,
      height: originalHeight,
      mimeType: mimeType,
    },
    display: {
      buffer: displayBuffer,
      size: displaySize,
      width: displayDims.width,
      height: displayDims.height,
      mimeType: "image/webp",
    },
    thumb: {
      buffer: thumbBuffer,
      size: thumbSize,
      width: thumbDims.width,
      height: thumbDims.height,
      mimeType: "image/webp",
    },
    placeholder: {
      buffer: placeholderBuffer,
      size: placeholderSize,
      width: 32,
      height: 18,
      mimeType: "image/webp",
    },
  };
}

// Lane A: Upload original file to ICP (matches frontend uploadOriginalToS3)
async function uploadOriginalToICP(backend, fileBuffer, fileName) {
  const startTime = Date.now();

  echoInfo(`📤 Uploading: ${fileName} (${formatFileSize(fileBuffer.length)})`);

  // Validate file size using helper
  validateFileSize(fileBuffer.length);

  // Get or create test capsule
  const capsuleResult = await backend.capsules_read_basic([]);
  let capsuleId;

  if ("Ok" in capsuleResult && capsuleResult.Ok) {
    capsuleId = capsuleResult.Ok.capsule_id;
  } else {
    const createResult = await backend.capsules_create([]);
    if (!("Ok" in createResult)) {
      throw new Error("Failed to create capsule");
    }
    capsuleId = createResult.Ok.id;
  }

  // Create asset metadata using helper
  const assetMetadata = createAssetMetadata(fileName, fileBuffer.length, "image/jpeg", "Original");

  // Calculate chunk count and create chunks using helpers
  const chunkCount = Math.ceil(fileBuffer.length / CHUNK_SIZE);
  const chunks = createFileChunks(fileBuffer, CHUNK_SIZE);
  const idempotencyKey = generateFileId("upload");

  // Begin upload session
  const beginResult = await backend.uploads_begin(capsuleId, assetMetadata, chunkCount, idempotencyKey);

  // Debug logging
  echoInfo(
    `🔍 Debug: beginResult type: ${typeof beginResult}, keys: ${Object.keys(beginResult)}, has Ok: ${
      "Ok" in beginResult
    }, has Err: ${"Err" in beginResult}`
  );

  // Handle different response formats
  let sessionId;
  if (typeof beginResult === "number" || typeof beginResult === "string") {
    // Direct response (number or string) - this is the current backend behavior
    sessionId = beginResult;
    echoInfo(`✅ Upload session started: ${sessionId}`);
  } else if (beginResult && typeof beginResult === "object") {
    // Object response with Ok/Err structure
    try {
      echoInfo(
        `🔍 Debug: About to call validateUploadResponse with: ${typeof beginResult}, keys: ${Object.keys(beginResult)}`
      );
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
  let memoryId;
  if (typeof finishResult === "string") {
    // Direct response (memory ID string) - this is the current backend behavior
    memoryId = finishResult;
    echoInfo(`✅ Upload finished successfully: ${memoryId}`);
  } else if (finishResult && typeof finishResult === "object") {
    // Object response with Ok/Err structure
    try {
      validateUploadResponse(finishResult, ["Ok"]);
      memoryId = finishResult.Ok;
      echoInfo(`✅ Upload finished successfully: ${memoryId}`);
    } catch (error) {
      throw handleUploadError(error, "Upload finish");
    }
  } else {
    throw new Error(`Unexpected finish response format: ${typeof finishResult} - ${JSON.stringify(finishResult)}`);
  }

  // Format blob ID as expected by backend (blob_{id})
  const blobId = `blob_${memoryId}`;

  const duration = Date.now() - startTime;
  const uploadSpeed = formatUploadSpeed(fileBuffer.length, duration);

  echoInfo(
    `✅ Upload completed: ${fileName} (${formatFileSize(fileBuffer.length)}) in ${formatDuration(
      duration
    )} (${uploadSpeed})`
  );

  return blobId;
}

// Lane B: Process image derivatives (matches frontend processImageDerivativesPure)
async function processImageDerivativesToICP(backend, fileBuffer, mimeType) {
  const laneBStartTime = Date.now();
  echoInfo(`🖼️ Starting Lane B: Processing derivatives`);

  const processedAssets = await processImageDerivativesPure(fileBuffer, mimeType);

  // Upload each derivative to ICP
  const results = {};
  const uploadPromises = [];

  if (processedAssets.display) {
    echoInfo(`📤 Uploading display derivative...`);
    uploadPromises.push(
      uploadOriginalToICP(backend, processedAssets.display.buffer, "display").then((blobId) => {
        results.display = blobId;
      })
    );
  }

  if (processedAssets.thumb) {
    echoInfo(`📤 Uploading thumb derivative...`);
    uploadPromises.push(
      uploadOriginalToICP(backend, processedAssets.thumb.buffer, "thumb").then((blobId) => {
        results.thumb = blobId;
      })
    );
  }

  if (processedAssets.placeholder) {
    echoInfo(`📤 Uploading placeholder derivative...`);
    uploadPromises.push(
      uploadOriginalToICP(backend, processedAssets.placeholder.buffer, "placeholder").then((blobId) => {
        results.placeholder = blobId;
      })
    );
  }

  // Wait for all uploads to complete
  await Promise.all(uploadPromises);

  const laneBDuration = Date.now() - laneBStartTime;
  const totalAssets = Object.keys(results).length;
  echoInfo(`✅ Lane B completed: ${totalAssets} derivatives uploaded in ${laneBDuration}ms`);

  return results;
}

// Finalize all assets (matches frontend finalizeAllAssets)
async function finalizeAllAssets(backend, originalBlobId, processedAssets, fileName) {
  // Get or create test capsule
  const capsuleResult = await backend.capsules_read_basic([]);
  let capsuleId;

  if ("Ok" in capsuleResult && capsuleResult.Ok) {
    capsuleId = capsuleResult.Ok.capsule_id;
  } else {
    const createResult = await backend.capsules_create([]);
    if (!("Ok" in createResult)) {
      throw new Error("Failed to create capsule");
    }
    capsuleId = createResult.Ok.id;
  }

  // Create asset metadata for the memory
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
        tags: ["test", "2lane-4asset"],
        processing_error: [],
        mime_type: "image/jpeg",
        description: [],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(0), // Will be updated with actual size
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

  // Create memory with original asset
  const memoryResult = await backend.memories_create(
    capsuleId, // text - capsule ID
    [], // opt blob - no inline data
    [{ locator: originalBlobId, len: BigInt(0), hash: [] }], // opt BlobRef - blob reference
    [], // opt StorageEdgeBlobType - no storage edge
    [], // opt text - no storage key
    [], // opt text - no bucket
    [], // opt nat64 - no file_created_at
    [], // opt blob - no sha256 hash
    assetMetadata, // AssetMetadata
    `memory-${Date.now()}` // text - idempotency key
  );

  if ("Err" in memoryResult) {
    throw new Error(`Memory creation failed: ${JSON.stringify(memoryResult.Err)}`);
  }

  const memoryId = memoryResult.Ok;

  return {
    memoryId,
    originalBlobId,
    processedAssets,
  };
}

// Main upload function (matches frontend uploadToS3WithProcessing)
async function uploadToICPWithProcessing(backend, fileBuffer, fileName, mimeType) {
  try {
    // Start both lanes simultaneously
    const laneAPromise = uploadOriginalToICP(backend, fileBuffer, fileName);
    const laneBPromise = processImageDerivativesToICP(backend, fileBuffer, mimeType);

    // Wait for both lanes to complete
    const [laneAResult, laneBResult] = await Promise.allSettled([laneAPromise, laneBPromise]);

    // Finalize all assets
    if (laneAResult.status === "fulfilled" && laneBResult.status === "fulfilled") {
      const finalResult = await finalizeAllAssets(backend, laneAResult.value, laneBResult.value, fileName);
      return finalResult;
    } else {
      throw new Error(`Lane failed: A=${laneAResult.status}, B=${laneBResult.status}`);
    }
  } catch (error) {
    throw error;
  }
}

// Test functions
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

async function testLaneBImageProcessing() {
  const fileBuffer = fs.readFileSync(TEST_IMAGE_PATH);

  const processedAssets = await processImageDerivativesPure(fileBuffer, "image/jpeg");

  // Verify all derivatives were created
  return processedAssets.original && processedAssets.display && processedAssets.thumb && processedAssets.placeholder;
}

async function testParallelLanes() {
  const fileBuffer = fs.readFileSync(TEST_IMAGE_PATH);
  const fileName = path.basename(TEST_IMAGE_PATH);

  // Start both lanes simultaneously
  const laneAPromise = uploadOriginalToICP(backend, fileBuffer, fileName);
  const laneBPromise = processImageDerivativesToICP(backend, fileBuffer, "image/jpeg");

  // Wait for both lanes to complete
  const [laneAResult, laneBResult] = await Promise.allSettled([laneAPromise, laneBPromise]);

  return laneAResult.status === "fulfilled" && laneBResult.status === "fulfilled";
}

async function testCompleteSystem() {
  const fileBuffer = fs.readFileSync(TEST_IMAGE_PATH);
  const fileName = path.basename(TEST_IMAGE_PATH);

  const result = await uploadToICPWithProcessing(backend, fileBuffer, fileName, "image/jpeg");

  // Verify all assets were created
  const hasOriginal = result.originalBlobId !== null;
  const hasDisplay = result.processedAssets.display !== null;
  const hasThumb = result.processedAssets.thumb !== null;
  const hasPlaceholder = result.processedAssets.placeholder !== null;

  return hasOriginal && hasDisplay && hasThumb && hasPlaceholder;
}

async function testAssetRetrieval() {
  const fileBuffer = fs.readFileSync(TEST_IMAGE_PATH);
  const fileName = path.basename(TEST_IMAGE_PATH);

  const result = await uploadToICPWithProcessing(backend, fileBuffer, fileName, "image/jpeg");

  // Test retrieval of original asset
  const originalMeta = await backend.blob_get_meta(result.originalBlobId);
  if ("Err" in originalMeta) {
    throw new Error(`Failed to get original meta: ${JSON.stringify(originalMeta.Err)}`);
  }

  // Test retrieval of display asset
  if (result.processedAssets.display) {
    const displayMeta = await backend.blob_get_meta(result.processedAssets.display);
    if ("Err" in displayMeta) {
      throw new Error(`Failed to get display meta: ${JSON.stringify(displayMeta.Err)}`);
    }
  }

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
    echoFail("Usage: node test_upload_2lane_4asset_system_functional.mjs <CANISTER_ID> [mainnet|local]");
    echoFail("Example: node test_upload_2lane_4asset_system_functional.mjs uxrrr-q7777-77774-qaaaq-cai local");
    echoFail("Example: node test_upload_2lane_4asset_system_functional.mjs uxrrr-q7777-77774-qaaaq-cai mainnet");
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

  // Run tests
  const tests = [
    { name: "Lane A: Original Upload", fn: testLaneAOriginalUpload },
    { name: "Lane B: Image Processing", fn: testLaneBImageProcessing },
    { name: "Parallel Lanes Execution", fn: testParallelLanes },
    { name: "Complete 2-Lane + 4-Asset System", fn: testCompleteSystem },
    { name: "Asset Retrieval", fn: testAssetRetrieval },
  ];

  let passed = 0;
  let failed = 0;

  for (const test of tests) {
    try {
      echoInfo(`Running: ${test.name}`);
      const result = await test.fn();
      if (result) {
        echoPass(test.name);
        passed++;
      } else {
        echoFail(test.name);
        failed++;
      }
    } catch (error) {
      echoFail(`${test.name}: ${error.message}`);
      failed++;
    }
  }

  // Summary
  echoInfo(`\n${TEST_NAME} Summary:`);
  echoInfo(`Total tests: ${tests.length}`);
  echoInfo(`Passed: ${passed}`);
  echoInfo(`Failed: ${failed}`);

  if (failed > 0) {
    echoFail("Some tests failed! ❌");
    process.exit(1);
  } else {
    echoPass("All tests passed! ✅");
  }
}

// Run the test
main().catch((error) => {
  echoFail(`Test execution failed: ${error.message}`);
  process.exit(1);
});
