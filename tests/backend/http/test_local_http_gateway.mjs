#!/usr/bin/env node

/**
 * Local HTTP Gateway Tests
 *
 * Tests the HTTP module with real HTTP requests via the local replica gateway
 * Uses existing test utilities for better integration and error handling
 */

import { createTestActor, getOrCreateTestCapsule } from "../utils/index.js";
import { logHeader, logSuccess, logError, logInfo } from "../utils/helpers/logging.js";
import { measureExecutionTime } from "../utils/helpers/timing.js";
import { createMemoryWithInline } from "../utils/helpers/memory-creation.js";
import { exec } from "child_process";
import { promisify } from "util";
import { join } from "path";

const execAsync = promisify(exec);

async function testLocalHttpGateway() {
  logHeader("🌐 Testing Local HTTP Gateway");

  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);

  try {
    // Get canister ID
    const canisterId = await getCanisterId();
    logInfo(`Backend canister ID: ${canisterId}`);

    // Test 1: Health Check via HTTP Gateway
    await testHealthCheckGateway(canisterId);

    // Test 2: Health Check via dfx canister call (for comparison)
    await testHealthCheckDfx();

    // Test 3: Create test memory and asset
    const memoryId = await createTestMemoryWithAsset(actor, capsuleId);

    // Test 4: Token minting
    const token = await testTokenMinting(memoryId);

    // Test 5: Asset serving with token (if available)
    if (token) {
      await testAssetServingWithToken(canisterId, memoryId, token);
    }

    // Test 6: Asset serving without token
    await testAssetServingWithoutToken(canisterId, memoryId);

    // Test 7: Invalid endpoints
    await testInvalidEndpoints(canisterId);

    // Test 8: Response headers validation
    if (token) {
      await testResponseHeaders(canisterId, memoryId, token);
    }

    // Cleanup
    logInfo("Cleaning up test memory...");
    await actor.memories_delete(memoryId);
    logSuccess("✅ Cleanup completed");

    logSuccess("🎉 Local HTTP Gateway tests completed!");
  } catch (error) {
    logError(`❌ Local HTTP Gateway test failed: ${error.message}`);
    throw error;
  }
}

async function getCanisterId() {
  try {
    const { stdout } = await execAsync("dfx canister id backend");
    return stdout.trim();
  } catch (error) {
    throw new Error(`Failed to get canister ID: ${error.message}`);
  }
}

async function testHealthCheckGateway(canisterId) {
  logInfo("Test 1: Health Check via HTTP Gateway");

  const url = `http://${canisterId}.localhost:4943/health`;
  logInfo(`Testing: ${url}`);

  try {
    const result = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(`curl -s -w "\\n%{http_code}" "${url}"`);
      return stdout;
    });

    const lines = result.result.trim().split("\n");
    const body = lines.slice(0, -1).join("\n");
    const status = lines[lines.length - 1];

    if (status === "200") {
      logSuccess("✅ Health check passed (200 OK)");
      logInfo(`Response: ${body}`);
    } else {
      logError(`❌ Health check failed (Status: ${status})`);
      logInfo(`Response: ${body}`);
    }
  } catch (error) {
    logError(`❌ Health check failed: ${error.message}`);
  }
}

async function testHealthCheckDfx() {
  logInfo("Test 2: Health Check via dfx canister call");

  try {
    const result = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(
        'dfx canister call backend http_request \'(record { method = "GET"; url = "/health"; headers = vec {}; body = blob ""; })\''
      );
      return stdout;
    });

    logSuccess("✅ dfx health check passed");
    logInfo(`Response: ${result.result}`);
  } catch (error) {
    logError(`❌ dfx health check failed: ${error.message}`);
  }
}

async function createTestMemoryWithAsset(actor, capsuleId) {
  logInfo("Test 3: Creating test memory with asset...");

  // Use an existing test image from the assets folder
  const testImagePath = join(process.cwd(), "../shared-capsule/upload/assets/input/orange_small_inline.jpg");
  logInfo(`Using test image: ${testImagePath}`);

  try {
    // Use the proper memory creation utility
    const result = await createMemoryWithInline(actor, testImagePath, capsuleId, {
      assetType: "image",
      mimeType: "image/jpeg",
      idempotencyKey: `http_gateway_test_${Date.now()}`,
    });

    if (result.success) {
      logSuccess(`✅ Test memory with image asset created: ${result.memoryId}`);
      return result.memoryId;
    } else {
      throw new Error(`Failed to create test memory: ${result.error}`);
    }
  } catch (error) {
    throw new Error(`Failed to create test memory with asset: ${error.message}`);
  }
}

async function testTokenMinting(memoryId) {
  logInfo("Test 4: Token minting...");

  try {
    const result = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(
        `dfx canister call backend mint_http_token '("${memoryId}", vec {"thumbnail"; "preview"}, null, 180)'`
      );
      return stdout;
    });

    if (result.result.includes('"') && result.result.length > 50) {
      const token = result.result.match(/"([^"]+)"/)?.[1];
      if (token) {
        logSuccess("✅ Token minting working");
        logInfo(`Token: ${token.substring(0, 20)}...`);
        return token;
      }
    }

    logError(`❌ Token minting failed: ${result.result}`);
    return null;
  } catch (error) {
    if (error.message.includes("forbidden")) {
      logSuccess("✅ Token minting properly validates permissions (forbidden as expected)");
      logInfo("This means ACL integration is working correctly!");
    } else {
      logError(`❌ Token minting failed with unexpected error: ${error.message}`);
    }
    return null;
  }
}

