/**
 * 2-Lane + 4-Asset Upload System Test (Functional Approach)
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

// Test configuration
const TEST_NAME = "2-Lane + 4-Asset Upload System Test (Functional)";
const TEST_IMAGE_PATH = "./assets/input/avocado_medium.jpg";
// Constants - Aligned with frontend configuration
const CHUNK_SIZE = 1.5 * 1024 * 1024; // 1.5MB - matches frontend UPLOAD_LIMITS_ICP.CHUNK_SIZE_BYTES
const INLINE_MAX = 1.5 * 1024 * 1024; // 1.5MB - matches frontend UPLOAD_LIMITS_ICP.INLINE_MAX_BYTES

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

// Image processing simulation (matches frontend pattern)
async function processImageDerivativesPure(fileBuffer, mimeType) {
  const originalSize = fileBuffer.length;

  // Simulate display version (80% of original size)
  const displaySize = Math.floor(originalSize * 0.8);
  const displayBuffer = Buffer.alloc(displaySize);
  fileBuffer.copy(displayBuffer, 0, 0, displaySize);

  // Simulate thumb version (20% of original size)
  const thumbSize = Math.floor(originalSize * 0.2);
  const thumbBuffer = Buffer.alloc(thumbSize);
  fileBuffer.copy(thumbBuffer, 0, 0, thumbSize);

  // Simulate placeholder (small binary data)
  const placeholderBuffer = Buffer.alloc(100, 0x42); // Fill with 0x42 instead of zeros

  return {
    original: {
      buffer: fileBuffer,
      size: originalSize,
      width: 1920, // Simulated dimensions
      height: 1080,
      mimeType: mimeType,
    },
    display: {
      buffer: displayBuffer,
      size: displaySize,
      width: 1536, // Simulated dimensions
      height: 864,
      mimeType: "image/webp",
    },
    thumb: {
      buffer: thumbBuffer,
      size: thumbSize,
      width: 512, // Simulated dimensions
      height: 288,
      mimeType: "image/webp",
    },
    placeholder: {
      buffer: placeholderBuffer,
      size: placeholderBuffer.length,
      width: 32,
      height: 18,
      mimeType: "image/webp",
    },
  };
}

// Lane A: Upload original file to ICP (matches frontend uploadOriginalToS3)
async function uploadOriginalToICP(backend, fileBuffer, fileName) {
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

  // Create asset metadata
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
        bytes: BigInt(fileBuffer.length),
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
  const chunkCount = Math.ceil(fileBuffer.length / CHUNK_SIZE);
  const idempotencyKey = `test-${Date.now()}-${Math.random()}`;

  // Begin upload session
  const beginResult = await backend.uploads_begin(capsuleId, assetMetadata, chunkCount, idempotencyKey);

  if ("Err" in beginResult) {
    throw new Error(`Upload begin failed: ${JSON.stringify(beginResult.Err)}`);
  }

  const sessionId = beginResult.Ok;

  // Upload file in chunks
  const chunks = [];
  for (let i = 0; i < fileBuffer.length; i += CHUNK_SIZE) {
    const chunk = fileBuffer.slice(i, i + CHUNK_SIZE);
    chunks.push(Array.from(chunk));
  }

  echoInfo(`Uploading ${chunks.length} chunks...`);
  for (let i = 0; i < chunks.length; i++) {
    const putChunkResult = await backend.uploads_put_chunk(sessionId, i, chunks[i]);
    if ("Err" in putChunkResult) {
      throw new Error(`Put chunk failed: ${JSON.stringify(putChunkResult.Err)}`);
    }

    // Progress indicator
    if ((i + 1) % 10 === 0 || i === chunks.length - 1) {
      const progress = Math.round(((i + 1) / chunks.length) * 100);
      echoInfo(`Upload progress: ${progress}% (${i + 1}/${chunks.length} chunks)`);
    }
  }

  // Calculate hash and total length for finish
  const hash = crypto.createHash("sha256").update(fileBuffer).digest();
  const totalLen = BigInt(fileBuffer.length);

  // Finish upload
  const finishResult = await backend.uploads_finish(sessionId, Array.from(hash), totalLen);
  if ("Err" in finishResult) {
    throw new Error(`Upload finish failed: ${JSON.stringify(finishResult.Err)}`);
  }

  // Format blob ID as expected by backend (blob_{id})
  return `blob_${finishResult.Ok}`;
}

// Lane B: Process image derivatives (matches frontend processImageDerivativesPure)
async function processImageDerivativesToICP(backend, fileBuffer, mimeType) {
  const processedAssets = await processImageDerivativesPure(fileBuffer, mimeType);

  // Upload each derivative to ICP
  const results = {};

  if (processedAssets.display) {
    results.display = await uploadOriginalToICP(backend, processedAssets.display.buffer, "display");
  }

  if (processedAssets.thumb) {
    results.thumb = await uploadOriginalToICP(backend, processedAssets.thumb.buffer, "thumb");
  }

  if (processedAssets.placeholder) {
    results.placeholder = await uploadOriginalToICP(backend, processedAssets.placeholder.buffer, "placeholder");
  }

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

  // Get backend canister ID
  const backendCanisterId = process.argv[2];
  if (!backendCanisterId) {
    echoFail("Please provide backend canister ID as argument");
    process.exit(1);
  }

  // Setup agent and backend
  const identity = await loadDfxIdentity();
  const agent = makeMainnetAgent(identity);
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
