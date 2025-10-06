#!/usr/bin/env node

/**
 * Exact Working Pattern Test (JavaScript)
 *
 * This test exactly replicates the working pattern from test_upload_download_file.mjs
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { loadDfxIdentity } from "../../upload/ic-identity.js";
import fetch from "node-fetch";

// Import the backend interface
import { idlFactory } from "../../../../../.dfx/local/canisters/backend/service.did.js";

// Test configuration
const TEST_NAME = "Exact Working Pattern Test";
const HOST = process.env.IC_HOST || "http://127.0.0.1:4943";
const IS_MAINNET = process.env.IC_HOST === "https://ic0.app" || process.env.IC_HOST === "https://icp0.io";

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

// Exact copy of the working createDocumentAssetMetadata function
function createDocumentAssetMetadata(fileName, fileSize, mimeType = "application/octet-stream") {
  return {
    Document: {
      document_type: [],
      base: {
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000), // Convert to nanoseconds
        asset_type: { Original: null },
        sha256: [],
        name: fileName,
        storage_key: [],
        tags: ["upload-test", "file", `size-${fileSize}`],
        processing_error: [],
        mime_type: mimeType,
        description: [`Upload test file - ${fileSize} bytes`],
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

// Test 1: Create memory using exact working pattern
async function testExactWorkingPattern() {
  echoInfo("Testing memories_create with exact working pattern...");

  try {
    // Get capsule ID using exact working pattern
    let capsuleId;
    const capsuleResult = await backend.capsules_read_basic([]);
    if ("Ok" in capsuleResult && capsuleResult.Ok) {
      capsuleId = capsuleResult.Ok.capsule_id;
      echoInfo(`Using existing capsule: ${capsuleId}`);
    } else {
      echoInfo("No capsule found, creating one...");
      const createResult = await backend.capsules_create([]);
      if (!("Ok" in createResult)) {
        echoFail("Failed to create capsule for memory creation");
        return false;
      }
      capsuleId = createResult.Ok.id;
      echoInfo(`Created new capsule: ${capsuleId}`);
    }

    // Create memory using exact working pattern
    const testData = new Uint8Array([84, 101, 115, 116, 32, 77, 101, 109, 111, 114, 121]); // "Test Memory"
    const assetMetadata = createDocumentAssetMetadata("test_memory.txt", testData.length);
    const idempotencyKey = `test_inline_${Date.now()}`;

    // Compute SHA256 hash
    const crypto = await import("crypto");
    const fileHash = crypto.createHash("sha256").update(testData).digest("hex");
    const hashBuffer = Buffer.from(fileHash, "hex");

    const result = await backend.memories_create(
      capsuleId,
      [new Uint8Array(testData)], // opt blob - inline data
      [], // opt BlobRef - no blob reference for inline
      [], // opt StorageEdgeBlobType - no storage edge for inline
      [], // opt text - no storage key for inline
      [], // opt text - no bucket for inline
      [], // opt nat64 - no file_created_at
      [new Uint8Array(hashBuffer)], // opt blob - sha256 hash
      assetMetadata,
      idempotencyKey
    );

    // Handle BigInt serialization for logging
    const logResult = JSON.stringify(result, (key, value) => (typeof value === "bigint" ? value.toString() : value), 2);
    echoInfo(`Response: ${logResult}`);

    if ("Ok" in result && result.Ok) {
      echoPass("memories_create call successful with exact working pattern");
      return true;
    } else {
      echoFail("memories_create should return success");
      return false;
    }
  } catch (error) {
    echoFail(`memories_create call failed: ${error.message}`);
    return false;
  }
}

// Main test execution
async function main() {
  echoInfo(`Starting ${TEST_NAME}`);
  echoInfo("==================================");

  // Get backend canister ID
  const backendCanisterId = process.argv[2];
  if (!backendCanisterId) {
    echoError("Usage: node test_exact_working_pattern.mjs <BACKEND_CANISTER_ID>");
    process.exit(1);
  }

  echoInfo(`Using canister ID: ${backendCanisterId}`);
  echoInfo(`Using host: ${HOST}`);
  echoInfo(`Mainnet mode: ${IS_MAINNET}`);

  // Setup agent and backend
  try {
    echoInfo("Loading DFX identity...");
    const identity = loadDfxIdentity();
    echoInfo(`Using identity: ${identity.getPrincipal().toString()}`);

    const agent = new HttpAgent({
      host: HOST,
      identity,
      fetch,
    });

    if (!IS_MAINNET) {
      await agent.fetchRootKey();
      echoInfo("Fetched root key for local replica");
    }

    backend = Actor.createActor(idlFactory, {
      agent,
      canisterId: backendCanisterId,
    });

    echoInfo("Backend actor created successfully");
  } catch (error) {
    echoError(`Failed to setup backend: ${error.message}`);
    process.exit(1);
  }

  // Run tests
  let totalTests = 0;
  let passedTests = 0;
  let failedTests = 0;

  const tests = [{ name: "memories_create with exact working pattern", fn: testExactWorkingPattern }];

  for (const test of tests) {
    totalTests++;
    echoInfo(`Running: ${test.name}`);

    try {
      const result = await test.fn();
      if (result) {
        passedTests++;
        echoPass(test.name);
      } else {
        failedTests++;
        echoFail(test.name);
      }
    } catch (error) {
      failedTests++;
      echoFail(`${test.name} - Error: ${error.message}`);
    }

    echoInfo(""); // Empty line for readability
  }

  // Test summary
  echoInfo("==================================");
  echoInfo("Test Summary:");
  echoInfo(`Total tests: ${totalTests}`);
  echoInfo(`Passed: ${passedTests}`);
  echoInfo(`Failed: ${failedTests}`);

  if (failedTests === 0) {
    echoPass("All tests passed!");
    process.exit(0);
  } else {
    echoFail(`${failedTests} test(s) failed!`);
    process.exit(1);
  }
}

// Run the test
main().catch((error) => {
  echoError(`Test execution failed: ${error.message}`);
  process.exit(1);
});


