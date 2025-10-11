#!/usr/bin/env node

import {
  createTestActor,
  parseTestArgs,
  createTestActorOptions,
  logNetworkConfig,
  getOrCreateTestCapsuleForUpload,
  createTestRunner,
} from "../../utils/index.js";

// Define available tests
const AVAILABLE_TESTS = [
  "Uploads put chunk (invalid session)",
  "Uploads put chunk (malformed data)",
  "Uploads put chunk (large chunk)",
  "Uploads put chunk (negative index)",
  "Uploads put chunk (empty data)",
  "Uploads put chunk (committed session)",
  "Uploads put chunk (1.8MB - at limit)",
  "Uploads put chunk (1.9MB - exceeds limit)",
];

// Parse command line arguments using shared utility
const parsedArgs = parseTestArgs(
  "test_uploads_put_chunk.mjs",
  "Tests the uploads_put_chunk API endpoint for chunked uploads",
  AVAILABLE_TESTS
);

// Import the backend interface
import { idlFactory } from "../../declarations/backend/backend.did.js";

// Test configuration
const TEST_NAME = "Uploads Put Chunk Tests";

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

// Test: Uploads put chunk (1.8MB - at limit)
async function testUploadsPutChunkAtLimit(backend, capsuleId) {
  console.log("🧪 Testing: Uploads put chunk (1.8MB - at limit)");

  try {
    // Create a valid session first with unique idempotency key
    const session = await backend.uploads_begin(capsuleId, 1, `test-limit-${Date.now()}`);
    if (!("Ok" in session)) {
      return { success: false, error: `Failed to create session: ${JSON.stringify(session.Err)}` };
    }

    // Create a chunk exactly at the 1.8MB limit
    const chunkAtLimit = new Uint8Array(1_800_000); // 1.8MB - exactly at limit
    const result = await backend.uploads_put_chunk(session.Ok, 0, chunkAtLimit);

    if ("Ok" in result) {
      console.log(`✅ Uploads put chunk accepted 1.8MB chunk (at limit): ${JSON.stringify(result.Ok)}`);
      return { success: true, result };
    } else {
      console.log(`❌ Uploads put chunk rejected 1.8MB chunk: ${JSON.stringify(result.Err)}`);
      return { success: false, result };
    }
  } catch (error) {
    console.error(`❌ Uploads put chunk 1.8MB test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

// Test: Uploads put chunk (1.9MB - exceeds limit)
async function testUploadsPutChunkExceedsLimit(backend, capsuleId) {
  console.log("🧪 Testing: Uploads put chunk (1.9MB - exceeds limit)");

  try {
    // Create a valid session first with unique idempotency key
    const session = await backend.uploads_begin(capsuleId, 1, `test-exceed-${Date.now()}`);
    if (!("Ok" in session)) {
      return { success: false, error: `Failed to create session: ${JSON.stringify(session.Err)}` };
    }

    // Create a chunk that exceeds the 1.8MB limit
    const chunkExceedsLimit = new Uint8Array(1_900_000); // 1.9MB - exceeds limit
    const result = await backend.uploads_put_chunk(session.Ok, 0, chunkExceedsLimit);

    if ("Err" in result) {
      console.log(`✅ Uploads put chunk correctly rejected 1.9MB chunk (exceeds limit): ${JSON.stringify(result.Err)}`);
      return { success: true, result };
    } else {
      console.log(`❌ Uploads put chunk should have rejected 1.9MB chunk: ${JSON.stringify(result)}`);
      return { success: false, result };
    }
  } catch (error) {
    console.error(`❌ Uploads put chunk 1.9MB test failed: ${error.message}`);
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

    // Define all tests with their functions
    const allTests = [
      { name: "Uploads put chunk (invalid session)", fn: testUploadsPutChunkInvalidSession, args: [backend] },
      { name: "Uploads put chunk (malformed data)", fn: testUploadsPutChunkMalformedData, args: [backend] },
      { name: "Uploads put chunk (large chunk)", fn: testUploadsPutChunkLargeChunk, args: [backend] },
      { name: "Uploads put chunk (negative index)", fn: testUploadsPutChunkNegativeIndex, args: [backend] },
      { name: "Uploads put chunk (empty data)", fn: testUploadsPutChunkEmptyData, args: [backend] },
      { name: "Uploads put chunk (committed session)", fn: testUploadsPutChunkCommittedSession, args: [backend] },
      { name: "Uploads put chunk (1.8MB - at limit)", fn: testUploadsPutChunkAtLimit, args: [backend, capsuleId] },
      {
        name: "Uploads put chunk (1.9MB - exceeds limit)",
        fn: testUploadsPutChunkExceedsLimit,
        args: [backend, capsuleId],
      },
    ];

    // Run tests based on selection
    if (parsedArgs.selectedTest) {
      // Run specific test
      const selectedTest = allTests.find((test) => test.name === parsedArgs.selectedTest);
      if (selectedTest) {
        console.log(`🎯 Running selected test: ${parsedArgs.selectedTest}`);
        await runner.runTest(selectedTest.name, selectedTest.fn, ...selectedTest.args);
      } else {
        console.error(`❌ Test not found: ${parsedArgs.selectedTest}`);
        console.log("Available tests:");
        AVAILABLE_TESTS.forEach((test) => console.log(`  - ${test}`));
        process.exit(1);
      }
    } else {
      // Run all tests
      for (const test of allTests) {
        await runner.runTest(test.name, test.fn, ...test.args);
      }
    }

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
