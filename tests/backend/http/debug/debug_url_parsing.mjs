/**
 * Debug URL Parsing Test
 *
 * This test helps us understand how the HTTP module parses URLs with multiple query parameters
 */

import { logHeader, logInfo, logSuccess, logError } from "../../utils/helpers/logging.js";
import { createTestActor } from "../../utils/core/actor.js";
import { createTestCapsule } from "../../utils/helpers/capsule-creation.js";
import { createTestImageMemory } from "../../utils/helpers/memory-creation.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function debugUrlParsing() {
  logHeader("🔍 Debugging URL Parsing");

  let capsuleId = null;
  let memoryId = null;
  let assetId = null;

  try {
    // Step 1: Create test actor
    logInfo("Step 1: Creating test actor...");
    const { actor } = await createTestActor();
    logSuccess("✅ Test actor created");

    // Step 2: Create capsule
    logInfo("Step 2: Creating capsule...");
    capsuleId = await createTestCapsule(actor);
    logSuccess(`✅ Capsule created: ${capsuleId}`);

    // Step 3: Create memory
    logInfo("Step 3: Creating memory...");
    memoryId = await createTestImageMemory(actor, capsuleId, {
      name: "debug_test.png",
      mimeType: "image/png",
      idempotencyKey: `debug_${Date.now()}`,
    });
    logSuccess(`✅ Memory created: ${memoryId}`);

    // Step 4: Get asset ID
    logInfo("Step 4: Getting asset ID...");
    const memoryResult = await actor.memories_read(memoryId);
    if (memoryResult.Ok && memoryResult.Ok.inline_assets && memoryResult.Ok.inline_assets.length > 0) {
      assetId = memoryResult.Ok.inline_assets[0].asset_id;
      logSuccess(`✅ Asset ID found: ${assetId}`);
    } else {
      throw new Error("No assets found in memory");
    }

    // Step 5: Mint token
    logInfo("Step 5: Minting token...");
    const token = await actor.mint_http_token(memoryId, ["thumbnail"], [], 180);
    logSuccess(`✅ Token minted: ${token.substring(0, 50)}...`);

    // Step 6: Test different URL formats
    const canisterId = "uxrrr-q7777-77774-qaaaq-cai";

    logInfo("Step 6: Testing different URL formats...");

    // Test 1: Only token parameter
    logInfo("Test 1: Only token parameter");
    const url1 = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?token=${token}`;
    logInfo(`URL 1: ${url1}`);

    try {
      const { stdout: response1 } = await execAsync(`curl -s -i ${url1}`);
      if (response1.includes("HTTP/1.1 200 OK")) {
        logSuccess("✅ Test 1 PASSED - Only token works");
      } else if (response1.includes("Missing token")) {
        logError("❌ Test 1 FAILED - Missing token");
      } else {
        logInfo(`Test 1 response: ${response1.split("\n")[0]}`);
      }
    } catch (error) {
      logError(`Test 1 error: ${error.message}`);
    }

    // Test 2: Token first, then id
    logInfo("Test 2: Token first, then id");
    const url2 = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?token=${token}&id=${assetId}`;
    logInfo(`URL 2: ${url2}`);

    try {
      const { stdout: response2 } = await execAsync(`curl -s -i ${url2}`);
      if (response2.includes("HTTP/1.1 200 OK")) {
        logSuccess("✅ Test 2 PASSED - Token first works");
      } else if (response2.includes("Missing token")) {
        logError("❌ Test 2 FAILED - Missing token");
      } else {
        logInfo(`Test 2 response: ${response2.split("\n")[0]}`);
      }
    } catch (error) {
      logError(`Test 2 error: ${error.message}`);
    }

    // Test 3: ID first, then token
    logInfo("Test 3: ID first, then token");
    const url3 = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?id=${assetId}&token=${token}`;
    logInfo(`URL 3: ${url3}`);

    try {
      const { stdout: response3 } = await execAsync(`curl -s -i ${url3}`);
      if (response3.includes("HTTP/1.1 200 OK")) {
        logSuccess("✅ Test 3 PASSED - ID first works");
      } else if (response3.includes("Missing token")) {
        logError("❌ Test 3 FAILED - Missing token");
      } else {
        logInfo(`Test 3 response: ${response3.split("\n")[0]}`);
      }
    } catch (error) {
      logError(`Test 3 error: ${error.message}`);
    }

    // Test 4: URL encoded token
    logInfo("Test 4: URL encoded token");
    const encodedToken = encodeURIComponent(token);
    const url4 = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?id=${assetId}&token=${encodedToken}`;
    logInfo(`URL 4: ${url4}`);

    try {
      const { stdout: response4 } = await execAsync(`curl -s -i ${url4}`);
      if (response4.includes("HTTP/1.1 200 OK")) {
        logSuccess("✅ Test 4 PASSED - URL encoded token works");
      } else if (response4.includes("Missing token")) {
        logError("❌ Test 4 FAILED - Missing token");
      } else {
        logInfo(`Test 4 response: ${response4.split("\n")[0]}`);
      }
    } catch (error) {
      logError(`Test 4 error: ${error.message}`);
    }

    return { success: true, memoryId, assetId };
  } catch (error) {
    logError(`❌ Test failed: ${error.message}`);
    return { success: false, error: error.message };
  } finally {
    // Cleanup
    if (memoryId) {
      logInfo("Cleaning up memory...");
      try {
        await actor.memories_delete(memoryId, false);
        logSuccess("✅ Memory cleaned up");
      } catch (cleanupError) {
        logError(`❌ Cleanup failed: ${cleanupError.message}`);
      }
    }
  }
}

async function main() {
  logHeader("🚀 URL Parsing Debug Test");

  const result = await debugUrlParsing();

  logHeader("📊 Debug Results");
  if (result.success) {
    logSuccess("✅ URL parsing debug completed");
    logInfo(`Memory ID: ${result.memoryId}`);
    logInfo(`Asset ID: ${result.assetId}`);
    logInfo("");
    logInfo("🔍 This test helps us understand:");
    logInfo("✅ Which URL format works for token parsing");
    logInfo("✅ Whether the issue is with parameter order");
    logInfo("✅ Whether URL encoding is needed");
    logInfo("✅ Whether the HTTP module parses query parameters correctly");
  } else {
    logError(`❌ Debug failed: ${result.error}`);
  }
}

main().catch(console.error);
