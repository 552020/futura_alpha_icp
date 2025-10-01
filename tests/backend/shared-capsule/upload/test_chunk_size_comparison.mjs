#!/usr/bin/env node

/**
 * Chunk Size Comparison Test
 *
 * This test compares different chunk sizes to help determine the optimal
 * configuration for ICP uploads. It tests various chunk sizes with the
 * same file to measure performance and resource usage.
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity } from "./ic-identity.js";
import fs from "node:fs";
import path from "node:path";

// Import the backend interface
import { idlFactory } from "../../../../src/nextjs/src/ic/declarations/backend/backend.did.js";

// Test configuration
const TEST_NAME = "Chunk Size Comparison Test";
const TEST_FILE_PATH = "./assets/input/avocado_medium.jpg";

// Chunk sizes to test (based on ICP expert recommendations)
const CHUNK_SIZES = [
  { size: 64 * 1024, name: "64KB" }, // Current (suboptimal)
  { size: 256 * 1024, name: "256KB" }, // Better
  { size: 1024 * 1024, name: "1MB" }, // Good
  { size: 1_800_000, name: "1.8MB" }, // Expert recommended (exact bytes)
  { size: 2_097_152, name: "2MB" }, // At ICP limit (exact bytes)
];

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

function echoError(message) {
  console.error(`💥 ${message}`);
}

function echoWarning(message) {
  console.log(`⚠️  ${message}`);
}

// Test function for a specific chunk size
async function testChunkSize(chunkSizeConfig) {
  const { size: chunkSize, name: chunkName } = chunkSizeConfig;
  const fileBuffer = fs.readFileSync(TEST_FILE_PATH);
  const fileName = `test_${chunkName.toLowerCase().replace("kb", "k").replace("mb", "m")}.jpg`;

  echoInfo(`\n🧪 Testing ${chunkName} chunks (${chunkSize} bytes)`);
  echoInfo(`File: ${fileName} (${fileBuffer.length} bytes)`);

  const chunkCount = Math.ceil(fileBuffer.length / chunkSize);
  echoInfo(`Chunks: ${chunkCount} chunks needed`);

  // Calculate efficiency metrics
  const totalRequests = chunkCount + 2; // +2 for begin/finish
  const efficiency = Math.round((1 - totalRequests / 58) * 100); // 58 = current total requests

  echoInfo(`Total requests: ${totalRequests} (${efficiency}% reduction vs 64KB)`);

  try {
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
          tags: ["test", "chunk-size-comparison"],
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

    const idempotencyKey = `test-${chunkName}-${Date.now()}`;

    // Begin upload session
    echoInfo(`Starting upload session...`);
    const startTime = Date.now();

    const beginResult = await backend.uploads_begin(capsuleId, assetMetadata, chunkCount, idempotencyKey);

    if ("Err" in beginResult) {
      const error = JSON.stringify(beginResult.Err);
      echoWarning(`${chunkName} chunks: Upload begin failed - ${error}`);
      return { success: false, error, chunkSize: chunkName, requests: totalRequests };
    }

    const sessionId = beginResult.Ok;
    echoInfo(`Upload session started: ${sessionId}`);

    // Upload file in chunks
    echoInfo(`Uploading ${chunkCount} chunks...`);
    for (let i = 0; i < chunkCount; i++) {
      const offset = i * chunkSize;
      const currentChunkSize = Math.min(chunkSize, fileBuffer.length - offset);
      const chunk = fileBuffer.slice(offset, offset + currentChunkSize);

      const putChunkResult = await backend.uploads_put_chunk(sessionId, i, Array.from(chunk));
      if ("Err" in putChunkResult) {
        const error = JSON.stringify(putChunkResult.Err);
        echoWarning(`${chunkName} chunks: Put chunk ${i} failed - ${error}`);
        return { success: false, error, chunkSize: chunkName, requests: totalRequests };
      }

      // Progress indicator
      if ((i + 1) % 5 === 0 || i === chunkCount - 1) {
        const progress = Math.round(((i + 1) / chunkCount) * 100);
        echoInfo(`Progress: ${progress}% (${i + 1}/${chunkCount} chunks)`);
      }
    }

    // Calculate hash and total length for finish
    const crypto = await import("crypto");
    const hash = crypto.createHash("sha256").update(fileBuffer).digest();
    const totalLen = BigInt(fileBuffer.length);

    // Finish upload
    const finishResult = await backend.uploads_finish(sessionId, Array.from(hash), totalLen);
    if ("Err" in finishResult) {
      const error = JSON.stringify(finishResult.Err);
      echoWarning(`${chunkName} chunks: Upload finish failed - ${error}`);
      return { success: false, error, chunkSize: chunkName, requests: totalRequests };
    }

    const endTime = Date.now();
    const duration = endTime - startTime;
    const blobId = finishResult.Ok;

    echoPass(`${chunkName} chunks: Upload successful! (${duration}ms)`);
    echoInfo(`Blob ID: ${blobId}`);

    return {
      success: true,
      chunkSize: chunkName,
      requests: totalRequests,
      duration,
      blobId,
      efficiency,
    };
  } catch (error) {
    echoError(`${chunkName} chunks: Test failed - ${error.message}`);
    return { success: false, error: error.message, chunkSize: chunkName, requests: totalRequests };
  }
}

// Main test execution
async function main() {
  echoInfo(`Starting ${TEST_NAME}`);

  // Get backend canister ID
  const backendCanisterId = process.argv[2];
  if (!backendCanisterId) {
    echoError("Usage: node test_chunk_size_comparison.mjs <BACKEND_CANISTER_ID>");
    process.exit(1);
  }

  // Check if test file exists
  if (!fs.existsSync(TEST_FILE_PATH)) {
    echoError(`Test file not found: ${TEST_FILE_PATH}`);
    process.exit(1);
  }

  const fileStats = fs.statSync(TEST_FILE_PATH);
  echoInfo(`Test file: ${TEST_FILE_PATH} (${fileStats.size} bytes)`);

  // Setup agent and backend
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

  // Run tests for each chunk size
  const results = [];

  for (const chunkSizeConfig of CHUNK_SIZES) {
    const result = await testChunkSize(chunkSizeConfig);
    results.push(result);

    // Small delay between tests
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }

  // Summary
  echoInfo(`\n📊 Chunk Size Comparison Results:`);
  echoInfo(`=====================================`);

  results.forEach((result) => {
    if (result.success) {
      echoPass(
        `${result.chunkSize}: ✅ Success (${result.requests} requests, ${result.duration}ms, ${result.efficiency}% efficiency)`
      );
    } else {
      echoFail(`${result.chunkSize}: ❌ Failed (${result.requests} requests, ${result.error})`);
    }
  });

  // Recommendations
  const successfulResults = results.filter((r) => r.success);
  if (successfulResults.length > 0) {
    const bestResult = successfulResults.reduce((best, current) =>
      current.efficiency > best.efficiency ? current : best
    );

    echoInfo(`\n🎯 Recommendation:`);
    echoInfo(`Best performing chunk size: ${bestResult.chunkSize}`);
    echoInfo(`Efficiency improvement: ${bestResult.efficiency}%`);
    echoInfo(`Total requests: ${bestResult.requests} (vs 58 for 64KB)`);
  } else {
    echoWarning(`\n⚠️  No successful uploads - all chunk sizes failed`);
    echoInfo(`This suggests a resource allocation issue rather than chunk size problem`);
  }

  process.exit(0);
}

// Run the test
main().catch((error) => {
  echoError(`Test execution failed: ${error.message}`);
  process.exit(1);
});