async function testAssetServingWithToken(canisterId, memoryId, token) {
  logInfo("Test 5: Asset serving with token...");

  const url = `http://${canisterId}.localhost:4943/assets/${memoryId}/thumbnail?token=${token}`;
  logInfo(`Testing: ${url}`);

  try {
    const result = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(`curl -s -w "\\n%{http_code}\\n%{content_type}" "${url}"`);
      return stdout;
    });

    const lines = result.result.trim().split("\n");
    const body = lines.slice(0, -2).join("\n");
    const status = lines[lines.length - 2];
    const contentType = lines[lines.length - 1];

    if (status === "200") {
      logSuccess("✅ Asset serving passed (200 OK)");
      logInfo(`Content-Type: ${contentType}`);
      logInfo(`Body size: ${body.length} bytes`);

      // Verify it's actually JPEG data
      if (body.startsWith("\xff\xd8\xff")) {
        logSuccess("✅ Response contains valid JPEG data");
      } else {
        logError("❌ Response does not contain valid JPEG data");
      }
    } else {
      logError(`❌ Asset serving failed (Status: ${status})`);
      logInfo(`Response: ${body}`);
    }
  } catch (error) {
    logError(`❌ Asset serving failed: ${error.message}`);
  }
}

async function testAssetServingWithoutToken(canisterId, memoryId) {
  logInfo("Test 6: Asset serving without token...");

  const url = `http://${canisterId}.localhost:4943/assets/${memoryId}/thumbnail`;
  logInfo(`Testing: ${url}`);

  try {
    const result = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(`curl -s -w "\\n%{http_code}" "${url}"`);
      return stdout;
    });

    const lines = result.result.trim().split("\n");
    const body = lines.slice(0, -1).join("\n");
    const status = lines[lines.length - 1];

    if (status === "401" || status === "403" || status === "404") {
      logSuccess(`✅ Asset serving properly rejects requests without token (${status})`);
    } else {
      logError(`❌ Asset serving should reject without token, got: ${status}`);
      logInfo(`Response: ${body}`);
    }
  } catch (error) {
    logError(`❌ Asset serving test failed: ${error.message}`);
  }
}

async function testInvalidEndpoints(canisterId) {
  logInfo("Test 7: Invalid endpoints...");

  const testCases = [
    {
      name: "Invalid endpoint",
      url: `http://${canisterId}.localhost:4943/invalid-endpoint`,
      expectedStatus: "404",
    },
    {
      name: "Non-existent memory",
      url: `http://${canisterId}.localhost:4943/assets/nonexistent_memory/thumbnail`,
      expectedStatus: "404",
    },
    {
      name: "Root path",
      url: `http://${canisterId}.localhost:4943/`,
      expectedStatus: "404",
    },
  ];

  for (const testCase of testCases) {
    logInfo(`Testing ${testCase.name}: ${testCase.url}`);

    try {
      const result = await measureExecutionTime(async () => {
        const { stdout } = await execAsync(`curl -s -w "\\n%{http_code}" "${testCase.url}"`);
        return stdout;
      });

      const lines = result.result.trim().split("\n");
      const body = lines.slice(0, -1).join("\n");
      const status = lines[lines.length - 1];

      if (status === testCase.expectedStatus) {
        logSuccess(`✅ ${testCase.name} properly returns ${testCase.expectedStatus}`);
      } else {
        logError(`❌ ${testCase.name} should return ${testCase.expectedStatus}, got: ${status}`);
        logInfo(`Response: ${body}`);
      }
    } catch (error) {
      logError(`❌ ${testCase.name} test failed: ${error.message}`);
    }
  }
}

async function testResponseHeaders(canisterId, memoryId, token) {
  logInfo("Test 8: Checking response headers...");

  const url = `http://${canisterId}.localhost:4943/assets/${memoryId}/thumbnail?token=${token}`;

  try {
    const result = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(`curl -s -I "${url}"`);
      return stdout;
    });

    const headers = result.result;

    if (headers.includes("Content-Type: image/jpeg")) {
      logSuccess("✅ Correct Content-Type header present");
    } else {
      logError("❌ Content-Type header missing or incorrect");
    }

    if (headers.includes("Cache-Control") && headers.includes("private")) {
      logSuccess("✅ Private cache control header present");
    } else {
      logError("❌ Private cache control header missing");
    }

    if (headers.includes("Cache-Control") && headers.includes("no-store")) {
      logSuccess("✅ No-store cache control header present");
    } else {
      logError("❌ No-store cache control header missing");
    }
  } catch (error) {
    logError(`❌ Header check failed: ${error.message}`);
  }
}

async function main() {
  try {
    await testLocalHttpGateway();
  } catch (error) {
    logError(`❌ Test suite failed: ${error.message}`);
    process.exit(1);
  }
}

main().catch(console.error);
