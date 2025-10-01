#!/usr/bin/env node

/**
 * 2-Lane + 4-Asset Upload System Test
 *
 * This test reproduces the S3 2-lane + 4-asset system using ICP backend:
 * - Lane A: Upload original file to ICP blob storage
 * - Lane B: Process image derivatives (display, thumb, placeholder)
 * - Finalize: Create memory with all 4 asset types
 *
 * This validates the concept before implementing in the frontend.
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity, makeMainnetAgent } from "./ic-identity.js";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

// Import the backend interface
import { idlFactory } from "../../../../src/nextjs/src/ic/declarations/backend/backend.did.js";

// Test configuration
const TEST_NAME = "2-Lane + 4-Asset Upload System Test";
let totalTests = 0;
let passedTests = 0;
let failedTests = 0;

// Global backend instance
let backend;

// Test assets
const TEST_IMAGE_PATH = "./assets/input/avocado_medium.jpg";
const CHUNK_SIZE = 1_800_000;

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

// Test runner
async function runTest(testName, testFunction) {
  echoInfo(`Running: ${testName}`);
  totalTests++;

  try {
    const result = await testFunction();
    if (result) {
      echoPass(testName);
      passedTests++;
    } else {
      echoFail(testName);
      failedTests++;
    }
  } catch (error) {
    echoError(`${testName}: ${error.message}`);
    failedTests++;
  }
}

// Image processing utilities (simplified version of frontend worker)
class ImageProcessor {
  constructor() {
    this.supportedFormats = ["image/jpeg", "image/png", "image/webp"];
  }

  async processImageDerivatives(fileBuffer, mimeType) {
    if (!this.supportedFormats.includes(mimeType)) {
      throw new Error(`Unsupported format: ${mimeType}`);
    }

    // For this test, we'll simulate the processing by creating different sized versions
    // In a real implementation, this would use canvas/image processing libraries
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
}

// Upload utilities (functional approach to match frontend S3 system)
async function uploadFileToBlob(backend, fileBuffer, fileName) {
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
      const putChunkResult = await this.backend.uploads_put_chunk(sessionId, i, chunks[i]);
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
    const crypto = await import("crypto");
    const hash = crypto.createHash("sha256").update(fileBuffer).digest();
    const totalLen = BigInt(fileBuffer.length);

    // Finish upload
    const finishResult = await this.backend.uploads_finish(sessionId, Array.from(hash), totalLen);
    if ("Err" in finishResult) {
      throw new Error(`Upload finish failed: ${JSON.stringify(finishResult.Err)}`);
    }

    // Format blob ID as expected by backend (blob_{id})
    return `blob_${finishResult.Ok}`;
  }

  async createMemoryWithAssets(originalBlobId, processedAssets, fileName) {
    // Get or create test capsule
    const capsuleResult = await this.backend.capsules_read_basic([]);
    let capsuleId;

    if ("Ok" in capsuleResult && capsuleResult.Ok) {
      capsuleId = capsuleResult.Ok.capsule_id;
    } else {
      const createResult = await this.backend.capsules_create([]);
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
    const memoryResult = await this.backend.memories_create(
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

    // Create additional assets for display, thumb, and placeholder
    const assetResults = [];

    // Upload display asset
    if (processedAssets.display) {
      const displayBlobId = await this.uploadFileToBlob(processedAssets.display.buffer, `${fileName}_display`);
      assetResults.push({
        assetType: "display",
        blobId: displayBlobId,
        size: processedAssets.display.size,
        width: processedAssets.display.width,
        height: processedAssets.display.height,
        mimeType: processedAssets.display.mimeType,
      });
    }

    // Upload thumb asset
    if (processedAssets.thumb) {
      const thumbBlobId = await this.uploadFileToBlob(processedAssets.thumb.buffer, `${fileName}_thumb`);
      assetResults.push({
        assetType: "thumb",
        blobId: thumbBlobId,
        size: processedAssets.thumb.size,
        width: processedAssets.thumb.width,
        height: processedAssets.thumb.height,
        mimeType: processedAssets.thumb.mimeType,
      });
    }

    // For placeholder, we'll store it as a note (inline data)
    if (processedAssets.placeholder) {
      const placeholderNote = await this.backend.memories_create(
        capsuleId, // text - capsule ID
        Array.from(processedAssets.placeholder.buffer), // opt blob - inline data
        [], // opt BlobRef - no blob reference
        [], // opt StorageEdgeBlobType - no storage edge
        [], // opt text - no storage key
        [], // opt text - no bucket
        [], // opt nat64 - no file_created_at
        [], // opt blob - no sha256 hash
        { Note: null }, // AssetMetadata
        `${fileName}_placeholder` // text - idempotency key
      );

      if ("Ok" in placeholderNote) {
        assetResults.push({
          assetType: "placeholder",
          memoryId: placeholderNote.Ok,
          size: processedAssets.placeholder.size,
          width: processedAssets.placeholder.width,
          height: processedAssets.placeholder.height,
          mimeType: processedAssets.placeholder.mimeType,
        });
      }
    }

    return {
      memoryId,
      originalBlobId,
      assets: assetResults,
    };
  }
}

// Main test functions
async function testLaneAOriginalUpload() {
  const fileBuffer = fs.readFileSync(TEST_IMAGE_PATH);
  const fileName = path.basename(TEST_IMAGE_PATH);

  const uploader = new ICPUploader(backend);
  const blobId = await uploader.uploadFileToBlob(fileBuffer, fileName);

  // Verify blob was created
  const blobMeta = await backend.blob_get_meta(blobId);
  if ("Err" in blobMeta) {
    throw new Error(`Failed to get blob meta: ${JSON.stringify(blobMeta.Err)}`);
  }

  return blobMeta.Ok.size === BigInt(fileBuffer.length);
}

async function testLaneBImageProcessing() {
  const fileBuffer = fs.readFileSync(TEST_IMAGE_PATH);
  const processor = new ImageProcessor();

  const processedAssets = await processor.processImageDerivatives(fileBuffer, "image/jpeg");

  // Verify all derivatives were created
  return processedAssets.original && processedAssets.display && processedAssets.thumb && processedAssets.placeholder;
}

async function testParallelLanes() {
  const fileBuffer = fs.readFileSync(TEST_IMAGE_PATH);
  const fileName = path.basename(TEST_IMAGE_PATH);

  const uploader = new ICPUploader(backend);
  const processor = new ImageProcessor();

  // Start both lanes simultaneously
  const laneAPromise = uploader.uploadFileToBlob(fileBuffer, fileName);
  const laneBPromise = processor.processImageDerivatives(fileBuffer, "image/jpeg");

  // Wait for both to complete
  const [laneAResult, laneBResult] = await Promise.all([laneAPromise, laneBPromise]);

  // Verify both lanes completed successfully
  return laneAResult && laneBResult;
}

async function testComplete2Lane4AssetSystem() {
  const fileBuffer = fs.readFileSync(TEST_IMAGE_PATH);
  const fileName = path.basename(TEST_IMAGE_PATH);

  const uploader = new ICPUploader(backend);
  const processor = new ImageProcessor();

  // Step 1: Start both lanes simultaneously
  const laneAPromise = uploader.uploadFileToBlob(fileBuffer, fileName);
  const laneBPromise = processor.processImageDerivatives(fileBuffer, "image/jpeg");

  // Step 2: Wait for both lanes to complete
  const [originalBlobId, processedAssets] = await Promise.all([laneAPromise, laneBPromise]);

  // Step 3: Create memory with all 4 assets
  const result = await uploader.createMemoryWithAssets(originalBlobId, processedAssets, fileName);

  // Verify all assets were created
  const hasOriginal = result.originalBlobId !== null;
  const hasDisplay = result.assets.some((asset) => asset.assetType === "display");
  const hasThumb = result.assets.some((asset) => asset.assetType === "thumb");
  const hasPlaceholder = result.assets.some((asset) => asset.assetType === "placeholder");

  return hasOriginal && hasDisplay && hasThumb && hasPlaceholder;
}

async function testAssetRetrieval() {
  const fileBuffer = fs.readFileSync(TEST_IMAGE_PATH);
  const fileName = path.basename(TEST_IMAGE_PATH);

  const uploader = new ICPUploader(backend);
  const processor = new ImageProcessor();

  // Create complete system
  const laneAPromise = uploader.uploadFileToBlob(fileBuffer, fileName);
  const laneBPromise = processor.processImageDerivatives(fileBuffer, "image/jpeg");
  const [originalBlobId, processedAssets] = await Promise.all([laneAPromise, laneBPromise]);
  const result = await uploader.createMemoryWithAssets(originalBlobId, processedAssets, fileName);

  // Test retrieval of original asset
  const originalMeta = await backend.blob_get_meta(result.originalBlobId);
  if ("Err" in originalMeta) {
    throw new Error(`Failed to get original meta: ${originalMeta.Err}`);
  }

  // Test retrieval of display asset
  const displayAsset = result.assets.find((asset) => asset.assetType === "display");
  if (displayAsset) {
    const displayMeta = await backend.blob_get_meta(displayAsset.blobId);
    if ("Err" in displayMeta) {
      throw new Error(`Failed to get display meta: ${displayMeta.Err}`);
    }
  }

  return true;
}

// Main test execution
async function main() {
  echoInfo(`Starting ${TEST_NAME}`);

  // Get backend canister ID
  const backendCanisterId = process.argv[2];
  if (!backendCanisterId) {
    echoError("Usage: node test_upload_2lane_4asset_system.mjs <BACKEND_CANISTER_ID>");
    process.exit(1);
  }

  // Setup agent and backend (using local setup like working tests)
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

  // Run tests
  await runTest("Lane A: Original Upload", testLaneAOriginalUpload);
  await runTest("Lane B: Image Processing", testLaneBImageProcessing);
  await runTest("Parallel Lanes Execution", testParallelLanes);
  await runTest("Complete 2-Lane + 4-Asset System", testComplete2Lane4AssetSystem);
  await runTest("Asset Retrieval", testAssetRetrieval);

  // Summary
  echoInfo(`\n${TEST_NAME} Summary:`);
  echoInfo(`Total tests: ${totalTests}`);
  echoInfo(`Passed: ${passedTests}`);
  echoInfo(`Failed: ${failedTests}`);

  if (failedTests === 0) {
    echoPass("All tests passed! 🎉");
    process.exit(0);
  } else {
    echoFail("Some tests failed! ❌");
    process.exit(1);
  }
}

// Run the tests
main().catch((error) => {
  echoError(`Test execution failed: ${error.message}`);
  process.exit(1);
});
