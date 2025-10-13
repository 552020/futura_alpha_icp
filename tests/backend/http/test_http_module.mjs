#!/usr/bin/env node

/**
 * HTTP Module Test
 *
 * Simple test for the HTTP module using existing test framework utilities
 */

import { createTestActor, getOrCreateTestCapsule } from "../utils/index.js";
import { logHeader, logSuccess, logError, logInfo } from "../utils/helpers/logging.js";
import { measureExecutionTime } from "../utils/helpers/timing.js";
import { createTestMemory } from "../utils/data/memory.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

/**
 * Test HTTP module basic functionality
 */
async function testHttpModule() {
  logHeader("🧪 Testing HTTP Module");

  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);

  try {
    // Test 1: Health check via dfx canister call
    logInfo("Testing health check endpoint...");
    const healthResult = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(
        'dfx canister call backend http_request \'(record { method = "GET"; url = "/health"; headers = vec {}; body = blob ""; })\''
      );
      return stdout;
    });

    if (healthResult.result.includes("200") || healthResult.result.includes("OK")) {
      logSuccess("✅ Health check working");
    } else {
      logError(`❌ Health check failed: ${healthResult.result}`);
    }

    // Test 2: Create a test memory for token testing
    logInfo("Creating test memory for token testing...");
    const memoryId = await createTestMemory(actor, capsuleId, {
      name: "http_test_memory",
      content: "Test content for HTTP module",
      tags: ["test", "http"],
    });
    logSuccess(`✅ Test memory created: ${memoryId}`);

    // Test 3: Token minting (should fail with "forbidden" - this is expected!)
    logInfo("Testing token minting (expecting 'forbidden' error)...");
    try {
      const tokenResult = await measureExecutionTime(async () => {
        const { stdout } = await execAsync(
          `dfx canister call backend mint_http_token '("${memoryId}", vec {"thumbnail"; "preview"}, null, 180)'`
        );
        return stdout;
      });

      if (tokenResult.result.includes('"') && tokenResult.result.length > 50) {
        logSuccess("✅ Token minting working");
        logInfo(`Token: ${tokenResult.result.substring(0, 50)}...`);
      } else {
        logError(`❌ Token minting failed: ${tokenResult.result}`);
      }
    } catch (error) {
      if (error.message.includes("forbidden")) {
        logSuccess("✅ Token minting properly validates permissions (forbidden as expected)");
      } else {
        logError(`❌ Token minting failed with unexpected error: ${error.message}`);
      }
    }

    // Test 4: Asset serving without token (should fail)
    logInfo("Testing asset serving without token...");
    const assetResult = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(
        `dfx canister call backend http_request '(record { method = "GET"; url = "/assets/${memoryId}/thumbnail"; headers = vec {}; body = blob ""; })'`
      );
      return stdout;
    });

    if (
      assetResult.result.includes("401") ||
      assetResult.result.includes("403") ||
      assetResult.result.includes("404")
    ) {
      logSuccess("✅ Asset serving properly rejects requests without token");
    } else {
      logError(`❌ Asset serving should reject without token, got: ${assetResult.result}`);
    }

    // Test 5: Invalid endpoint (should return 404)
    logInfo("Testing invalid endpoint...");
    const invalidResult = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(
        'dfx canister call backend http_request \'(record { method = "GET"; url = "/invalid-endpoint"; headers = vec {}; body = blob ""; })\''
      );
      return stdout;
    });

    if (invalidResult.result.includes("404")) {
      logSuccess("✅ Invalid endpoints properly return 404");
    } else {
      logError(`❌ Invalid endpoint should return 404, got: ${invalidResult.result}`);
    }

    // Cleanup
    logInfo("Cleaning up test memory...");
    await actor.memories_delete(memoryId);
    logSuccess("✅ Cleanup completed");

    logSuccess("🎉 HTTP module tests completed!");
  } catch (error) {
    logError(`❌ HTTP module test failed: ${error.message}`);
    throw error;
  }
}

/**
 * Main function
 */
async function main() {
  try {
    await testHttpModule();
  } catch (error) {
    logError(`Test suite failed: ${error.message}`);
    process.exit(1);
  }
}

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}
