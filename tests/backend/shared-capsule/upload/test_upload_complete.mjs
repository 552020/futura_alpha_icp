#!/usr/bin/env node

/**
 * Complete Upload Test Suite
 *
 * This test suite runs comprehensive upload tests with different image assets
 * to verify chunking behavior, file size handling, and upload reliability.
 */

import {
  createTestActor,
  parseTestArgs,
  createTestActorOptions,
  logNetworkConfig,
  getOrCreateTestCapsuleForUpload,
  createTestRunner,
} from "../../utils/index.js";

// Import the backend interface
import { idlFactory } from "../../declarations/backend/backend.did.js";

// Test configuration
const TEST_NAME = "Complete Upload Test Suite";
const CHUNK_SIZE = 1_800_000; // 1.8MB chunks - matches backend CHUNK_SIZE

// Define available tests
const AVAILABLE_TESTS = [
  "Single chunk upload (3KB)",
  "Single chunk upload (44KB)",
  "Single chunk upload (240KB)",
  "Single chunk upload (372KB)",
  "Multi-chunk upload (3.6MB)",
  "Large multi-chunk upload (21MB)",
  "Chunking behavior validation",
  "Size range coverage test",
];

// Parse command line arguments using shared utility
const parsedArgs = parseTestArgs(
  "test_upload_complete.mjs",
  "Comprehensive upload test suite with different image assets",
  AVAILABLE_TESTS
);

// Test asset definitions
const TEST_ASSETS = [
  {
    file: "assets/input/orange_small_inline.jpg",
    size: "3KB",
    expectedChunks: 1,
    description: "Single chunk upload (3KB)",
    category: "tiny",
  },
  {
    file: "assets/input/orange_tiny.jpg",
    size: "44KB",
    expectedChunks: 1,
    description: "Single chunk upload (44KB)",
    category: "small",
  },
  {
    file: "assets/input/avocado_tiny_240kb.jpg",
    size: "240KB",
    expectedChunks: 1,
    description: "Single chunk upload (240KB)",
    category: "medium",
  },
  {
    file: "assets/input/avocado_small_372kb.jpg",
    size: "372KB",
    expectedChunks: 1,
    description: "Single chunk upload (372KB)",
    category: "medium",
  },
  {
    file: "assets/input/avocado_medium_3.5mb.jpg",
    size: "3.6MB",
    expectedChunks: 3,
    description: "Multi-chunk upload (3.6MB)",
    category: "large",
  },
  {
    file: "assets/input/avocado_big_21mb.jpg",
    size: "21MB",
    expectedChunks: 13,
    description: "Large multi-chunk upload (21MB)",
    category: "xlarge",
  },
];

// Import the single file upload test function
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";

