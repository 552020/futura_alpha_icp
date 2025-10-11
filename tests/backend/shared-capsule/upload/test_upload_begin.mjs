#!/usr/bin/env node

import {
  createTestActor,
  parseTestArgs,
  createTestActorOptions,
  logNetworkConfig,
  getOrCreateTestCapsuleForUpload,
  createTestRunner,
} from "../../utils/index.js";

// Parse command line arguments using shared utility
const parsedArgs = parseTestArgs("test_upload_begin.mjs", "Tests the upload_begin API endpoint for chunked uploads");

// Import the backend interface
import { idlFactory } from "../../declarations/backend/backend.did.js";

// Test configuration
const TEST_NAME = "Upload Begin Tests";

// Helper function to begin an upload session
async function beginUploadSession(backend, capsuleId, chunkCount, idempotencyKey) {
  console.log(`🚀 Starting upload session with ${chunkCount} chunks...`);
  const begin = await backend.uploads_begin(capsuleId, chunkCount, idempotencyKey);

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
    const begin = await backend.uploads_begin(capsuleId, 0, "idem-zero");

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

    // Run all tests in order
    await runner.runTest("Upload begin (success)", testUploadBeginSuccess, backend, capsuleId);
    await runner.runTest("Upload begin (idempotency)", testUploadBeginIdempotency, backend, capsuleId);
    await runner.runTest("Upload begin (zero chunks validation)", testUploadBeginZeroChunks, backend, capsuleId);

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
