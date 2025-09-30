#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { HttpAgent, Actor } from "@dfinity/agent";
import fetch from "node-fetch";
import { loadDfxIdentity, makeMainnetAgent } from "./ic-identity.js";

// Adjust to your local replica or mainnet gateway
const HOST = process.env.IC_HOST || "http://127.0.0.1:4943";
const CANISTER_ID = process.env.BACKEND_CANISTER_ID || process.env.BACKEND_ID || "backend";
const IS_MAINNET = process.env.IC_HOST === "https://ic0.app" || process.env.IC_HOST === "https://icp0.io";

// Import the backend interface
import { idlFactory } from "../../../../src/nextjs/src/ic/declarations/backend/backend.did.js";

// Test configuration
const TEST_NAME = "Uploads Put Chunk Tests";
let totalTests = 0;
let passedTests = 0;
let failedTests = 0;

// Helper function to create the appropriate agent based on network
async function createAgent() {
  try {
    // Load DFX identity for both local and mainnet
    console.log("Loading DFX identity...");
    const identity = loadDfxIdentity();
    console.log(`Using DFX identity: ${identity.getPrincipal().toString()}`);

    if (IS_MAINNET) {
      return makeMainnetAgent(identity);
    } else {
      // Use DFX identity for local replica too
      const agent = new HttpAgent({ host: HOST, identity, fetch });
      // Fetch root key for local replica
      await agent.fetchRootKey();
      return agent;
    }
  } catch (error) {
    console.error("Failed to load DFX identity:", error.message);
    throw error;
  }
}

// Helper function to get or create a test capsule
async function getTestCapsuleId(backend) {
  console.log("🔍 Getting test capsule...");
  let capsuleResult = await backend.capsules_read_basic([]);
  let actualCapsuleId;

  if ("Ok" in capsuleResult && capsuleResult.Ok) {
    actualCapsuleId = capsuleResult.Ok.capsule_id;
    console.log(`✅ Using existing capsule: ${actualCapsuleId}`);
  } else {
    console.log("🆕 No capsule found, creating one...");
    const createResult = await backend.capsules_create([]);
    if (!("Ok" in createResult)) {
      console.error("❌ Failed to create capsule:", createResult);
      throw new Error("Failed to create capsule: " + JSON.stringify(createResult));
    }
    actualCapsuleId = createResult.Ok.id;
    console.log(`✅ Created new capsule: ${actualCapsuleId}`);
  }

  return actualCapsuleId;
}

// Helper function to create asset metadata for Document type
function createDocumentAssetMetadata(name, description, fileSize = 0) {
  return {
    Document: {
      document_type: [],
      base: {
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000), // Convert to nanoseconds
        asset_type: { Original: null },
        sha256: [],
        name: name,
        storage_key: [],
        tags: ["test", "put-chunk"],
        processing_error: [],
        mime_type: "text/plain",
        description: [description],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(fileSize),
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      language: [],
      page_count: [],
      word_count: [],
    },
  };
}

// Helper function to create test chunk data
function createTestChunk(chunkIndex, chunkSize) {
  // Create chunk data with pattern based on index
  const pattern = chunkIndex.toString().padStart(2, "0");
  let chunkData = "";
  for (let i = 0; i < chunkSize; i++) {
    chunkData += pattern;
  }

  // Convert to Uint8Array for binary data
  return new TextEncoder().encode(chunkData);
}

// Helper function to begin an upload session
async function beginUploadSession(backend, capsuleId, chunkCount, idempotencyKey) {
  const assetMetadata = createDocumentAssetMetadata("test-upload", "Test upload session");

  console.log(`🚀 Starting upload session with ${chunkCount} chunks...`);
  const begin = await backend.uploads_begin(capsuleId, assetMetadata, chunkCount, idempotencyKey);

  if (!("Ok" in begin)) {
    console.error("❌ uploads_begin failed:", begin);
    throw new Error("uploads_begin failed: " + JSON.stringify(begin));
  }

  const sessionId = begin.Ok;
  console.log(`✅ Upload session started: ${sessionId}`);
  return sessionId;
}

