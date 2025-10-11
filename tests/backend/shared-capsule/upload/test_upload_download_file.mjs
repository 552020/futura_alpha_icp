#!/usr/bin/env node

/**
 * Upload/Download Roundtrip Test
 *
 * This test performs a complete roundtrip: upload a file, create a memory,
 * download the file, and verify it matches the original.
 */

import path from "node:path";
import {
  createTestActor,
  parseTestArgs,
  createTestActorOptions,
  logNetworkConfig,
  getOrCreateTestCapsuleForUpload,
  createTestRunner,
  createMemoryWithInline,
  uploadFileAsBlob,
  downloadFileFromMemory,
  verifyDownloadedFile,
  getFileSize,
  fileExists,
  ensureDirectoryExists,
} from "../../utils/index.js";

// Import the backend interface
import { idlFactory } from "../../declarations/backend/backend.did.js";

// Test configuration
const TEST_NAME = "Upload/Download Roundtrip Test";
const CHUNK_SIZE = 1_800_000; // 1.8MB chunks - matches backend CHUNK_SIZE
const OUTPUT_DIR = "tests/backend/shared-capsule/upload/assets/output";

// Parse command line arguments using shared utility
const parsedArgs = parseTestArgs(
  "test_upload_download_file.mjs",
  "Tests complete upload/download roundtrip with memory creation"
);

// Override canister ID to use the one from command line
const args = process.argv.slice(2);
const canisterIdArg = args.find((arg) => !arg.startsWith("--") && !arg.includes("/"));
if (canisterIdArg) {
  parsedArgs.canisterId = canisterIdArg;
}

// Main test function - directly orchestrates helper functions
async function testFileUploadDownload(backend, filePath, capsuleId) {
  const fileName = path.basename(filePath);
  const testName = "File upload/download test";

  console.log("🧪 === File Upload and Download Test ===");
  console.log(`📁 File: ${fileName}`);
  console.log(`📁 Path: ${filePath}`);

  try {
    // Check if file exists
    if (!fileExists(filePath)) {
      return { success: false, error: `File not found: ${filePath}` };
    }

    const fileSize = getFileSize(filePath);
    console.log(`📏 Size: ${fileSize} bytes`);

    // Test decides: use inline upload for small files, blob upload for large files
    const useInlineUpload = fileSize <= 32768; // 32KB limit
    let memoryId;

    if (useInlineUpload) {
      console.log(`📄 Using inline memory creation for ${fileSize} bytes`);
      const uploadResult = await createMemoryWithInline(backend, filePath, capsuleId);
      if (!uploadResult.success) {
        return { success: false, error: uploadResult.error };
      }
      memoryId = uploadResult.memoryId;
    } else {
      console.log(`🚀 Using blob upload for ${fileSize} bytes`);
      const uploadResult = await uploadFileAsBlob(backend, filePath, capsuleId, { createMemory: true });
      if (!uploadResult.success) {
        return { success: false, error: uploadResult.error };
      }
      memoryId = uploadResult.memoryId;
    }

    // Create output directory if it doesn't exist
    ensureDirectoryExists(OUTPUT_DIR);

    // Download the file
    console.log("📥 Downloading file...");
    const outputPath = path.join(OUTPUT_DIR, `downloaded_${fileName}`);
    const downloadResult = await downloadFileFromMemory(backend, memoryId, outputPath);

    if (!downloadResult.success) {
      return { success: false, error: downloadResult.error };
    }

    // Verify the downloaded file
    console.log("🔍 Verifying downloaded file...");
    if (verifyDownloadedFile(filePath, outputPath)) {
      console.log("🎉 File upload and download test completed successfully!");
      console.log(`📁 Original file: ${filePath}`);
      console.log(`📁 Downloaded file: ${outputPath}`);
      return {
        success: true,
        result: {
          memoryId,
          outputPath,
          fileSize,
          uploadMethod: useInlineUpload ? "inline" : "blob",
        },
      };
    } else {
      return { success: false, error: "File verification failed" };
    }
  } catch (error) {
    console.error(`❌ Upload/download test failed: ${error.message}`);
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
    console.error("Usage: node test_upload_download_file.mjs [OPTIONS] <CANISTER_ID> <FILE_PATH>");
    console.error(
      "Example: node test_upload_download_file.mjs --local uxrrr-q7777-77774-qaaaq-cai assets/input/avocado.jpg"
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

    // Run the upload/download test
    await runner.runTest("Upload/Download roundtrip", testFileUploadDownload, backend, filePath, capsuleId);

    // Print test summary using shared utility
    const allPassed = runner.printTestSummary();

    if (allPassed) {
      console.log(`📁 Output directory: ${OUTPUT_DIR}`);
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
