#!/usr/bin/env node

/**
 * Generic Upload Test
 *
 * This test uploads any file to verify chunked upload functionality.
 * Takes a file path as a command line argument.
 */

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
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
const TEST_NAME = "Generic Upload Test";
const CHUNK_SIZE = 1_800_000; // 1.8MB chunks - matches backend CHUNK_SIZE

// Parse command line arguments using shared utility
const parsedArgs = parseTestArgs("test_upload.mjs", "Tests generic chunked upload functionality with any file");

// Override canister ID to use the one from command line
const args = process.argv.slice(2);
const canisterIdArg = args.find((arg) => !arg.startsWith("--") && !arg.includes("/"));
if (canisterIdArg) {
  parsedArgs.canisterId = canisterIdArg;
}

// Main test function
async function testUpload(backend, capsuleId, filePath) {
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

    // Create asset metadata for image
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
          tags: ["test", "upload", "chunked-upload"],
          processing_error: [],
          mime_type: "image/jpeg",
          description: ["Test file for chunked upload"],
          created_at: BigInt(Date.now() * 1000000),
          deleted_at: [],
          bytes: BigInt(fileSize),
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
    const chunkCount = Math.ceil(fileSize / CHUNK_SIZE);
    const idempotencyKey = `upload-test-${Date.now()}`;

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

    return { success: true, result: { blobId: result.blob_id, size: result.size } };
  } catch (error) {
    console.error(`❌ Upload test failed: ${error.message}`);
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
    console.error("Usage: node test_upload.mjs [OPTIONS] <CANISTER_ID> <FILE_PATH>");
    console.error("Example: node test_upload.mjs --local uxrrr-q7777-77774-qaaaq-cai assets/input/avocado.jpg");
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

    // Run the upload test
    await runner.runTest("Generic file upload", testUpload, backend, capsuleId, filePath);

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
