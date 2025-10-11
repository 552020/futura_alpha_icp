#!/usr/bin/env node

/**
 * Lane A: Original Upload Test
 *
 * Tests the complete Lane A workflow:
 * 1. Upload image file to ICP as a blob
 * 2. Create memory from that blob
 * 3. Verify both blob and memory were created correctly
 *
 * This is a focused test that validates the core upload + memory creation workflow
 * using our shared utilities and a small test file for fast execution.
 */

import {
  parseTestArgs,
  createTestActorOptions,
  createTestActor,
  logNetworkConfig,
  getOrCreateTestCapsuleForUpload,
  createTestRunner,
  uploadFileAsBlob,
  createMemoryFromBlob,
  readFileAsBuffer,
  getFileSize,
  computeSHA256Hash,
  verifyBlobIntegrity,
  verifyMemoryIntegrity,
} from "../../utils/index.js";
import { formatFileSize } from "../../utils/helpers/logging.js";
import path from "node:path";

// Test configuration
const TEST_NAME = "Lane A: Original Upload Test";
const TEST_IMAGE_PATH = "./assets/input/orange_tiny.jpg"; // 44KB file for fast testing

// Test function for Lane A: Original Upload + Memory Creation
async function testLaneAOriginalUpload(backend, capsuleId) {
  console.log("🧪 Testing Lane A: Original Upload + Memory Creation");

  try {
    // Check if test file exists
    if (!(await import("node:fs").then((fs) => fs.existsSync(TEST_IMAGE_PATH)))) {
      return { success: false, error: `Test file not found: ${TEST_IMAGE_PATH}` };
    }

    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);
    const fileName = path.basename(TEST_IMAGE_PATH);
    const fileSize = fileBuffer.length;
    const fileHash = computeSHA256Hash(fileBuffer);

    console.log(`📁 Test file: ${fileName} (${formatFileSize(fileSize)})`);

    // Step 1: Upload image file as blob
    console.log("📤 Step 1: Uploading image file as blob...");
    const uploadResult = await uploadFileAsBlob(backend, TEST_IMAGE_PATH, capsuleId, {
      createMemory: false, // Just blob first, no memory
      idempotencyKey: `lane-a-${Date.now()}`,
    });

    if (!uploadResult.success) {
      return { success: false, error: `Blob upload failed: ${uploadResult.error}` };
    }

    console.log(`✅ Blob uploaded successfully - Blob ID: ${uploadResult.blobId}`);

    // Step 2: Verify blob integrity
    console.log("🔍 Step 2: Verifying blob integrity...");
    const blobVerification = await verifyBlobIntegrity(backend, uploadResult.blobId, fileSize, fileHash);

    if (!blobVerification) {
      return { success: false, error: "Blob integrity verification failed" };
    }

    console.log("✅ Blob integrity verified");

    // Step 3: Create memory from the blob
    console.log("📝 Step 3: Creating memory from blob...");
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

    // Step 4: Verify memory integrity
    console.log("🔍 Step 4: Verifying memory integrity...");
    const memoryVerification = await verifyMemoryIntegrity(
      backend,
      memoryResult.memoryId,
      1 // expected 1 blob asset
    );

    if (!memoryVerification) {
      return { success: false, error: "Memory integrity verification failed" };
    }

    console.log("✅ Memory integrity verified");

    // Step 5: Verify complete workflow
    console.log("🔍 Step 5: Verifying complete workflow...");

    // Check that memory contains the correct blob
    const memoryRead = await backend.memories_read(memoryResult.memoryId);
    if ("Err" in memoryRead) {
      return { success: false, error: `Failed to read memory: ${JSON.stringify(memoryRead.Err)}` };
    }

    const memory = memoryRead.Ok;

    // Check if memory has blob_internal_assets (the correct property name)
    if (memory.blob_internal_assets && memory.blob_internal_assets.length !== 1) {
      return { success: false, error: `Expected 1 internal blob asset, got ${memory.blob_internal_assets.length}` };
    }

    if (memory.blob_internal_assets && memory.blob_internal_assets.length > 0) {
      const blobAsset = memory.blob_internal_assets[0];
      if (blobAsset.blob_ref.locator !== uploadResult.blobId) {
        return { success: false, error: "Memory contains wrong blob ID" };
      }
    } else {
      return { success: false, error: "Memory has no internal blob assets" };
    }

    console.log("✅ Complete workflow verified");

    return {
      success: true,
      blobId: uploadResult.blobId,
      memoryId: memoryResult.memoryId,
      fileSize: fileSize,
      fileName: fileName,
    };
  } catch (error) {
    console.log(`❌ Lane A test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

// Main test runner
async function main() {
  console.log(`Starting ${TEST_NAME}`);

  // Parse command line arguments
  const parsedArgs = parseTestArgs(
    "test_lane_a_original_upload.mjs",
    "Tests Lane A: Original upload + memory creation workflow"
  );

  try {
    // Create test actor
    const options = createTestActorOptions(parsedArgs);
    const { actor: backend, canisterId } = await createTestActor(options);

    // Log network configuration
    logNetworkConfig(parsedArgs, canisterId);

    // Get or create test capsule
    const capsuleId = await getOrCreateTestCapsuleForUpload(backend);
    console.log(`Using capsule: ${capsuleId}`);

    // Create test runner
    const runner = createTestRunner(TEST_NAME);

    // Run the test
    await runner.runTest("Lane A: Original Upload + Memory Creation", testLaneAOriginalUpload, backend, capsuleId);

    // Print summary and exit
    const allPassed = runner.printTestSummary();
    process.exit(allPassed ? 0 : 1);
  } catch (error) {
    console.error(`❌ Test execution failed: ${error.message}`);
    process.exit(1);
  }
}

// Run the test
main().catch((error) => {
  console.error(`❌ Test execution failed: ${error.message}`);
  process.exit(1);
});
