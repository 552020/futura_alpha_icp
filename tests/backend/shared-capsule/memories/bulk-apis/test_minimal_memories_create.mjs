#!/usr/bin/env node

/**
 * Minimal Memories Create Test (JavaScript)
 *
 * This test tries memories_create with minimal arguments to isolate the issue.
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { loadDfxIdentity } from "../../upload/ic-identity.js";
import fetch from "node-fetch";

// Import the backend interface
import { idlFactory } from "../../../../../.dfx/local/canisters/backend/service.did.js";

// Test configuration
const TEST_NAME = "Minimal Memories Create Test";
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

// Test 1: Create a minimal memory
async function testMinimalMemoriesCreate() {
  echoInfo("Testing memories_create with minimal arguments...");

  try {
    // Get capsule ID
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

    // Try with minimal arguments
    const memoryResult = await backend.memories_create(
      capsuleId, // text - capsule ID
      [], // opt blob - no inline data
      [], // opt BlobRef - no blob reference
      [], // opt StorageEdgeBlobType - no storage type
      [], // opt text - no storage key
      [], // opt text - no bucket
      [], // opt nat64 - no file_created_at
      [], // opt blob - no sha256
      {
        Note: {
          base: {
            url: [],
            height: [],
            updated_at: BigInt(Date.now() * 1000000),
            asset_type: { Original: null },
            sha256: [],
            name: "minimal_test",
            storage_key: [],
            tags: ["test"],
            processing_error: [],
            mime_type: "text/plain",
            description: ["Minimal test memory"],
            created_at: BigInt(Date.now() * 1000000),
            deleted_at: [],
            bytes: BigInt(0),
            asset_location: [],
            width: [],
            processing_status: [],
            bucket: [],
          },
          language: [],
          word_count: [],
          format: [],
        },
      },
      "minimal-test-key"
    );

    // Handle BigInt serialization for logging
    const logResult = JSON.stringify(
      memoryResult,
      (key, value) => (typeof value === "bigint" ? value.toString() : value),
      2
    );
    echoInfo(`Response: ${logResult}`);

    if ("Ok" in memoryResult && memoryResult.Ok) {
      echoPass("memories_create call successful with minimal arguments");
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
    echoError("Usage: node test_minimal_memories_create.mjs <BACKEND_CANISTER_ID>");
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

  const tests = [{ name: "memories_create with minimal arguments", fn: testMinimalMemoriesCreate }];

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