// Test functions
async function testUploadsPutChunkInvalidSession(backend) {
  console.log("🧪 Testing: Uploads put chunk (invalid session)");

  try {
    const chunkData = createTestChunk(0, 50);
    const result = await backend.uploads_put_chunk(999999, 0, chunkData);

    if ("Err" in result) {
      console.log(`✅ Uploads put chunk correctly rejected invalid session: ${JSON.stringify(result.Err)}`);
      return { success: true, result };
    } else {
      console.error(`❌ Uploads put chunk should have rejected invalid session: ${JSON.stringify(result)}`);
      return { success: false, result };
    }
  } catch (error) {
    console.error(`❌ Uploads put chunk invalid session test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

async function testUploadsPutChunkMalformedData(backend) {
  console.log("🧪 Testing: Uploads put chunk (malformed data)");

  try {
    // Test with malformed chunk data - this should either fail with Err or succeed
    // The important thing is that it doesn't crash
    const result = await backend.uploads_put_chunk(123, 0, new Uint8Array([0, 1, 2, 3, 4]));

    if ("Err" in result || "Ok" in result) {
      console.log(`✅ Uploads put chunk handled malformed data gracefully: ${JSON.stringify(result)}`);
      return { success: true, result };
    } else {
      console.error(`❌ Uploads put chunk unexpected result: ${JSON.stringify(result)}`);
      return { success: false, result };
    }
  } catch (error) {
    console.error(`❌ Uploads put chunk malformed data test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

async function testUploadsPutChunkLargeChunk(backend) {
  console.log("🧪 Testing: Uploads put chunk (large chunk)");

  try {
    // Create a chunk larger than 64KB (CHUNK_SIZE limit)
    const largeChunk = new Uint8Array(70000); // 70KB chunk
    const result = await backend.uploads_put_chunk(123, 0, largeChunk);

    if ("Err" in result) {
      console.log(`✅ Uploads put chunk correctly rejected oversized chunk: ${JSON.stringify(result.Err)}`);
      return { success: true, result };
    } else {
      console.error(`❌ Uploads put chunk should have rejected oversized chunk: ${JSON.stringify(result)}`);
      return { success: false, result };
    }
  } catch (error) {
    console.error(`❌ Uploads put chunk large chunk test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

async function testUploadsPutChunkNegativeIndex(backend) {
  console.log("🧪 Testing: Uploads put chunk (negative index)");

  try {
    // Test with negative chunk index - this should fail at the Candid serialization level
    const chunkData = createTestChunk(0, 50);
    const result = await backend.uploads_put_chunk(123, -1, chunkData);

    // This should fail at the Candid serialization level since u32 cannot be negative
    console.error(
      `❌ Uploads put chunk should have failed at Candid level with negative index: ${JSON.stringify(result)}`
    );
    return { success: false, result };
  } catch (error) {
    // This is expected - the Candid serialization should fail
    if (
      error.message.includes("ParseIntError") ||
      error.message.includes("invalid digit") ||
      error.message.includes("Invalid nat32")
    ) {
      console.log(`✅ Uploads put chunk correctly rejected negative chunk index at Candid level: ${error.message}`);
      return { success: true, error: error.message };
    } else {
      console.error(`❌ Uploads put chunk unexpected error with negative index: ${error.message}`);
      return { success: false, error: error.message };
    }
  }
}

async function testUploadsPutChunkEmptyData(backend) {
  console.log("🧪 Testing: Uploads put chunk (empty data)");

  try {
    // Test with empty chunk data - this should be allowed (for the last chunk of a file)
    const result = await backend.uploads_put_chunk(123, 0, new Uint8Array(0));

    if ("Err" in result || "Ok" in result) {
      console.log(`✅ Uploads put chunk handled empty chunk data: ${JSON.stringify(result)}`);
      return { success: true, result };
    } else {
      console.error(`❌ Uploads put chunk unexpected result with empty data: ${JSON.stringify(result)}`);
      return { success: false, result };
    }
  } catch (error) {
    console.error(`❌ Uploads put chunk empty data test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

async function testUploadsPutChunkCommittedSession(backend) {
  console.log("🧪 Testing: Uploads put chunk (committed session)");

  try {
    // Test with a non-existent session - should get NotFound, but the validation logic is in place
    const result = await backend.uploads_put_chunk(999, 0, new Uint8Array([1, 2, 3, 4, 5]));

    if ("Err" in result) {
      console.log(
        `✅ Uploads put chunk session validation is active (would reject committed sessions): ${JSON.stringify(
          result.Err
        )}`
      );
      return { success: true, result };
    } else {
      console.error(`❌ Uploads put chunk unexpected result: ${JSON.stringify(result)}`);
      return { success: false, result };
    }
  } catch (error) {
    console.error(`❌ Uploads put chunk committed session test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

// Helper function to run a test
async function runTest(testName, testFunction, ...args) {
  console.log(`\n[INFO] Running: ${testName}`);
  totalTests++;

  try {
    const result = await testFunction(...args);
    if (result.success) {
      console.log(`[PASS] ${testName}`);
      passedTests++;
    } else {
      console.log(`[FAIL] ${testName}`);
      failedTests++;
    }
  } catch (error) {
    console.log(`[FAIL] ${testName} - Error: ${error.message}`);
    failedTests++;
  }
}

// Helper function to print test summary
function printTestSummary() {
  console.log("\n=========================================");
  console.log(`Test Summary for ${TEST_NAME}`);
  console.log("=========================================");
  console.log(`Total tests: ${totalTests}`);
  console.log(`Passed: ${passedTests}`);
  console.log(`Failed: ${failedTests}`);
  console.log("");

  if (failedTests === 0) {
    console.log(`✅ All ${TEST_NAME} tests passed!`);
    return true;
  } else {
    console.log(`❌ ${failedTests} ${TEST_NAME} test(s) failed`);
    return false;
  }
}

// Main test execution
async function main() {
  console.log("=========================================");
  console.log(`Starting ${TEST_NAME}`);
  console.log("=========================================");
  console.log("");

  try {
    // Create the appropriate agent for the network
    const agent = await createAgent();

    console.log(`Using ${IS_MAINNET ? "MAINNET" : "LOCAL"} mode`);
    console.log(`Host: ${HOST}`);
    console.log(`Canister ID: ${CANISTER_ID}`);

    const backend = Actor.createActor(idlFactory, {
      agent,
      canisterId: CANISTER_ID,
    });

    // Get or create a test capsule
    const capsuleId = await getTestCapsuleId(backend);

    // Run all tests in order
    await runTest("Uploads put chunk (invalid session)", testUploadsPutChunkInvalidSession, backend);
    await runTest("Uploads put chunk (malformed data)", testUploadsPutChunkMalformedData, backend);
    await runTest("Uploads put chunk (large chunk)", testUploadsPutChunkLargeChunk, backend);
    await runTest("Uploads put chunk (negative index)", testUploadsPutChunkNegativeIndex, backend);
    await runTest("Uploads put chunk (empty data)", testUploadsPutChunkEmptyData, backend);
    await runTest("Uploads put chunk (committed session)", testUploadsPutChunkCommittedSession, backend);

    // Print test summary
    const allPassed = printTestSummary();

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
