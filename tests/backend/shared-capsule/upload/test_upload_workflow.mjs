#!/usr/bin/env node

/**
 * Upload Workflow Tests
 * Tests end-to-end upload workflows and edge cases
 *
 * Migrated from test_upload_workflow.sh to JavaScript for better Candid handling
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity, makeMainnetAgent } from "./ic-identity.js";
import crypto from "node:crypto";

// Import the backend interface
import { idlFactory } from "../../../../src/nextjs/src/ic/declarations/backend/backend.did.js";

// Test configuration
const TEST_NAME = "Upload Workflow Tests";
let totalTests = 0;
let passedTests = 0;
let failedTests = 0;

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

// Test runner
async function runTest(testName, testFunction) {
  echoInfo(`Running: ${testName}`);
  totalTests++;

  try {
    const result = await testFunction();
    if (result) {
      echoPass(testName);
      passedTests++;
    } else {
      echoFail(testName);
      failedTests++;
    }
  } catch (error) {
    echoError(`${testName} - Error: ${error.message}`);
    echoFail(testName);
    failedTests++;
  }
  console.log("");
}

// Helper function to clean up upload sessions
async function cleanupUploadSessions(backend) {
  echoInfo("Cleaning up upload sessions...");
  for (let i = 1; i <= 20; i++) {
    try {
      await backend.uploads_abort(i);
    } catch (error) {
      // Ignore errors during cleanup
    }
  }
  // Give a moment for cleanup to complete
  await new Promise((resolve) => setTimeout(resolve, 1000));
}

// Helper function to get test capsule ID
async function getTestCapsuleId(backend) {
  try {
    const capsules = await backend.capsules_list();
    if (capsules && capsules.length > 0) {
      return capsules[0].id;
    }

    // Create a new capsule if none exists
    const result = await backend.capsules_create(null);
    if ("Ok" in result) {
      return result.Ok.id;
    } else {
      throw new Error(`Failed to create capsule: ${JSON.stringify(result)}`);
    }
  } catch (error) {
    throw new Error(`Failed to get test capsule: ${error.message}`);
  }
}

// Helper function to begin upload session
async function beginUploadSession(backend, capsuleId, chunkCount, idem) {
  const assetMetadata = {
    Document: {
      base: {
        name: "test-document",
        description: ["Test document for upload workflow"],
        tags: [],
        asset_type: { Original: null },
        bytes: 0,
        mime_type: "text/plain",
        sha256: [],
        width: [],
        height: [],
        url: [],
        storage_key: [],
        bucket: [],
        asset_location: [],
        processing_status: [],
        processing_error: [],
        created_at: 0,
        updated_at: 0,
        deleted_at: [],
      },
      page_count: [],
      document_type: [],
      language: [],
      word_count: [],
    },
  };

  const result = await backend.uploads_begin(capsuleId, assetMetadata, chunkCount, idem);
  if ("Ok" in result) {
    return result.Ok;
  } else {
    throw new Error(`Failed to begin upload session: ${JSON.stringify(result)}`);
  }
}

// Helper function to create test chunk data
function createTestChunk(index, size) {
  const data = new Uint8Array(size);
  for (let i = 0; i < size; i++) {
    data[i] = (index + i) % 256;
  }
  return data;
}

// Helper function to upload chunk
async function uploadChunk(backend, sessionId, chunkIndex, chunkData) {
  const result = await backend.uploads_put_chunk(sessionId, chunkIndex, chunkData);
  return "Ok" in result;
}

// Helper function to compute test hash (matches backend logic)
function computeTestHash(chunkData, chunkCount) {
  // Compute hash of all chunks concatenated together
  // This matches what the backend computes in store_from_chunks
  const allChunks = new Uint8Array(chunkCount * chunkData.length);
  for (let i = 0; i < chunkCount; i++) {
    const chunk = createTestChunk(i, chunkData.length);
    allChunks.set(chunk, i * chunkData.length);
  }

  // Compute SHA256 hash
  const hash = crypto.createHash("sha256").update(allChunks).digest();
  return new Uint8Array(hash);
}

// Helper function to finish upload
async function finishUpload(backend, sessionId, hash, totalLen) {
  const result = await backend.uploads_finish(sessionId, hash, totalLen);
  return "Ok" in result;
}

// Test functions

async function testUploadsBeginTooManyChunks(backend) {
  // Test with too many chunks (exceeds MAX_CHUNKS)
  try {
    const capsuleId = await getTestCapsuleId(backend);
    const assetMetadata = {
      Document: {
        base: {
          name: "test",
          description: ["test"],
          tags: [],
          asset_type: { Original: null },
          bytes: 0,
          mime_type: "text/plain",
          sha256: [],
          width: [],
          height: [],
          url: [],
          storage_key: [],
          bucket: [],
          asset_location: [],
          processing_status: [],
          processing_error: [],
          created_at: 0,
          updated_at: 0,
          deleted_at: [],
        },
        page_count: [],
        document_type: [],
        language: [],
        word_count: [],
      },
    };

    const result = await backend.uploads_begin(capsuleId, assetMetadata, 20000, "test-idem");

    if ("Err" in result) {
      echoInfo(`Upload session validation correctly rejected too many chunks: ${JSON.stringify(result)}`);
      return true;
    } else {
      echoInfo(`Upload session validation test result: ${JSON.stringify(result)}`);
      return false;
    }
  } catch (error) {
    echoInfo(`Upload session validation correctly rejected too many chunks (error): ${error.message}`);
    return true;
  }
}

async function testCompleteUploadWorkflow(backend) {
  // Test a complete upload workflow from begin to finish
  const capsuleId = await getTestCapsuleId(backend);
  const chunkCount = 3;
  const chunkSize = 50;
  const idem = `workflow-test-${Date.now()}`;

  echoInfo(`Testing complete upload workflow with ${chunkCount} chunks`);

  // Begin upload
  const sessionId = await beginUploadSession(backend, capsuleId, chunkCount, idem);
  if (!sessionId) {
    echoInfo("Failed to create upload session");
    return false;
  }

  // Upload all chunks using regular endpoint
  for (let i = 0; i < chunkCount; i++) {
    const chunkData = createTestChunk(i, chunkSize);
    if (!(await uploadChunk(backend, sessionId, i, chunkData))) {
      echoInfo(`Failed to upload chunk ${i}`);
      return false;
    }
  }

  // Finish upload with correct hash using regular endpoint
  const chunkData = createTestChunk(0, chunkSize);
  const expectedHash = computeTestHash(chunkData, chunkCount);
  const totalLen = chunkCount * chunkSize; // Use actual chunk size

  echoInfo(
    `Finishing upload with hash: ${Array.from(expectedHash)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("")}`
  );
  echoInfo(`Total length: ${totalLen}`);

  const finishResult = await backend.uploads_finish(sessionId, expectedHash, totalLen);
  echoInfo(`Finish result: ${JSON.stringify(finishResult)}`);

  if ("Ok" in finishResult) {
    echoInfo("Complete upload workflow successful");
    return true;
  } else {
    echoInfo(`Upload workflow failed at finish: ${JSON.stringify(finishResult)}`);
    return false;
  }
}

async function testUploadWorkflowMissingChunks(backend) {
  // Test workflow with missing chunks
  const capsuleId = await getTestCapsuleId(backend);
  const chunkCount = 3;
  const idem = `missing-chunks-test-${Date.now()}`;

  echoInfo("Testing upload workflow with missing chunks");

  // Begin upload
  const sessionId = await beginUploadSession(backend, capsuleId, chunkCount, idem);
  if (!sessionId) {
    echoInfo("Failed to create upload session");
    return false;
  }

  // Upload only first chunk (leaving 2 missing)
  const chunkData = createTestChunk(0, 50);
  await uploadChunk(backend, sessionId, 0, chunkData);

  // Try to finish with missing chunks
  const expectedHash = computeTestHash(chunkData, 1);
  if (await finishUpload(backend, sessionId, expectedHash, 50)) {
    echoInfo("Should have failed with missing chunks");
    return false;
  } else {
    echoInfo("Correctly rejected incomplete upload");
    return true;
  }
}

async function testUploadWorkflowAbort(backend) {
  // Test upload abort workflow
  const capsuleId = await getTestCapsuleId(backend);
  const chunkCount = 2;
  const idem = `abort-test-${Date.now()}`;

  echoInfo("Testing upload abort workflow");

  // Begin upload
  const sessionId = await beginUploadSession(backend, capsuleId, chunkCount, idem);
  if (!sessionId) {
    echoInfo("Failed to create upload session");
    return false;
  }

  // Upload one chunk
  const chunkData = createTestChunk(0, 50);
  await uploadChunk(backend, sessionId, 0, chunkData);

  // Abort the upload
  const result = await backend.uploads_abort(sessionId);

  if ("Ok" in result) {
    echoInfo(`Upload abort successful: ${JSON.stringify(result)}`);
    return true;
  } else {
    echoInfo(`Upload abort failed: ${JSON.stringify(result)}`);
    return false;
  }
}

async function testUploadWorkflowIdempotency(backend) {
  // Test that begin_upload with same idem returns same session
  const capsuleId = await getTestCapsuleId(backend);
  const idem = `idempotency-test-${Date.now()}`;

  echoInfo("Testing upload begin idempotency");

  const assetMetadata = {
    Document: {
      base: {
        name: "idempotency-test",
        description: ["Idempotency test"],
        tags: [],
        asset_type: { Original: null },
        bytes: 0,
        mime_type: "text/plain",
        sha256: [],
        width: [],
        height: [],
        url: [],
        storage_key: [],
        bucket: [],
        asset_location: [],
        processing_status: [],
        processing_error: [],
        created_at: 0,
        updated_at: 0,
        deleted_at: [],
      },
      page_count: [],
      document_type: [],
      language: [],
      word_count: [],
    },
  };

  const result1 = await backend.uploads_begin(capsuleId, assetMetadata, 2, idem);
  const result2 = await backend.uploads_begin(capsuleId, assetMetadata, 2, idem);

  const sessionId1 = "Ok" in result1 ? result1.Ok : null;
  const sessionId2 = "Ok" in result2 ? result2.Ok : null;

  if (sessionId1 === sessionId2 && sessionId1 !== null) {
    echoInfo("Upload begin idempotency working correctly: same session ID returned");
    return true;
  } else {
    echoInfo(`Upload begin idempotency test failed. Session1: ${sessionId1}, Session2: ${sessionId2}`);
    echoInfo(`Result1: ${JSON.stringify(result1)}`);
    echoInfo(`Result2: ${JSON.stringify(result2)}`);
    return false;
  }
}

async function testLargeFileWorkflow(backend) {
  // Test with a larger number of chunks to simulate a bigger file
  const capsuleId = await getTestCapsuleId(backend);
  const chunkCount = 10;
  const chunkSize = 50;
  const idem = `large-file-test-${Date.now()}`;

  echoInfo(`Testing large file workflow with ${chunkCount} chunks`);

  // Begin upload
  const sessionId = await beginUploadSession(backend, capsuleId, chunkCount, idem);
  if (!sessionId) {
    echoInfo("Failed to create upload session");
    return false;
  }

  // Upload all chunks
  for (let i = 0; i < chunkCount; i++) {
    const chunkData = createTestChunk(i, chunkSize);
    if (!(await uploadChunk(backend, sessionId, i, chunkData))) {
      echoInfo(`Failed to upload chunk ${i} in large file test`);
      return false;
    }
  }

  // Finish the large file with correct hash
  const chunkData = createTestChunk(0, chunkSize);
  const expectedHash = computeTestHash(chunkData, chunkCount);
  const totalLen = chunkCount * chunkSize; // Use actual chunk size

  echoInfo(
    `Finishing large file with hash: ${Array.from(expectedHash)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("")}`
  );
  echoInfo(`Total length: ${totalLen}`);

  const finishResult = await backend.uploads_finish(sessionId, expectedHash, totalLen);
  echoInfo(`Finish result: ${JSON.stringify(finishResult)}`);

  if ("Ok" in finishResult) {
    echoInfo("Large file workflow completed successfully");
    return true;
  } else {
    echoInfo(`Large file workflow failed: ${JSON.stringify(finishResult)}`);
    return false;
  }
}

// Print test summary
function printTestSummary() {
  console.log("=========================================");
  console.log(`Test Summary for ${TEST_NAME}`);
  console.log("=========================================");
  console.log(`Total Tests: ${totalTests}`);
  console.log(`Passed: ${passedTests}`);
  console.log(`Failed: ${failedTests}`);
  console.log(`Success Rate: ${totalTests > 0 ? ((passedTests / totalTests) * 100).toFixed(1) : 0}%`);
  console.log("=========================================");

  if (failedTests === 0) {
    console.log("🎉 All tests passed!");
  } else {
    console.log(`❌ ${failedTests} test(s) failed`);
  }
}

// Main function
async function main() {
  console.log("=========================================");
  console.log(`Starting ${TEST_NAME}`);
  console.log("=========================================");
  console.log("");

  // Get backend canister ID from environment
  const backendCanisterId = process.env.BACKEND_ID;
  if (!backendCanisterId) {
    echoError("BACKEND_ID environment variable not set");
    echoInfo("Please set the backend canister ID before running tests");
    process.exit(1);
  }

  // Create agent and actor
  console.log("Loading DFX identity...");
  const identity = loadDfxIdentity();
  console.log(`Using DFX identity: ${identity.getPrincipal().toString()}`);

  const agent = new HttpAgent({
    host: "http://127.0.0.1:4943",
    identity,
  });

  // Fetch root key for local development
  await agent.fetchRootKey();

  const backend = Actor.createActor(idlFactory, {
    agent,
    canisterId: backendCanisterId,
  });

  echoInfo(`Using backend canister: ${backendCanisterId}`);

  // Clean up any existing upload sessions
  await cleanupUploadSessions(backend);

  // Run all tests in order
  await runTest("Uploads begin (too many chunks validation)", () => testUploadsBeginTooManyChunks(backend));
  await runTest("Complete upload workflow", () => testCompleteUploadWorkflow(backend));
  await runTest("Upload workflow (missing chunks)", () => testUploadWorkflowMissingChunks(backend));
  await runTest("Upload workflow (abort)", () => testUploadWorkflowAbort(backend));
  await runTest("Upload workflow (idempotency)", () => testUploadWorkflowIdempotency(backend));
  await runTest("Large file workflow", () => testLargeFileWorkflow(backend));

  // Print test summary
  printTestSummary();

  if (failedTests === 0) {
    process.exit(0);
  } else {
    process.exit(1);
  }
}

// Run main function
main().catch((error) => {
  echoError(`Test execution failed: ${error.message}`);
  process.exit(1);
});
