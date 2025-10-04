#!/usr/bin/env node

/**
 * Capsules Create Test (JavaScript)
 *
 * This test directly calls the capsules_create endpoint using the ICP agent
 * to verify the backend functionality and response format.
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { loadDfxIdentity } from "../shared-capsule/upload/ic-identity.js";
import fetch from "node-fetch";

// Import the backend interface
import { idlFactory } from "../../../src/nextjs/src/ic/declarations/backend/backend.did.js";

// Test configuration
const TEST_NAME = "Capsules Create Test (JavaScript)";
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

// Test 2: Create capsule for specific subject
async function testCapsulesCreateWithSubject() {
  echoInfo("Testing capsules_create with specific subject...");

  try {
    // Create a PersonRef for testing (using a valid principal)
    const subject = { Principal: Principal.fromText("2vxsx-fae") };
    const result = await backend.capsules_create([subject]);
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
      echoPass("capsules_create call successful with specific subject");
      return true;
    } else {
      echoFail("capsules_create should return success and capsule data for specific subject");
      // Handle BigInt serialization for logging
      const logResult = JSON.stringify(result, (key, value) => (typeof value === "bigint" ? value.toString() : value));
      echoInfo(`Response: ${logResult}`);
      return false;
    }
  } catch (error) {
    echoFail(`capsules_create call failed with specific subject: ${error.message}`);
    return false;
  }
}

// Test 3: Test idempotent behavior for self-capsule
async function testCapsulesCreateIdempotent() {
  echoInfo("Testing capsules_create idempotent behavior for self-capsule...");

  try {
    // First call
    const firstResult = await backend.capsules_create([]);
    // Handle BigInt serialization for logging
    const firstLogResult = JSON.stringify(
      firstResult,
      (key, value) => (typeof value === "bigint" ? value.toString() : value),
      2
    );
    echoInfo(`First call response: ${firstLogResult}`);

    if (!("Ok" in firstResult)) {
      echoFail("First capsules_create call failed");
      return false;
    }

    // Second call (should return existing capsule)
    const secondResult = await backend.capsules_create([]);
    // Handle BigInt serialization for logging
    const secondLogResult = JSON.stringify(
      secondResult,
      (key, value) => (typeof value === "bigint" ? value.toString() : value),
      2
    );
    echoInfo(`Second call response: ${secondLogResult}`);

    if ("Ok" in secondResult && secondResult.Ok) {
      echoPass("Second capsules_create call successful (idempotent behavior)");
      return true;
    } else {
      echoFail("Second capsules_create call should succeed for idempotent behavior");
      return false;
    }
  } catch (error) {
    echoFail(`Second capsules_create call failed: ${error.message}`);
    return false;
  }
}

// Test 4: Test response structure validation
async function testResponseStructure() {
  echoInfo("Testing response structure validation...");

  try {
    const result = await backend.capsules_create([]);
    // Handle BigInt serialization for logging
    const logResult = JSON.stringify(result, (key, value) => (typeof value === "bigint" ? value.toString() : value), 2);
    echoInfo(`Response: ${logResult}`);

    if ("Ok" in result && result.Ok) {
      const capsule = result.Ok;

      // Check for required fields
      const requiredFields = ["id", "subject", "owners", "created_at", "updated_at"];
      const missingFields = requiredFields.filter((field) => !(field in capsule));

      if (missingFields.length === 0) {
        echoPass("Response contains required capsule fields");
        return true;
      } else {
        echoFail(`Response missing required capsule fields: ${missingFields.join(", ")}`);
        return false;
      }
    } else {
      echoFail("Response should be Ok variant with capsule data");
      echoInfo(`Got: ${JSON.stringify(result)}`);
      return false;
    }
  } catch (error) {
    echoFail(`capsules_create call failed during structure validation: ${error.message}`);
    return false;
  }
}

// Test 5: Test with authenticated user
async function testAuthenticatedUser() {
  echoInfo("Testing capsules_create with authenticated user...");

  try {
    const result = await backend.capsules_create([]);
    // Handle BigInt serialization for logging
    const logResult = JSON.stringify(result, (key, value) => (typeof value === "bigint" ? value.toString() : value), 2);
    echoInfo(`Response: ${logResult}`);

    if ("Ok" in result && result.Ok) {
      echoPass("capsules_create call successful with authenticated user");
      return true;
    } else {
      echoFail("capsules_create should succeed with authenticated user");
      return false;
    }
  } catch (error) {
    echoFail(`capsules_create call failed with authenticated user: ${error.message}`);
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
    echoError("Usage: node test_capsules_create_mjs.mjs <BACKEND_CANISTER_ID>");
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

  const tests = [
    { name: "capsules_create with no subject (self-capsule)", fn: testCapsulesCreateSelf },
    { name: "capsules_create with specific subject", fn: testCapsulesCreateWithSubject },
    { name: "capsules_create idempotent behavior", fn: testCapsulesCreateIdempotent },
    { name: "Response structure validation", fn: testResponseStructure },
    { name: "Authenticated user access", fn: testAuthenticatedUser },
  ];

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
