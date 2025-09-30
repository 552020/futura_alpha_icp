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
const TEST_NAME = "Upload Begin Tests";
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
        tags: ["test", "upload-begin"],
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
async function testUploadBeginSuccess(backend, capsuleId) {
  console.log("🧪 Testing: Upload begin (success)");

  try {
    const sessionId = await beginUploadSession(backend, capsuleId, 4, "idem-1");
    console.log(`✅ Upload session created successfully: ${sessionId}`);
    return { success: true, sessionId };
  } catch (error) {
    console.error(`❌ Upload session creation failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

async function testUploadBeginIdempotency(backend, capsuleId) {
  console.log("🧪 Testing: Upload begin (idempotency)");

  try {
    const sessionId1 = await beginUploadSession(backend, capsuleId, 4, "idem-1");
    const sessionId2 = await beginUploadSession(backend, capsuleId, 4, "idem-1");

    if (sessionId1 === sessionId2 && sessionId1) {
      console.log(`✅ Upload begin idempotency working correctly: same session ID returned (${sessionId1})`);
      return { success: true, sessionId1, sessionId2 };
    } else {
      console.error(`❌ Upload begin idempotency test failed. Session1: ${sessionId1}, Session2: ${sessionId2}`);
      return { success: false, sessionId1, sessionId2 };
    }
  } catch (error) {
    console.error(`❌ Upload begin idempotency test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

async function testUploadBeginZeroChunks(backend, capsuleId) {
  console.log("🧪 Testing: Upload begin (zero chunks validation)");

  try {
    const assetMetadata = createDocumentAssetMetadata("test", "test");
    const begin = await backend.uploads_begin(capsuleId, assetMetadata, 0, "idem-zero");

    if ("Err" in begin) {
      console.log(`✅ Upload begin correctly rejected zero chunks: ${JSON.stringify(begin.Err)}`);
      return { success: true, result: begin };
    } else {
      console.error(`❌ Upload begin should have rejected zero chunks: ${JSON.stringify(begin)}`);
      return { success: false, result: begin };
    }
  } catch (error) {
    console.error(`❌ Upload begin zero chunks test failed: ${error.message}`);
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
    await runTest("Upload begin (success)", testUploadBeginSuccess, backend, capsuleId);
    await runTest("Upload begin (idempotency)", testUploadBeginIdempotency, backend, capsuleId);
    await runTest("Upload begin (zero chunks validation)", testUploadBeginZeroChunks, backend, capsuleId);

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
