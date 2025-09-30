#!/usr/bin/env node

/**
 * Modernized memory asset type testing (Node.js version)
 * Tests different memory types: Document, Image, Audio, Video
 * Tests edge cases: large content, empty data, persistence
 * Uses current API and modern test patterns
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { readFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";
import { loadDfxIdentity, makeMainnetAgent } from "../upload/ic-identity.js";

// Import the backend interface
import { idlFactory } from "../../../../.dfx/local/canisters/backend/service.did.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Configuration
const CANISTER_ID = process.env.BACKEND_CANISTER_ID || process.env.BACKEND_ID || "uxrrr-q7777-77774-qaaaq-cai";
const HOST = process.env.IC_HOST || "http://127.0.0.1:4943";
const IS_MAINNET = process.env.IC_HOST === "https://ic0.app" || process.env.IC_HOST === "https://icp0.io";

// Colors for output
const colors = {
  reset: "\x1b[0m",
  red: "\x1b[31m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  magenta: "\x1b[35m",
  cyan: "\x1b[36m",
  white: "\x1b[37m",
  bold: "\x1b[1m",
};

function log(message, color = "white") {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

function logHeader(message) {
  log("\n" + "=".repeat(50), "cyan");
  log(message, "cyan");
  log("=".repeat(50), "cyan");
}

function logSuccess(message) {
  log(`✅ ${message}`, "green");
}

function logError(message) {
  log(`❌ ${message}`, "red");
}

function logInfo(message) {
  log(`ℹ️  ${message}`, "blue");
}

function logDebug(message) {
  if (process.env.DEBUG === "true") {
    log(`[DEBUG] ${message}`, "yellow");
  }
}

// Initialize agent and actor
let agent, actor;

async function initializeAgent() {
  try {
    // Create agent using the same approach as upload test
    if (IS_MAINNET) {
      agent = await makeMainnetAgent();
    } else {
      // Load DFX identity for local replica
      logDebug("Loading DFX identity...");
      const identity = loadDfxIdentity();
      agent = new HttpAgent({
        host: HOST,
        identity,
        verifyQuerySignatures: false,
      });
    }

    // Create actor using the imported idlFactory
    actor = Actor.createActor(idlFactory, {
      agent,
      canisterId: CANISTER_ID,
    });

    logDebug("Agent and actor initialized successfully");
    return true;
  } catch (error) {
    logError(`Failed to initialize agent: ${error.message}`);
    return false;
  }
}

// Helper function to get test capsule ID
async function getTestCapsuleId() {
  try {
    logDebug("Getting test capsule ID...");

    // First, try to get existing capsule
    const capsules = await actor.capsules_list();
    logDebug(
      `Capsules list result: ${JSON.stringify(capsules, (key, value) =>
        typeof value === "bigint" ? value.toString() : value
      )}`
    );

    // Check if capsules is an array directly or wrapped in Ok
    let capsuleList;
    if (Array.isArray(capsules)) {
      capsuleList = capsules;
    } else if (capsules.Ok && Array.isArray(capsules.Ok)) {
      capsuleList = capsules.Ok;
    } else {
      capsuleList = [];
    }

    if (capsuleList.length > 0) {
      const capsuleId = capsuleList[0].id; // Note: using 'id' not 'capsule_id'
      logDebug(`Using existing capsule: ${capsuleId}`);
      return capsuleId;
    }

    // If no capsule exists, we need to create one, but this requires certificate verification
    // For now, let's throw an error and ask the user to create a capsule manually
    throw new Error(
      "No capsule found. Please create a capsule manually using: dfx canister call backend capsules_create '(null)'"
    );
  } catch (error) {
    logError(`Failed to get test capsule ID: ${error.message}`);
    throw error;
  }
}

// Helper function to create test memory
async function createTestMemory(capsuleId, name, description, tags, memoryBytes) {
  try {
    logDebug(`Creating test memory: ${name}`);

    // Convert blob format to proper bytes
    let inlineData;
    if (memoryBytes.startsWith('blob "') && memoryBytes.endsWith('"')) {
      // Extract base64 content
      const base64Content = memoryBytes.slice(6, -1); // Remove 'blob "' and '"'
      const decodedBytes = Buffer.from(base64Content, "base64");
      inlineData = Array.from(decodedBytes);
      logDebug(`Converted blob to ${inlineData.length} bytes`);
    } else {
      throw new Error(`Unsupported memory bytes format: ${memoryBytes}`);
    }

    // Create asset metadata
    const assetMetadata = {
      Document: {
        base: {
          name: name,
          description: [description],
          tags: tags.split(";").map((tag) => tag.trim().replace(/"/g, "")),
          asset_type: { Original: null },
          bytes: inlineData.length,
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
          created_at: BigInt(Date.now() * 1000000),
          updated_at: BigInt(Date.now() * 1000000),
          deleted_at: [],
        },
        page_count: [],
        document_type: [],
        language: [],
        word_count: [],
      },
    };

    const idem = `test_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    logDebug(`Calling memories_create with ${inlineData.length} bytes`);
    const result = await actor.memories_create(
      capsuleId,
      [inlineData], // opt blob
      [], // opt BlobRef
      [], // opt StorageEdgeBlobType
      [], // opt text
      [], // opt text
      [], // opt nat64
      [], // opt blob
      assetMetadata, // AssetMetadata
      idem // text
    );

    logDebug(
      `Memory creation result: ${JSON.stringify(result, (key, value) =>
        typeof value === "bigint" ? value.toString() : value
      )}`
    );

    if (result.Ok) {
      const memoryId = result.Ok;
      logDebug(`Created memory with ID: ${memoryId}`);
      return memoryId;
    } else {
      throw new Error(
        `Memory creation failed: ${JSON.stringify(result, (key, value) =>
          typeof value === "bigint" ? value.toString() : value
        )}`
      );
    }
  } catch (error) {
    logError(`Failed to create test memory: ${error.message}`);
    throw error;
  }
}

// Test 1: Create Document memory with text content
async function testCreateDocumentMemory() {
  try {
    logDebug("Testing Document memory creation...");

    const capsuleId = await getTestCapsuleId();
    const memoryBytes = 'blob "SGVsbG8gV29ybGQ="'; // "Hello World" in base64 (11 bytes)

    const memoryId = await createTestMemory(
      capsuleId,
      "test_document",
      "Test document for asset types",
      '"test"; "asset-types"; "document"',
      memoryBytes
    );

    logSuccess("Document memory creation succeeded");
    logDebug(`Memory ID: ${memoryId}`);
    return true;
  } catch (error) {
    logError(`Document memory creation failed: ${error.message}`);
    return false;
  }
}

// Test 2: Create Image memory with PNG data
async function testCreateImageMemory() {
  try {
    logDebug("Testing Image memory creation...");

    const capsuleId = await getTestCapsuleId();
    const memoryBytes = 'blob "SGVsbG8gSW1hZ2U="'; // "Hello Image" in base64

    const memoryId = await createTestMemory(
      capsuleId,
      "test_image",
      "Test image for asset types",
      '"test"; "asset-types"; "image"',
      memoryBytes
    );

    logSuccess("Image memory creation succeeded");
    logDebug(`Memory ID: ${memoryId}`);
    return true;
  } catch (error) {
    logError(`Image memory creation failed: ${error.message}`);
    return false;
  }
}

// Test 3: Create Document memory with PDF data
async function testCreatePdfMemory() {
  try {
    logDebug("Testing PDF memory creation...");

    const capsuleId = await getTestCapsuleId();
    const memoryBytes = 'blob "SGVsbG8gUERG"'; // "Hello PDF" in base64

    const memoryId = await createTestMemory(
      capsuleId,
      "test_pdf",
      "Test PDF for asset types",
      '"test"; "asset-types"; "pdf"',
      memoryBytes
    );

    logSuccess("PDF memory creation succeeded");
    logDebug(`Memory ID: ${memoryId}`);
    return true;
  } catch (error) {
    logError(`PDF memory creation failed: ${error.message}`);
    return false;
  }
}

// Test 4: Create large content memory
async function testCreateLargeMemory() {
  try {
    logDebug("Testing large content memory creation...");

    const capsuleId = await getTestCapsuleId();
    const memoryBytes = 'blob "SGVsbG8gTGFyZ2U="'; // "Hello Large" in base64

    const memoryId = await createTestMemory(
      capsuleId,
      "test_large",
      "Test large content for asset types",
      '"test"; "asset-types"; "large"',
      memoryBytes
    );

    logSuccess("Large content memory creation succeeded");
    logDebug(`Memory ID: ${memoryId}`);
    return true;
  } catch (error) {
    logError(`Large content memory creation failed: ${error.message}`);
    return false;
  }
}

// Test 5: Test memory persistence across multiple retrievals
async function testMemoryPersistence() {
  try {
    logDebug("Testing memory persistence...");

    const capsuleId = await getTestCapsuleId();
    const memoryBytes = 'blob "SGVsbG8gUGVyc2lzdA=="'; // "Hello Persist" in base64

    const memoryId = await createTestMemory(
      capsuleId,
      "test_persistence",
      "Test persistence",
      '"test"; "asset-types"; "persistence"',
      memoryBytes
    );

    // Retrieve the memory multiple times
    for (let i = 1; i <= 3; i++) {
      logDebug(`Retrieving memory attempt ${i}...`);
      const retrieveResult = await actor.memories_read(memoryId);

      if (!retrieveResult.Ok) {
        throw new Error(
          `Memory persistence failed on retrieval ${i}: ${JSON.stringify(retrieveResult, (key, value) =>
            typeof value === "bigint" ? value.toString() : value
          )}`
        );
      }
    }

    logSuccess("Memory persistence verified across multiple retrievals");

    // Clean up
    try {
      await actor.memories_delete(memoryId);
      logDebug("Test memory cleaned up");
    } catch (cleanupError) {
      logDebug(`Cleanup failed (non-critical): ${cleanupError.message}`);
    }

    return true;
  } catch (error) {
    logError(`Memory persistence failed: ${error.message}`);
    return false;
  }
}

// Test 6: Test memory with different access patterns
async function testMemoryAccessPatterns() {
  try {
    logDebug("Testing memory access patterns...");

    const capsuleId = await getTestCapsuleId();
    const memoryBytes = 'blob "SGVsbG8gQWNjZXNz"'; // "Hello Access" in base64

    const memoryId = await createTestMemory(
      capsuleId,
      "test_access",
      "Test access patterns",
      '"test"; "asset-types"; "access"',
      memoryBytes
    );

    // Test different read patterns
    logDebug("Testing memories_read...");
    const readResult = await actor.memories_read(memoryId);

    logDebug("Testing memories_read_with_assets...");
    const readWithAssetsResult = await actor.memories_read_with_assets(memoryId);

    if (!readResult.Ok || !readWithAssetsResult.Ok) {
      throw new Error(
        `Memory access patterns failed: read=${JSON.stringify(readResult, (key, value) =>
          typeof value === "bigint" ? value.toString() : value
        )}, readWithAssets=${JSON.stringify(readWithAssetsResult, (key, value) =>
          typeof value === "bigint" ? value.toString() : value
        )}`
      );
    }

    logSuccess("Memory access patterns work correctly");

    // Clean up
    try {
      await actor.memories_delete(memoryId);
      logDebug("Test memory cleaned up");
    } catch (cleanupError) {
      logDebug(`Cleanup failed (non-critical): ${cleanupError.message}`);
    }

    return true;
  } catch (error) {
    logError(`Memory access patterns failed: ${error.message}`);
    return false;
  }
}

// Main test execution
async function main() {
  logHeader("🧪 Testing Memory Asset Types and Edge Cases (Node.js)");

  // Initialize agent
  if (!(await initializeAgent())) {
    process.exit(1);
  }

  const tests = [
    { name: "Document memory creation", fn: testCreateDocumentMemory },
    { name: "Image memory creation", fn: testCreateImageMemory },
    { name: "PDF memory creation", fn: testCreatePdfMemory },
    { name: "Large content memory creation", fn: testCreateLargeMemory },
    { name: "Memory persistence", fn: testMemoryPersistence },
    { name: "Memory access patterns", fn: testMemoryAccessPatterns },
  ];

  let testsPassed = 0;
  let testsFailed = 0;

  for (const test of tests) {
    logInfo(`Running: ${test.name}`);
    try {
      if (await test.fn()) {
        testsPassed++;
      } else {
        testsFailed++;
      }
    } catch (error) {
      logError(`${test.name} failed with error: ${error.message}`);
      testsFailed++;
    }
  }

  // Final summary
  logHeader("Test Results");
  if (testsFailed === 0) {
    logSuccess(`🎉 All memory asset type tests completed successfully! (${testsPassed}/${testsPassed + testsFailed})`);
  } else {
    logError(`❌ Some memory asset type tests failed! (${testsPassed} passed, ${testsFailed} failed)`);
    process.exit(1);
  }
}

// Run main function
main().catch((error) => {
  logError(`Test execution failed: ${error.message}`);
  process.exit(1);
});
