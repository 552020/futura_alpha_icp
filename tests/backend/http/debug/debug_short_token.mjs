/**
 * Debug Short Token Test
 *
 * This test helps us understand if the issue is URL length by using a shorter token
 */

import { logHeader, logInfo, logSuccess, logError } from "../../utils/helpers/logging.js";
import { createTestActor } from "../../utils/core/actor.js";
import { createTestCapsule } from "../../utils/helpers/capsule-creation.js";
import { createTestImageMemory } from "../../utils/helpers/memory-creation.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function debugShortToken() {
  logHeader("🔍 Debugging Short Token");

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
      name: "short_token_test.png",
      mimeType: "image/png",
      idempotencyKey: `short_token_${Date.now()}`,
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

    // Step 5: Mint token with very short TTL to make it shorter
    logInfo("Step 5: Minting short token...");
    const token = await actor.mint_http_token(memoryId, ["thumbnail"], [], 1); // 1 second TTL
    logSuccess(`✅ Short token minted: ${token.substring(0, 50)}...`);
    logInfo(`Token length: ${token.length}`);

    // Step 6: Test with short token
    const canisterId = "uxrrr-q7777-77774-qaaaq-cai";

    // Test 1: Only token (should work with fallback)
    logInfo("Test 1: Only token parameter");
    const url1 = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?token=${token}`;
    logInfo(`URL 1 length: ${url1.length}`);
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

    // Test 2: Token with asset ID
    logInfo("Test 2: Token with asset ID");
    const url2 = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?id=${assetId}&token=${token}`;
    logInfo(`URL 2 length: ${url2.length}`);
    logInfo(`URL 2: ${url2}`);

    try {
      const { stdout: response2 } = await execAsync(`curl -s -i ${url2}`);
      if (response2.includes("HTTP/1.1 200 OK")) {
        logSuccess("✅ Test 2 PASSED - Token with asset ID works");
      } else if (response2.includes("Missing token")) {
        logError("❌ Test 2 FAILED - Missing token");
      } else {
        logInfo(`Test 2 response: ${response2.split("\n")[0]}`);
      }
    } catch (error) {
      logError(`Test 2 error: ${error.message}`);
    }

    return { success: true, memoryId, assetId, token };
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
  logHeader("🚀 Short Token Debug Test");

  const result = await debugShortToken();

  logHeader("📊 Debug Results");
  if (result.success) {
    logSuccess("✅ Short token debug completed");
    logInfo(`Memory ID: ${result.memoryId}`);
    logInfo(`Asset ID: ${result.assetId}`);
    logInfo(`Token: ${result.token.substring(0, 50)}...`);
    logInfo("");
    logInfo("🔍 This test helps us understand:");
    logInfo("✅ Whether the issue is URL length");
    logInfo("✅ Whether shorter tokens work better");
    logInfo("✅ Whether the fallback mechanism works with shorter tokens");
  } else {
    logError(`❌ Debug failed: ${result.error}`);
  }
}

main().catch(console.error);
