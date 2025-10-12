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

import {
  parseTestArgs,
  createTestActorOptions,
  createTestActor,
  logNetworkConfig,
  getOrCreateTestCapsuleForUpload,
  createTestRunner,
  uploadFileAsBlob,
  uploadBufferAsBlob,
  createMemoryFromBlob,
  readFileAsBuffer,
  getFileSize,
  computeSHA256Hash,
  createImageAssetMetadata,
  createMemoryMetadata,
  createDerivativeAssetMetadata,
  verifyBlobIntegrity,
  verifyMemoryIntegrity,
  processImageDerivativesPure,
} from "../../utils/index.js";
import { formatFileSize } from "../../utils/helpers/logging.js";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

// Test configuration
const TEST_NAME = "2-Lane + 4-Asset Upload System Test";
const TEST_IMAGE_PATH = "./assets/input/avocado_big_21mb.jpg";
const TEST_IMAGE_PATH_SMALL = "./assets/input/orange_tiny.jpg"; // 44KB for Lane A tests
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

// Image processing function moved to shared utilities: processImageDerivativesPure

// Lane A: Upload original file to ICP (matches frontend uploadOriginalToS3)
async function uploadOriginalToICP(backend, fileBuffer, fileName, capsuleId) {
  const startTime = Date.now();

  echoInfo(`📤 Uploading: ${fileName} (${formatFileSize(fileBuffer.length)})`);

  // Validate file size (basic check)
  if (fileBuffer.length === 0) {
    throw new Error("File is empty");
  }

  // Use shared buffer upload helper
  const uploadResult = await uploadBufferAsBlob(backend, fileBuffer, capsuleId, {
    createMemory: false, // Just blob, no memory
    idempotencyKey: `upload-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
  });

  if (!uploadResult.success) {
    throw new Error(`Upload failed: ${uploadResult.error}`);
  }

  const duration = Date.now() - startTime;
  const uploadSpeed = (fileBuffer.length / (duration / 1000) / 1024 / 1024).toFixed(2);
  const durationSeconds = (duration / 1000).toFixed(1);

  echoInfo(
    `✅ Upload completed: ${fileName} (${formatFileSize(
      fileBuffer.length
    )}) in ${durationSeconds}s (${uploadSpeed} MB/s)`
  );

  return uploadResult.blobId;
}

// Lane B: Process image derivatives (matches frontend processImageDerivativesPure)
async function processImageDerivativesToICP(backend, fileBuffer, mimeType, capsuleId) {
  const laneBStartTime = Date.now();
  echoInfo(`🖼️ Starting Lane B: Processing derivatives`);

  const processedAssets = await processImageDerivativesPure(fileBuffer, mimeType);

  // Upload each derivative to ICP using shared helper
  const results = {};
  const uploadPromises = [];

  if (processedAssets.display) {
    echoInfo(`📤 Uploading display derivative...`);
    uploadPromises.push(
      uploadBufferAsBlob(backend, processedAssets.display.buffer, capsuleId, {
        createMemory: false,
        idempotencyKey: `display-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      }).then((uploadResult) => {
        if (uploadResult.success) {
          results.display = uploadResult.blobId;
        } else {
          throw new Error(`Display upload failed: ${uploadResult.error}`);
        }
      })
    );
  }

  if (processedAssets.thumb) {
    echoInfo(`📤 Uploading thumb derivative...`);
    uploadPromises.push(
      uploadBufferAsBlob(backend, processedAssets.thumb.buffer, capsuleId, {
        createMemory: false,
        idempotencyKey: `thumb-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      }).then((uploadResult) => {
        if (uploadResult.success) {
          results.thumb = uploadResult.blobId;
        } else {
          throw new Error(`Thumb upload failed: ${uploadResult.error}`);
        }
      })
    );
  }

  if (processedAssets.placeholder) {
    echoInfo(`📤 Storing placeholder derivative as inline data...`);
    // Store placeholder data directly (not as blob) - it will be inline in the memory
    results.placeholder = processedAssets.placeholder.buffer;
  }

  // Wait for all uploads to complete
  await Promise.all(uploadPromises);

  const laneBDuration = Date.now() - laneBStartTime;
  const totalAssets = Object.keys(results).length;
  echoInfo(`✅ Lane B completed: ${totalAssets} derivatives uploaded in ${laneBDuration}ms`);

  return results;
}

// Finalize all assets (matches frontend finalizeAllAssets)
// Creates a complete memory with all assets (original + derivatives)
async function finalizeAllAssets(backend, originalBlobId, results, fileName, capsuleId) {
  // Create memory metadata using helper function
  const memoryMetadata = createMemoryMetadata({
    title: fileName,
    description: "2-Lane + 4-Asset System Test Memory",
    tags: ["test", "2lane-4asset"],
    contentType: "image/jpeg",
    memoryType: { Image: null },
  });

  // Create asset metadata for the original blob using helper function
  const assetMetadata = createImageAssetMetadata({
    name: fileName,
    size: 0, // Will be updated with actual size
    mimeType: "image/jpeg",
    assetType: "Original",
    tags: ["test", "2lane-4asset", "original"],
    description: "Original image asset",
  });

  // Create asset metadata for derivatives using helper function
  const derivativeAssetMetadata = createDerivativeAssetMetadata({
    tags: ["test", "2lane-4asset", "derivative"],
    description: "Derivative image asset",
  });

  // Create memory with 3 blob assets (original + 2 derivatives) + 1 inline asset (placeholder)
  const blobAssets = [
    { blob_id: originalBlobId, metadata: assetMetadata },
    { blob_id: results.display, metadata: derivativeAssetMetadata },
    { blob_id: results.thumb, metadata: derivativeAssetMetadata },
  ];

  // Create inline asset for placeholder using actual processed data
  const placeholderInlineAsset = {
    data: results.placeholder, // Use actual processed placeholder data
    metadata: createDerivativeAssetMetadata({
      name: "placeholder",
      size: results.placeholder.length,
      mimeType: "image/webp",
      tags: ["test", "2lane-4asset", "derivative", "inline"],
      description: "Inline placeholder asset",
    }),
  };

  echoInfo(`📝 Creating memory with ${blobAssets.length} blob assets + 1 inline asset...`);
  echoInfo(`  Original: ${originalBlobId}`);
  echoInfo(`  Display: ${results.display}`);
  echoInfo(`  Thumb: ${results.thumb}`);
  echoInfo(`  Placeholder: inline asset (${placeholderInlineAsset.data.length} bytes)`);

  const memoryResult = await backend.memories_create_with_internal_blobs_and_inline_assets(
    capsuleId, // text - capsule ID
    memoryMetadata, // MemoryMetadata
    blobAssets, // Vec<InternalBlobAssetInput> - 3 blob assets
    [placeholderInlineAsset], // Vec<InlineAssetInput> - 1 inline asset
    `memory-${Date.now()}` // text - idempotency key
  );

  if ("Err" in memoryResult) {
    echoFail(`Memory creation failed: ${JSON.stringify(memoryResult.Err)}`);
    throw new Error(`Memory creation failed: ${JSON.stringify(memoryResult.Err)}`);
  }

  const memoryId = memoryResult.Ok;

  return {
    memoryId,
    originalBlobId,
    processedAssets: {
      display: results.display,
      thumb: results.thumb,
      placeholder: "inline", // Indicate placeholder is stored inline
    },
  };
}

// Main upload function (matches frontend uploadToS3WithProcessing)
async function uploadToICPWithProcessing(backend, fileBuffer, fileName, mimeType) {
  try {
    // Create a capsule for this upload session
    const capsuleResult = await backend.capsules_create([]);
    if ("Err" in capsuleResult) {
      throw new Error(`Capsule creation failed: ${JSON.stringify(capsuleResult.Err)}`);
    }
    const capsuleId = capsuleResult.Ok.id;

    // Start both lanes simultaneously with shared capsule
    const laneAPromise = uploadOriginalToICP(backend, fileBuffer, fileName, capsuleId);
    const laneBPromise = processImageDerivativesToICP(backend, fileBuffer, mimeType, capsuleId);

    // Wait for both lanes to complete
    const [laneAResult, laneBResult] = await Promise.allSettled([laneAPromise, laneBPromise]);

    // Finalize all assets
    if (laneAResult.status === "fulfilled" && laneBResult.status === "fulfilled") {
      const finalResult = await finalizeAllAssets(backend, laneAResult.value, laneBResult.value, fileName, capsuleId);
      return finalResult;
    } else {
      throw new Error(`Lane failed: A=${laneAResult.status}, B=${laneBResult.status}`);
    }
  } catch (error) {
    throw error;
  }
}

// Test functions
async function testLaneAOriginalUpload(backend, capsuleId) {
  try {
    console.log("🧪 Testing Lane A: Original Upload + Memory Creation");

    // Step 1: Upload image file as blob using our shared utility (use small file for verification)
    const uploadResult = await uploadFileAsBlob(backend, TEST_IMAGE_PATH_SMALL, capsuleId, {
      createMemory: false, // Just blob first, no memory
      idempotencyKey: `lane-a-${Date.now()}`,
    });

    if (!uploadResult.success) {
      return { success: false, error: `Blob upload failed: ${uploadResult.error}` };
    }

    console.log(`✅ Blob uploaded successfully - Blob ID: ${uploadResult.blobId}`);

    // Step 2: Verify blob integrity using our shared utility
    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH_SMALL);
    const fileSize = fileBuffer.length;
    const fileHash = computeSHA256Hash(fileBuffer);

    const blobVerification = await verifyBlobIntegrity(backend, uploadResult.blobId, fileSize, fileHash);
    if (!blobVerification) {
      return { success: false, error: "Blob integrity verification failed" };
    }

    console.log("✅ Blob integrity verified");

    // Step 3: Create memory from the blob using our shared utility
    const fileName = path.basename(TEST_IMAGE_PATH_SMALL);
    const memoryResult = await createMemoryFromBlob(
      backend,
      capsuleId,
      fileName,
      fileSize,
      uploadResult.blobId,
      uploadResult, // upload result object
      {
        assetType: "image",
        mimeType: "image/jpeg",
        memoryType: { Image: null },
      }
    );

    if (!memoryResult.success) {
      return { success: false, error: `Memory creation failed: ${memoryResult.error}` };
    }

    console.log(`✅ Memory created successfully - Memory ID: ${memoryResult.memoryId}`);

    // Step 4: Verify memory integrity using our shared utility
    const memoryVerification = await verifyMemoryIntegrity(backend, memoryResult.memoryId, 1);
    if (!memoryVerification) {
      return { success: false, error: "Memory integrity verification failed" };
    }

    console.log("✅ Memory integrity verified");

    return {
      success: true,
      blobId: uploadResult.blobId,
      memoryId: memoryResult.memoryId,
    };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

async function testLaneBImageProcessing(backend, capsuleId) {
  try {
    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);

    const processedAssets = await processImageDerivativesPure(fileBuffer, "image/jpeg");

    // Verify all derivatives were created
    const success =
      processedAssets.original && processedAssets.display && processedAssets.thumb && processedAssets.placeholder;
    return { success };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

async function testParallelLanes(backend, capsuleId) {
  try {
    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);
    const fileName = path.basename(TEST_IMAGE_PATH);

    // Create a capsule for this test
    const capsuleResult = await backend.capsules_create([]);
    if ("Err" in capsuleResult) {
      throw new Error(`Capsule creation failed: ${JSON.stringify(capsuleResult.Err)}`);
    }
    const testCapsuleId = capsuleResult.Ok.id;

    // Start both lanes simultaneously with shared capsule
    const laneAPromise = uploadOriginalToICP(backend, fileBuffer, fileName, testCapsuleId);
    const laneBPromise = processImageDerivativesToICP(backend, fileBuffer, "image/jpeg", testCapsuleId);

    // Wait for both lanes to complete
    const [laneAResult, laneBResult] = await Promise.allSettled([laneAPromise, laneBPromise]);

    const success = laneAResult.status === "fulfilled" && laneBResult.status === "fulfilled";
    return { success };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

async function testCompleteSystem(backend, capsuleId) {
  try {
    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);
    const fileName = path.basename(TEST_IMAGE_PATH);

    const result = await uploadToICPWithProcessing(backend, fileBuffer, fileName, "image/jpeg");

    // Verify all assets were created
    const hasOriginal = result.originalBlobId !== null;
    const hasDisplay = result.processedAssets.display !== null;
    const hasThumb = result.processedAssets.thumb !== null;
    const hasPlaceholder = result.processedAssets.placeholder === "inline"; // Placeholder is stored inline

    const success = hasOriginal && hasDisplay && hasThumb && hasPlaceholder;
    return { success };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

async function testAssetRetrieval(backend, capsuleId) {
  try {
    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);
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

    return { success: true };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

// Main test runner
async function main() {
  echoInfo(`Starting ${TEST_NAME}`);

  // Parse command line arguments
  const parsedArgs = parseTestArgs(
    "test_upload_2lane_4asset_system.mjs",
    "Tests the complete 2-lane + 4-asset upload system"
  );

  // Extract test filter from arguments (look for quoted test name or unquoted test name)
  let testFilter = null;
  const testNameArg = process.argv.find((arg) => arg.startsWith("'") || arg.startsWith('"'));
  if (testNameArg) {
    testFilter = testNameArg.slice(1, -1);
  } else {
    // Look for test name without quotes (everything after the canister ID)
    const args = process.argv.slice(2);
    const canisterIdIndex = args.findIndex((arg) => !arg.startsWith("--"));
    if (canisterIdIndex !== -1 && args.length > canisterIdIndex + 1) {
      testFilter = args[canisterIdIndex + 1];
    }
  }

  try {
    // Create test actor
    const options = createTestActorOptions(parsedArgs);
    const { actor: backend, canisterId } = await createTestActor(options);

    // Log network configuration
    logNetworkConfig(parsedArgs, canisterId);

    // Get or create test capsule
    const capsuleId = await getOrCreateTestCapsuleForUpload(backend);
    echoInfo(`Using capsule: ${capsuleId}`);

    // Create test runner
    const runner = createTestRunner(TEST_NAME);

    // Run tests
    const allTests = [
      { name: "Lane A: Original Upload", fn: testLaneAOriginalUpload },
      { name: "Lane B: Image Processing", fn: testLaneBImageProcessing },
      { name: "Parallel Lanes Execution", fn: testParallelLanes },
      { name: "Complete 2-Lane + 4-Asset System", fn: testCompleteSystem },
      { name: "Asset Retrieval", fn: testAssetRetrieval },
    ];

    // Filter tests if test name is provided
    const tests = testFilter ? allTests.filter((test) => test.name === testFilter) : allTests;

    if (testFilter && tests.length === 0) {
      echoFail(`Test not found: "${testFilter}"`);
      echoFail("Available tests:");
      allTests.forEach((test) => echoFail(`  - "${test.name}"`));
      process.exit(1);
    }

    if (testFilter) {
      echoInfo(`Running single test: "${testFilter}"`);
    } else {
      echoInfo(`Running all ${tests.length} tests`);
    }

    // Run all tests
    for (const test of tests) {
      await runner.runTest(test.name, test.fn, backend, capsuleId);
    }

    // Print summary and exit
    const allPassed = runner.printTestSummary();
    process.exit(allPassed ? 0 : 1);
  } catch (error) {
    echoFail(`Test execution failed: ${error.message}`);
    process.exit(1);
  }
}

// Run the test
main().catch((error) => {
  echoFail(`Test execution failed: ${error.message}`);
  process.exit(1);
});
