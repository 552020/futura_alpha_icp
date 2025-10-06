#!/usr/bin/env node

/**
 * Run Only Failing Tests from 2-Lane + 4-Asset System
 * 
 * This script runs only the tests that were failing in the main test suite.
 * Useful for debugging and iterating on fixes.
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity, makeMainnetAgent } from "./ic-identity.js";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { idlFactory } from "../../../../src/nextjs/src/ic/declarations/backend/backend.did.js";
import {
  validateFileSize,
  validateImageType,
  calculateFileHash,
  generateFileId,
  calculateDerivativeDimensions,
  calculateDerivativeSizes,
  createFileChunks,
  createProgressCallback,
  createAssetMetadata,
  createBlobReference,
  handleUploadError,
  validateUploadResponse,
  formatFileSize,
  formatUploadSpeed,
  formatDuration,
} from "./helpers.mjs";

// Import the test functions from the main test file
import {
  testCompleteSystem,
  testAssetRetrieval,
  testFullDeletionWorkflow,
  testSelectiveDeletionWorkflow,
} from "./test_upload_2lane_4asset_system.mjs";

// Test configuration
const TEST_NAME = "Failing Tests Only";

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

// Main test runner
async function main() {
  echoInfo(`Starting ${TEST_NAME}`);

  // Parse command line arguments
  const args = process.argv.slice(2);
  const backendCanisterId = args[0];
  const network = args[1] || "local"; // Default to local network

  if (!backendCanisterId) {
    echoFail("Usage: node run_failing_tests_only.mjs <CANISTER_ID> [mainnet|local]");
    echoFail("Example: node run_failing_tests_only.mjs uxrrr-q7777-77774-qaaaq-cai local");
    process.exit(1);
  }

  // Setup agent and backend based on network
  const identity = loadDfxIdentity();
  let agent;

  if (network === "mainnet") {
    echoInfo(`🌐 Connecting to mainnet (ic0.app)`);
    agent = makeMainnetAgent(identity);
  } else {
    echoInfo(`🏠 Connecting to local network (127.0.0.1:4943)`);
    agent = new HttpAgent({
      host: "http://127.0.0.1:4943",
      identity,
      fetch: (await import("node-fetch")).default,
    });
  }

  await agent.fetchRootKey();

  backend = Actor.createActor(idlFactory, {
    agent,
    canisterId: backendCanisterId,
  });

  // Run only the failing tests
  const failingTests = [
    { name: "Complete 2-Lane + 4-Asset System", fn: testCompleteSystem },
    { name: "Asset Retrieval", fn: testAssetRetrieval },
    { name: "Full Deletion Workflow", fn: testFullDeletionWorkflow },
    { name: "Selective Deletion Workflow", fn: testSelectiveDeletionWorkflow },
  ];

  let passed = 0;
  let failed = 0;

  for (const test of failingTests) {
    try {
      echoInfo(`Running: ${test.name}`);
      const result = await test.fn();
      if (result) {
        echoPass(test.name);
        passed++;
      } else {
        echoFail(test.name);
        failed++;
      }
    } catch (error) {
      echoFail(`${test.name}: ${error.message}`);
      failed++;
    }
  }

  // Summary
  echoInfo(`\n${TEST_NAME} Summary:`);
  echoInfo(`Total tests: ${failingTests.length}`);
  echoInfo(`Passed: ${passed}`);
  echoInfo(`Failed: ${failed}`);

  if (failed > 0) {
    echoFail("Some tests failed! ❌");
    process.exit(1);
  } else {
    echoPass("All failing tests now pass! ✅");
  }
}

// Run the test
main().catch((error) => {
  echoFail(`Test execution failed: ${error.message}`);
  process.exit(1);
});
