#!/usr/bin/env node

/**
 * Memory Creation with Internal Blobs Test
 *
 * This test performs a complete workflow:
 * 1. Upload a blob using chunked upload
 * 2. Create a memory with that blob as an internal asset
 * 3. Verify the memory was created correctly
 */

import path from "node:path";
import {
  createTestActor,
  parseTestArgs,
  createTestActorOptions,
  logNetworkConfig,
  getOrCreateTestCapsuleForUpload,
  createTestRunner,
  uploadFileAsBlob,
  createMemoryFromBlob,
  getFileSize,
  fileExists,
  verifyCompleteUploadWorkflow,
} from "../../utils/index.js";

// Import the backend interface
import { idlFactory } from "../../declarations/backend/backend.did.js";

// Test configuration
const TEST_NAME = "Memory Creation with Internal Blobs Test";

// Parse command line arguments using shared utility
const parsedArgs = parseTestArgs(
  "test_memory_creation_with_internal_blobs.mjs",
  "Tests memory creation with internal blob assets"
);

// Override canister ID to use the one from command line
const args = process.argv.slice(2);
const canisterIdArg = args.find((arg) => !arg.startsWith("--") && !arg.includes("/"));
if (canisterIdArg) {
  parsedArgs.canisterId = canisterIdArg;
}

// Main test function
async function testMemoryCreationWithInternalBlobs(backend, filePath, capsuleId) {
  const fileName = path.basename(filePath);
  console.log(`🧪 Testing memory creation with internal blobs`);
  console.log(`📁 File: ${fileName}`);
  console.log(`📁 Path: ${filePath}`);

  try {
    // Check if file exists
    if (!fileExists(filePath)) {
      return { success: false, error: `File not found: ${filePath}` };
    }

    // Get file size
    const fileSize = getFileSize(filePath);
    console.log(`📏 Size: ${fileSize} bytes`);

    // Step 1: Upload blob using helper function
    console.log("🚀 Step 1: Uploading blob...");
    const uploadResult = await uploadFileAsBlob(backend, filePath, capsuleId, {
      createMemory: false,
      idempotencyKey: `memory-test-${Date.now()}`,
    });

    if (!uploadResult.success) {
      return { success: false, error: uploadResult.error };
    }

    const blobId = uploadResult.blobId;
    console.log(`✅ Blob upload successful!`);
    console.log(`📦 Blob ID: ${blobId}`);

    // Step 2: Create memory with internal blob using helper function
    console.log("🧠 Step 2: Creating memory with internal blob...");
    const memoryResult = await createMemoryFromBlob(backend, capsuleId, fileName, fileSize, blobId, uploadResult, {
      assetType: "image",
      mimeType: "image/jpeg",
      memoryType: { Image: null },
      idempotencyKey: `memory-${Date.now()}`,
    });

    if (!memoryResult.success) {
      return { success: false, error: memoryResult.error };
    }

    const memoryId = memoryResult.memoryId;
    console.log(`✅ Memory created successfully!`);
    console.log(`🧠 Memory ID: ${memoryId}`);

    // Step 3: Verify complete workflow using verification helpers
    console.log("🔍 Step 3: Verifying complete workflow...");
    const workflowVerified = await verifyCompleteUploadWorkflow(backend, capsuleId, filePath, uploadResult, memoryId);

    if (!workflowVerified) {
      return { success: false, error: "Complete workflow verification failed" };
    }

    console.log("🎉 Memory creation with internal blobs test PASSED!");
    console.log("✅ Blob upload works correctly");
    console.log("✅ Memory creation with internal blob works");
    console.log("✅ Memory can be read and verified");
    console.log("✅ Blob data integrity verified");

    return {
      success: true,
      result: {
        memoryId,
        blobId,
        fileSize,
        fileName,
      },
    };
  } catch (error) {
    console.error(`❌ Memory creation test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

// Main test execution
async function main() {
  console.log("=========================================");
  console.log(`Starting ${TEST_NAME}`);
  console.log("=========================================");
  console.log("");

  // Get file path from command line arguments (after flags)
  const args = process.argv.slice(2);
  const filePath = args.find((arg) => !arg.startsWith("--") && arg.includes("/"));

  if (!filePath) {
    console.error("Usage: node test_memory_creation_with_internal_blobs.mjs [OPTIONS] <CANISTER_ID> <FILE_PATH>");
    console.error(
      "Example: node test_memory_creation_with_internal_blobs.mjs --local uxrrr-q7777-77774-qaaaq-cai assets/input/avocado.jpg"
    );
    process.exit(1);
  }

  try {
    // Create test actor using shared utilities
    console.log("Loading DFX identity...");
    const options = createTestActorOptions(parsedArgs);
    const { actor: backend, agent, canisterId } = await createTestActor(options);

    // Log network configuration using shared utility
    logNetworkConfig(parsedArgs, canisterId);

    // Get or create a test capsule using shared utility
    const capsuleId = await getOrCreateTestCapsuleForUpload(backend);

    // Create test runner using shared utility
    const runner = createTestRunner(TEST_NAME);

    // Run the memory creation test
    await runner.runTest(
      "Memory creation with internal blobs",
      testMemoryCreationWithInternalBlobs,
      backend,
      filePath,
      capsuleId
    );

    // Print test summary using shared utility
    const allPassed = runner.printTestSummary();

    if (allPassed) {
      process.exit(0);
    } else {
      process.exit(1);
    }
  } catch (error) {
    console.error("❌ Test execution failed:", error.message);
    process.exit(1);
  }
}

// Run main function if script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}
