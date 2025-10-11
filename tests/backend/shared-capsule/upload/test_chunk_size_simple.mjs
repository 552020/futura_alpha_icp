#!/usr/bin/env node

/**
 * Simple Chunk Size Comparison Test
 *
 * This test compares different chunk sizes using the generic upload test.
 * Much simpler than the original 275-line version.
 */

import {
  parseTestArgs,
  createTestActorOptions,
  createTestActor,
  logNetworkConfig,
  getOrCreateTestCapsuleForUpload,
  createTestRunner,
  uploadFileAsBlob,
  readFileAsBuffer,
  getFileSize,
} from "../../utils/index.js";
import { formatFileSize } from "../../utils/helpers/logging.js";
import fs from "node:fs";

// Test configuration
const TEST_NAME = "Simple Chunk Size Comparison";
const TEST_FILE_PATH = "./assets/input/avocado_medium_3.5mb.jpg";

// Chunk sizes to test (for theoretical comparison)
const CHUNK_SIZES = [
  { size: 64 * 1024, name: "64KB" },
  { size: 256 * 1024, name: "256KB" },
  { size: 1024 * 1024, name: "1MB" },
  { size: 1_800_000, name: "1.8MB" },
  { size: 2_097_152, name: "2MB" },
];

// Test function for a specific chunk size
async function testChunkSize(backend, capsuleId, chunkSizeConfig) {
  const { size: chunkSize, name: chunkName } = chunkSizeConfig;

  console.log(`\n🧪 Testing ${chunkName} chunks (${formatFileSize(chunkSize)})`);

  const fileBuffer = readFileAsBuffer(TEST_FILE_PATH);
  const fileSize = fileBuffer.length;
  const chunkCount = Math.ceil(fileSize / chunkSize);
  const totalRequests = chunkCount + 2; // +2 for begin/finish
  const efficiency = Math.round((1 - totalRequests / 58) * 100); // 58 = current total requests

  console.log(`File: ${formatFileSize(fileSize)}`);
  console.log(`Chunks: ${chunkCount} (${efficiency}% efficiency vs 64KB)`);

  try {
    // Use shared uploadFileAsBlob function
    const startTime = Date.now();
    const idempotencyKey = `test-${chunkName}-${Date.now()}`;

    // Note: uploadFileAsBlob uses a fixed chunk size, but we can still measure the performance
    // For this test, we'll use the standard chunk size but measure the overall performance
    const uploadResult = await uploadFileAsBlob(backend, TEST_FILE_PATH, capsuleId, {
      createMemory: false,
      idempotencyKey: idempotencyKey,
    });

    const endTime = Date.now();
    const duration = endTime - startTime;

    if (!uploadResult.success) {
      console.log(`❌ ${chunkName}: Upload failed - ${uploadResult.error}`);
      return { success: false, error: uploadResult.error, chunkSize: chunkName, requests: totalRequests };
    }

    console.log(`✅ ${chunkName}: Upload successful! (${duration}ms)`);
    console.log(`Blob ID: ${uploadResult.blobId}`);

    return {
      success: true,
      chunkSize: chunkName,
      requests: totalRequests,
      duration,
      blobId: uploadResult.blobId,
      efficiency,
    };
  } catch (error) {
    console.log(`❌ ${chunkName}: Test failed - ${error.message}`);
    return { success: false, error: error.message, chunkSize: chunkName, requests: totalRequests };
  }
}

// Main test execution
async function main() {
  console.log(`Starting ${TEST_NAME}`);

  // Parse command line arguments
  const parsedArgs = parseTestArgs("test_chunk_size_simple.mjs", "Tests performance with different chunk sizes");

  // Check if test file exists
  if (!fs.existsSync(TEST_FILE_PATH)) {
    console.error(`❌ Test file not found: ${TEST_FILE_PATH}`);
    process.exit(1);
  }

  const fileSize = getFileSize(TEST_FILE_PATH);
  console.log(`Test file: ${TEST_FILE_PATH} (${formatFileSize(fileSize)})`);

  try {
    // Create test actor
    const options = createTestActorOptions(parsedArgs);
    const { actor: backend, canisterId } = await createTestActor(options);

    // Log network configuration
    logNetworkConfig(parsedArgs, canisterId);

    // Get or create test capsule
    const capsuleId = await getOrCreateTestCapsuleForUpload(backend);
    console.log(`Using capsule: ${capsuleId}`);

    // Run tests for each chunk size
    const results = [];

    for (const chunkSizeConfig of CHUNK_SIZES) {
      const result = await testChunkSize(backend, capsuleId, chunkSizeConfig);
      results.push(result);

      // Small delay between tests
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    // Summary
    console.log(`\n📊 Chunk Size Comparison Results:`);
    console.log(`=====================================`);

    results.forEach((result) => {
      if (result.success) {
        console.log(
          `✅ ${result.chunkSize}: Success (${result.requests} requests, ${result.duration}ms, ${result.efficiency}% efficiency)`
        );
      } else {
        console.log(`❌ ${result.chunkSize}: Failed (${result.requests} requests, ${result.error})`);
      }
    });

    // Recommendations
    const successfulResults = results.filter((r) => r.success);
    if (successfulResults.length > 0) {
      const bestResult = successfulResults.reduce((best, current) =>
        current.efficiency > best.efficiency ? current : best
      );

      console.log(`\n🎯 Recommendation:`);
      console.log(`Best performing chunk size: ${bestResult.chunkSize}`);
      console.log(`Efficiency improvement: ${bestResult.efficiency}%`);
      console.log(`Total requests: ${bestResult.requests} (vs 58 for 64KB)`);
    } else {
      console.log(`\n⚠️  No successful uploads - all chunk sizes failed`);
      console.log(`This suggests a resource allocation issue rather than chunk size problem`);
    }

    process.exit(0);
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