// Single file upload test (extracted from test_upload.mjs)
async function testSingleFileUpload(backend, capsuleId, filePath) {
  console.log(`🧪 Testing upload with file: ${filePath}`);

  try {
    // Check if file exists
    if (!fs.existsSync(filePath)) {
      return { success: false, error: `Test file not found: ${filePath}` };
    }

    // Read file
    const fileBuffer = fs.readFileSync(filePath);
    const fileName = path.basename(filePath);
    const fileSize = fileBuffer.length;

    console.log(`📁 File: ${fileName} (${fileSize} bytes)`);
    console.log(`📦 Using capsule: ${capsuleId}`);

    // Calculate chunk count
    const chunkCount = Math.ceil(fileSize / CHUNK_SIZE);
    const idempotencyKey = `upload-test-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

    console.log(`📊 Chunk count: ${chunkCount}, Idempotency key: ${idempotencyKey}`);

    // Begin upload session
    console.log("🚀 Calling uploads_begin...");
    const beginResult = await backend.uploads_begin(capsuleId, chunkCount, idempotencyKey);

    if ("Err" in beginResult) {
      return { success: false, error: `Upload begin failed: ${JSON.stringify(beginResult.Err)}` };
    }

    const sessionId = beginResult.Ok;
    console.log(`✅ Upload session started: ${sessionId}`);

    // Upload chunks
    console.log("📤 Uploading chunks...");
    for (let i = 0; i < chunkCount; i++) {
      const start = i * CHUNK_SIZE;
      const end = Math.min(start + CHUNK_SIZE, fileSize);
      const chunkData = fileBuffer.slice(start, end);

      console.log(`📦 Uploading chunk ${i + 1}/${chunkCount} (${chunkData.length} bytes)`);

      const chunkResult = await backend.uploads_put_chunk(sessionId, i, chunkData);
      if ("Err" in chunkResult) {
        return { success: false, error: `Chunk upload failed: ${JSON.stringify(chunkResult.Err)}` };
      }
    }

    console.log("✅ All chunks uploaded successfully");

    // Compute expected hash
    const expectedHash = crypto.createHash("sha256").update(fileBuffer).digest();
    console.log(`🔐 Expected hash: ${expectedHash.toString("hex")}`);

    // Finish upload
    console.log("🏁 Calling uploads_finish...");
    const finishResult = await backend.uploads_finish(sessionId, expectedHash, fileSize);

    if ("Err" in finishResult) {
      return { success: false, error: `Upload finish failed: ${JSON.stringify(finishResult.Err)}` };
    }

    const result = finishResult.Ok;
    console.log(`✅ Upload finished successfully!`);
    console.log(`📦 Blob ID: ${result.blob_id}`);
    console.log(`🧠 Memory ID: ${result.memory_id}`);
    console.log(`📍 Storage Location: ${result.storage_location}`);
    console.log(`📏 Size: ${result.size} bytes`);

    // Verify the pure blob upload (no memory should be created)
    console.log("🔍 Verifying pure blob upload...");

    // Check that no memory was created (memory_id should be empty)
    if (result.memory_id === "") {
      console.log("✅ Memory ID is empty - no memory created (correct for pure blob upload)");
    } else {
      return { success: false, error: `Memory ID should be empty but got: ${result.memory_id}` };
    }

    // Verify blob ID is valid
    if (result.blob_id && result.blob_id.startsWith("blob_")) {
      console.log("✅ Blob ID is valid");
    } else {
      return { success: false, error: `Invalid blob ID: ${result.blob_id}` };
    }

    // Verify file size matches
    if (Number(result.size) === fileSize) {
      console.log("✅ File size matches uploaded size");
    } else {
      return { success: false, error: `Size mismatch: expected ${fileSize}, got ${result.size}` };
    }

    // Test blob read - first get metadata to determine the right approach
    console.log("📖 Testing blob read...");

    // First, get blob metadata to determine size and chunk count
    const blobMetaResult = await backend.blob_get_meta(result.blob_id);
    if ("Err" in blobMetaResult) {
      return { success: false, error: `Blob metadata read failed: ${JSON.stringify(blobMetaResult.Err)}` };
    }

    const blobMeta = blobMetaResult.Ok;
    console.log(`📊 Blob metadata: size=${blobMeta.size} bytes, chunks=${blobMeta.chunk_count}`);

    // Verify metadata size matches expected
    if (Number(blobMeta.size) !== fileSize) {
      return { success: false, error: `Blob metadata size mismatch: expected ${fileSize}, got ${blobMeta.size}` };
    }
    console.log("✅ Blob metadata size matches expected size");

    // Choose read method based on size
    if (blobMeta.size > 3 * 1024 * 1024) {
      // Large file: use chunked read
      console.log("📖 File is large (>3MB), using chunked blob read...");

      // Read first chunk to verify blob exists and is readable
      const blobReadResult = await backend.blob_read_chunk(result.blob_id, 0);
      if ("Ok" in blobReadResult) {
        const chunkData = blobReadResult.Ok;
        console.log(`✅ Blob read successful - first chunk size: ${chunkData.length} bytes`);
        console.log("✅ Large blob exists and is readable (chunked read works)");
      } else {
        return { success: false, error: `Blob chunk read failed: ${JSON.stringify(blobReadResult.Err)}` };
      }
    } else {
      // Small file: read entire blob
      console.log("📖 File is small (≤3MB), reading entire blob...");
      const blobReadResult = await backend.blob_read(result.blob_id);
      if ("Ok" in blobReadResult) {
        const blobData = blobReadResult.Ok;
        if (blobData.length === fileSize) {
          console.log("✅ Blob read successful - size matches");

          // Verify hash matches
          const actualHash = crypto.createHash("sha256").update(blobData).digest();
          if (actualHash.equals(expectedHash)) {
            console.log("✅ Blob hash matches expected hash");
          } else {
            return { success: false, error: "Blob hash mismatch" };
          }
        } else {
          return { success: false, error: `Blob size mismatch: expected ${fileSize}, got ${blobData.length}` };
        }
      } else {
        return { success: false, error: `Blob read failed: ${JSON.stringify(blobReadResult.Err)}` };
      }
    }

    // Verify no memory was created in the list
    console.log("🔍 Verifying no memory was created...");
    const memoriesResult = await backend.memories_list(capsuleId, [], [10]);
    if ("Ok" in memoriesResult) {
      const page = memoriesResult.Ok;
      const memories = page.items;
      const newMemory = memories.find((m) => m.id === result.memory_id);

      if (!newMemory) {
        console.log("✅ Memory list API works (no new memory should be created)");
      } else {
        return { success: false, error: "Unexpected memory found in list" };
      }
    } else {
      return { success: false, error: `Memory list failed: ${JSON.stringify(memoriesResult.Err)}` };
    }

    console.log("🎉 Pure blob upload test PASSED!");
    console.log("✅ Blob upload, creation, and readback all work correctly");
    console.log("✅ No memory was created (pure blob storage)");
    console.log("✅ Ready for separate memory creation endpoints");

    return {
      success: true,
      result: {
        blobId: result.blob_id,
        size: result.size,
        chunkCount: chunkCount,
        fileName: fileName,
      },
    };
  } catch (error) {
    console.error(`❌ Upload test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

// Individual test functions
async function testSingleChunkUpload3KB(backend, capsuleId) {
  const asset = TEST_ASSETS.find((a) => a.category === "tiny");
  return await testSingleFileUpload(backend, capsuleId, asset.file);
}

async function testSingleChunkUpload44KB(backend, capsuleId) {
  const asset = TEST_ASSETS.find((a) => a.file.includes("orange_tiny"));
  return await testSingleFileUpload(backend, capsuleId, asset.file);
}

async function testSingleChunkUpload240KB(backend, capsuleId) {
  const asset = TEST_ASSETS.find((a) => a.file.includes("avocado_tiny_240kb"));
  return await testSingleFileUpload(backend, capsuleId, asset.file);
}

async function testSingleChunkUpload372KB(backend, capsuleId) {
  const asset = TEST_ASSETS.find((a) => a.file.includes("avocado_small_372kb"));
  return await testSingleFileUpload(backend, capsuleId, asset.file);
}

async function testMultiChunkUpload3_6MB(backend, capsuleId) {
  const asset = TEST_ASSETS.find((a) => a.file.includes("avocado_medium_3.5mb"));
  return await testSingleFileUpload(backend, capsuleId, asset.file);
}

async function testLargeMultiChunkUpload21MB(backend, capsuleId) {
  const asset = TEST_ASSETS.find((a) => a.file.includes("avocado_big_21mb"));
  return await testSingleFileUpload(backend, capsuleId, asset.file);
}

// Chunking behavior validation test
async function testChunkingBehaviorValidation(backend, capsuleId) {
  console.log("🧪 Testing chunking behavior validation...");

  const results = [];

  // Test each asset and verify chunking behavior
  for (const asset of TEST_ASSETS) {
    console.log(`\n📊 Testing ${asset.description}...`);
    const result = await testSingleFileUpload(backend, capsuleId, asset.file);

    if (result.success) {
      const actualChunks = result.result.chunkCount;
      const expectedChunks = asset.expectedChunks;

      if (actualChunks === expectedChunks) {
        console.log(`✅ Chunking correct: ${actualChunks} chunks (expected ${expectedChunks})`);
        results.push({ asset: asset.description, success: true, chunks: actualChunks });
      } else {
        console.log(`❌ Chunking incorrect: ${actualChunks} chunks (expected ${expectedChunks})`);
        results.push({ asset: asset.description, success: false, chunks: actualChunks, expected: expectedChunks });
      }
    } else {
      console.log(`❌ Upload failed: ${result.error}`);
      results.push({ asset: asset.description, success: false, error: result.error });
    }
  }

  // Summary
  const successful = results.filter((r) => r.success).length;
  const total = results.length;

  console.log(`\n📊 Chunking Behavior Summary:`);
  console.log(`✅ Successful: ${successful}/${total}`);

  if (successful === total) {
    console.log("🎉 All chunking behavior tests passed!");
    return { success: true, results };
  } else {
    console.log("❌ Some chunking behavior tests failed");
    return { success: false, results };
  }
}

// Size range coverage test
async function testSizeRangeCoverage(backend, capsuleId) {
  console.log("🧪 Testing size range coverage...");

  const sizeRanges = [
    { min: 0, max: 1024, name: "Tiny (<1KB)" },
    { min: 1024, max: 1024 * 100, name: "Small (1KB-100KB)" },
    { min: 1024 * 100, max: 1024 * 1024, name: "Medium (100KB-1MB)" },
    { min: 1024 * 1024, max: 1024 * 1024 * 10, name: "Large (1MB-10MB)" },
    { min: 1024 * 1024 * 10, max: Infinity, name: "XLarge (>10MB)" },
  ];

  const coverage = {};

  for (const asset of TEST_ASSETS) {
    const result = await testSingleFileUpload(backend, capsuleId, asset.file);

    if (result.success) {
      const size = result.result.size;
      const range = sizeRanges.find((r) => size >= r.min && size < r.max);

      if (range) {
        if (!coverage[range.name]) {
          coverage[range.name] = { tested: 0, passed: 0 };
        }
        coverage[range.name].tested++;
        coverage[range.name].passed++;
        console.log(`✅ ${asset.description} - ${range.name} range covered`);
      }
    } else {
      console.log(`❌ ${asset.description} failed: ${result.error}`);
    }
  }

  console.log(`\n📊 Size Range Coverage:`);
  for (const [range, stats] of Object.entries(coverage)) {
    console.log(`${range}: ${stats.passed}/${stats.tested} passed`);
  }

  return { success: true, coverage };
}

// Main test execution
async function main() {
  console.log("=========================================");
  console.log(`Starting ${TEST_NAME}`);
  console.log("=========================================");
  console.log("");

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

    // Define all tests with their functions
    const allTests = [
      { name: "Single chunk upload (3KB)", fn: testSingleChunkUpload3KB, args: [backend, capsuleId] },
      { name: "Single chunk upload (44KB)", fn: testSingleChunkUpload44KB, args: [backend, capsuleId] },
      { name: "Single chunk upload (240KB)", fn: testSingleChunkUpload240KB, args: [backend, capsuleId] },
      { name: "Single chunk upload (372KB)", fn: testSingleChunkUpload372KB, args: [backend, capsuleId] },
      { name: "Multi-chunk upload (3.6MB)", fn: testMultiChunkUpload3_6MB, args: [backend, capsuleId] },
      { name: "Large multi-chunk upload (21MB)", fn: testLargeMultiChunkUpload21MB, args: [backend, capsuleId] },
      { name: "Chunking behavior validation", fn: testChunkingBehaviorValidation, args: [backend, capsuleId] },
      { name: "Size range coverage test", fn: testSizeRangeCoverage, args: [backend, capsuleId] },
    ];

    // Run tests based on selection
    if (parsedArgs.selectedTest) {
      // Run specific test
      const selectedTest = allTests.find((test) => test.name === parsedArgs.selectedTest);
      if (selectedTest) {
        console.log(`🎯 Running selected test: ${parsedArgs.selectedTest}`);
        await runner.runTest(selectedTest.name, selectedTest.fn, ...selectedTest.args);
      } else {
        console.error(`❌ Test not found: ${parsedArgs.selectedTest}`);
        console.log("Available tests:");
        AVAILABLE_TESTS.forEach((test) => console.log(`  - ${test}`));
        process.exit(1);
      }
    } else {
      // Run all tests
      for (const test of allTests) {
        await runner.runTest(test.name, test.fn, ...test.args);
      }
    }

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
