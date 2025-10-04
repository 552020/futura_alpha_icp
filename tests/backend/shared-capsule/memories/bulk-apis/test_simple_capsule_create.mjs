#!/usr/bin/env node

/**
 * Simple Capsule Create Test (JavaScript)
 *
 * This test replicates the working pattern from test_capsules_create_mjs.mjs
 * to understand why certificate verification fails in the bulk directory.
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { loadDfxIdentity } from "../../upload/ic-identity.js";
import fetch from "node-fetch";

// Import the backend interface
import { idlFactory } from "../../../../../.dfx/local/canisters/backend/service.did.js";

// Test configuration
const TEST_NAME = "Simple Capsule Create Test (Bulk Directory)";
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

// Test 1: Create self-capsule (no subject parameter)
async function testCapsulesCreateSelf() {
  echoInfo("Testing capsules_create with no subject (self-capsule)...");

  try {
    const result = await backend.capsules_create([]);
    // Handle BigInt serialization for logging
    const logResult = JSON.stringify(result, (key, value) => (typeof value === "bigint" ? value.toString() : value), 2);
    echoInfo(`Response: ${logResult}`);

    if ("Ok" in result && result.Ok) {
      const capsule = result.Ok;
      echoInfo(`Created capsule with ID: ${capsule.id}`);
      echoInfo(
        `Subject: ${JSON.stringify(capsule.subject, (key, value) =>
          typeof value === "bigint" ? value.toString() : value
        )}`
      );
      echoInfo(
        `Owners: ${JSON.stringify(capsule.owners, (key, value) =>
          typeof value === "bigint" ? value.toString() : value
        )}`
      );
      echoPass("capsules_create call successful with no subject (creates self-capsule)");
      return true;
    } else {
      echoFail("capsules_create should return success and capsule data for self-capsule");
      // Handle BigInt serialization for logging
      const logResult = JSON.stringify(result, (key, value) => (typeof value === "bigint" ? value.toString() : value));
      echoInfo(`Response: ${logResult}`);
      return false;
    }
  } catch (error) {
    echoFail(`capsules_create call failed: ${error.message}`);
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
    echoError("Usage: node test_simple_capsule_create.mjs <BACKEND_CANISTER_ID>");
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

  const tests = [{ name: "capsules_create with no subject (self-capsule)", fn: testCapsulesCreateSelf }];

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
