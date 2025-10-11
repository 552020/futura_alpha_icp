#!/usr/bin/env node

/**
 * Debug test to understand session management during parallel uploads
 * This test adds detailed logging to track session lifecycle
 */

import { readFileSync } from "fs";
import { resolve } from "path";
import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { idlFactory } from "./declarations/backend/backend.did.js";
import { createAssetMetadata, createFileChunks, generateFileId } from "./helpers.mjs";

// Test configuration
const TEST_IMAGE_PATH = "./assets/input/avocado_big_21mb.jpg";
const BACKEND_CANISTER_ID = process.argv[2];

if (!BACKEND_CANISTER_ID) {
  console.error("❌ Backend canister ID required");
  console.error("Usage: node test_session_debug.mjs <BACKEND_CANISTER_ID>");
  process.exit(1);
}

// Helper functions
function echoInfo(message) {
  console.log(`ℹ️  ${message}`);
}

function echoSuccess(message) {
  console.log(`✅ ${message}`);
}

function echoError(message) {
  console.log(`❌ ${message}`);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Upload function with session tracking
async function uploadFileToICP(actor, fileBuffer, fileName, onProgress, delayMs = 0) {
  const fileSizeMB = (fileBuffer.length / (1024 * 1024)).toFixed(1);

  try {
    echoInfo(`📤 Uploading: ${fileName} (${fileSizeMB} MB)${delayMs > 0 ? ` [DELAY: ${delayMs}ms]` : ""}`);

    if (delayMs > 0) {
      await sleep(delayMs);
    }

    // Get or create test capsule
    const capsuleResult = await actor.capsules_read_basic([]);
    let capsuleId;

    if ("Ok" in capsuleResult && capsuleResult.Ok) {
      capsuleId = capsuleResult.Ok.capsule_id;
    } else {
      const createResult = await actor.capsules_create([]);
      if (!("Ok" in createResult)) {
        throw new Error("Failed to create capsule");
      }
      capsuleId = createResult.Ok.id;
    }

    // Create asset metadata
    const assetMetadata = createAssetMetadata(fileName, fileBuffer.length, "image/jpeg", "Original");

    // Calculate chunk count and create chunks
    const chunkSize = 1_800_000; // 1.8MB chunks
    const chunkCount = Math.ceil(fileBuffer.length / chunkSize);
    const chunks = createFileChunks(fileBuffer, chunkSize);
    const idempotencyKey = generateFileId("upload");

    echoInfo(`🔍 Creating session for ${fileName} with idempotency key: ${idempotencyKey}`);

    // Start upload session
    const beginResult = await actor.uploads_begin(capsuleId, assetMetadata, chunkCount, idempotencyKey);

    // Handle different response formats
    let sessionId;
    if (typeof beginResult === "number" || typeof beginResult === "string") {
      sessionId = beginResult;
    } else if (beginResult && typeof beginResult === "object") {
      if ("Ok" in beginResult) {
        sessionId = beginResult.Ok;
      } else {
        throw new Error(`Upload begin failed: ${JSON.stringify(beginResult.Err)}`);
      }
    } else {
      throw new Error(`Unexpected begin result format: ${typeof beginResult}`);
    }

    echoInfo(`✅ Upload session started: ${sessionId}`);

    // Check session stats before uploading
    try {
      const stats = await actor.sessions_stats();
      echoInfo(`📊 Session stats: ${stats}`);
    } catch (error) {
      echoInfo(`⚠️  Could not get session stats: ${error.message}`);
    }

    // Upload chunks
    echoInfo(`📦 Uploading ${chunkCount} chunks (${(chunkSize / (1024 * 1024)).toFixed(1)} MB each)...`);

    for (let i = 0; i < chunks.length; i++) {
      echoInfo(`🔍 Uploading chunk ${i}/${chunkCount} for session ${sessionId}`);

      const putChunkResult = await actor.uploads_put_chunk(sessionId, i, chunks[i]);

      // Handle different response formats for put_chunk
      if (typeof putChunkResult === "object" && putChunkResult !== null) {
        if ("Err" in putChunkResult) {
          echoError(`❌ Chunk ${i} upload failed: ${JSON.stringify(putChunkResult.Err)}`);
          throw new Error(`Chunk ${i} upload failed: ${JSON.stringify(putChunkResult.Err)}`);
        }
      }

      const percentage = Math.round(((i + 1) / chunkCount) * 100);
      if (percentage % 25 === 0 || i === chunkCount - 1) {
        echoInfo(`  📈 ${percentage}% (${i + 1}/${chunkCount} chunks)`);
      }

      if (onProgress) {
        onProgress({
          fileIndex: 0,
          totalFiles: 1,
          currentFile: fileName,
          bytesUploaded: (i + 1) * chunkSize,
          totalBytes: fileBuffer.length,
          percentage: percentage,
          status: "uploading",
          message: `Uploading chunk ${i + 1}/${chunkCount}`,
        });
      }
    }

    // Calculate hash
    const crypto = await import("node:crypto");
    const hash = crypto.createHash("sha256").update(fileBuffer).digest();
    const totalLen = BigInt(fileBuffer.length);

    // Finish upload
    const finishResult = await actor.uploads_finish(sessionId, Array.from(hash), totalLen);

    // Handle different response formats for finish
    let blobId, memoryId;
    if (typeof finishResult === "object" && finishResult !== null) {
      if ("Ok" in finishResult) {
        const uploadResult = finishResult.Ok;
        blobId = uploadResult.blob_id;
        memoryId = uploadResult.memory_id;
      } else {
        throw new Error(`Upload finish failed: ${JSON.stringify(finishResult.Err)}`);
      }
    } else {
      throw new Error(`Unexpected finish result format: ${typeof finishResult}`);
    }

    echoSuccess(`✅ Upload finished successfully: blob_id=${blobId}, memory_id=${memoryId}`);

    return {
      memoryId: memoryId,
      blobId: blobId,
      remoteId: memoryId,
      size: fileBuffer.length,
      checksumSha256: null,
      storageBackend: "icp",
      storageLocation: `icp://memory/${memoryId}`,
      uploadedAt: new Date(),
    };
  } catch (error) {
    echoError(`❌ Upload failed for ${fileName}: ${error.message}`);
    throw error;
  }
}

// Simulate image processing (same as main test)
function simulateImageProcessing(fileBuffer, fileName) {
  const fileSizeMB = fileBuffer.length / (1024 * 1024);

  // Simulate processing time based on file size
  const processingTime = Math.max(1000, fileSizeMB * 50);

  // Simulate derivative sizes (realistic for a 20MB image)
  const derivatives = {
    display: {
      size: Math.min(200 * 1024, fileSizeMB * 1024 * 0.01), // 200KB or 1% of original
      width: 1920,
      height: 1080,
      buffer: Buffer.alloc(Math.min(200 * 1024, fileSizeMB * 1024 * 0.01)),
    },
    thumb: {
      size: Math.min(50 * 1024, fileSizeMB * 1024 * 0.0025), // 50KB or 0.25% of original
      width: 300,
      height: 169,
      buffer: Buffer.alloc(Math.min(50 * 1024, fileSizeMB * 1024 * 0.0025)),
    },
    placeholder: {
      size: Math.min(1024, fileSizeMB * 1024 * 0.00005), // 1KB or 0.005% of original
      width: 32,
      height: 18,
      buffer: Buffer.alloc(Math.min(1024, fileSizeMB * 1024 * 0.00005)),
    },
  };

  return { processingTime, derivatives };
}

// Process derivatives with detailed error handling
async function processDerivatives(actor, fileBuffer, fileName, onProgress, delayMs = 0) {
  try {
    echoInfo(
      `🖼️ Processing derivatives for ${(fileBuffer.length / (1024 * 1024)).toFixed(1)} MB file${
        delayMs > 0 ? ` [DELAY: ${delayMs}ms]` : ""
      }`
    );

    if (delayMs > 0) {
      await sleep(delayMs);
    }

    const { processingTime, derivatives } = simulateImageProcessing(fileBuffer, fileName);

    echoInfo(`📊 Derivative sizes:`);
    echoInfo(
      `  Display: ${(derivatives.display.size / 1024).toFixed(1)} KB (${derivatives.display.width}x${
        derivatives.display.height
      })`
    );
    echoInfo(
      `  Thumb: ${(derivatives.thumb.size / 1024).toFixed(1)} KB (${derivatives.thumb.width}x${
        derivatives.thumb.height
      })`
    );
    echoInfo(
      `  Placeholder: ${(derivatives.placeholder.size / 1024).toFixed(1)} KB (${derivatives.placeholder.width}x${
        derivatives.placeholder.height
      })`
    );

    const results = {};

    // Upload display derivative
    echoInfo(`📤 Uploading display derivative...`);
    echoInfo(`📤 Uploading: display (${(derivatives.display.size / 1024).toFixed(1)} KB)`);
    results.display = await uploadFileToICP(actor, derivatives.display.buffer, "display", onProgress);

    // Upload thumb derivative
    echoInfo(`📤 Uploading thumb derivative...`);
    echoInfo(`📤 Uploading: thumb (${(derivatives.thumb.size / 1024).toFixed(1)} KB)`);
    results.thumb = await uploadFileToICP(actor, derivatives.thumb.buffer, "thumb", onProgress);

    // Upload placeholder derivative
    echoInfo(`📤 Uploading placeholder derivative...`);
    echoInfo(`📤 Uploading: placeholder (${(derivatives.placeholder.size / 1024).toFixed(1)} KB)`);
    results.placeholder = await uploadFileToICP(actor, derivatives.placeholder.buffer, "placeholder", onProgress);

    return results;
  } catch (error) {
    echoError(`❌ Derivative processing failed: ${error.message}`);
    throw error;
  }
}

// Main test function
async function testSessionDebug() {
  try {
    echoInfo("🔍 Starting Session Debug Test");
    echoInfo("This test adds detailed logging to understand session lifecycle during parallel uploads");

    // Load test image
    const imagePath = resolve(TEST_IMAGE_PATH);
    const fileBuffer = readFileSync(imagePath);
    const fileName = "avocado_big_21mb.jpg";

    echoInfo(`📁 Test image: ${fileName} (${(fileBuffer.length / (1024 * 1024)).toFixed(1)} MB)`);

    // Create actor
    const agent = new HttpAgent({ host: "http://127.0.0.1:4943" });
    await agent.fetchRootKey();
    const actor = Actor.createActor(idlFactory, {
      agent,
      canisterId: Principal.fromText(BACKEND_CANISTER_ID),
    });

    echoInfo("🔗 Connected to backend canister");

    // Check initial session stats
    try {
      const initialStats = await actor.sessions_stats();
      echoInfo(`📊 Initial session stats: ${initialStats}`);
    } catch (error) {
      echoInfo(`⚠️  Could not get initial session stats: ${error.message}`);
    }

    // Test: Parallel execution with detailed logging
    echoInfo("\n🧪 Test: Parallel Execution with Session Debug");
    try {
      const startTime = Date.now();

      // Run both lanes in parallel
      const [laneAResult, laneBResult] = await Promise.allSettled([
        uploadFileToICP(actor, fileBuffer, fileName),
        processDerivatives(actor, fileBuffer, fileName, null, 100), // Small delay to avoid exact collision
      ]);

      const endTime = Date.now();
      const duration = endTime - startTime;

      echoInfo(`⏱️  Parallel execution completed in ${duration}ms`);

      // Check final session stats
      try {
        const finalStats = await actor.sessions_stats();
        echoInfo(`📊 Final session stats: ${finalStats}`);
      } catch (error) {
        echoInfo(`⚠️  Could not get final session stats: ${error.message}`);
      }

      // Check results
      if (laneAResult.status === "fulfilled" && laneBResult.status === "fulfilled") {
        echoSuccess(`✅ Both lanes completed successfully`);
        echoSuccess(`  Lane A: ${laneAResult.value.memoryId}`);
        echoSuccess(`  Lane B: ${Object.keys(laneBResult.value).length} derivatives`);
      } else {
        echoError(`❌ Lane execution failed:`);
        echoError(`  Lane A: ${laneAResult.status}`);
        echoError(`  Lane B: ${laneBResult.status}`);

        if (laneAResult.status === "rejected") {
          echoError(`  Lane A error: ${laneAResult.reason.message}`);
        }
        if (laneBResult.status === "rejected") {
          echoError(`  Lane B error: ${laneBResult.reason.message}`);
        }

        throw new Error(`Lane failed: A=${laneAResult.status}, B=${laneBResult.status}`);
      }
    } catch (error) {
      echoError(`❌ Parallel execution failed: ${error.message}`);
      throw error;
    }

    echoSuccess("\n🎉 Session Debug Test completed successfully!");
  } catch (error) {
    echoError(`\n💥 Session Debug Test failed: ${error.message}`);
    process.exit(1);
  }
}

// Run the test
testSessionDebug();
