/**
 * Query Parameter Parsing Debug Test
 *
 * This test debugs the query parameter parsing issue:
 * 1. Create a memory with an image asset
 * 2. Mint an HTTP token
 * 3. Test different URL formats to see which ones work
 */

import { logHeader, logInfo, logSuccess, logError } from "../utils/helpers/logging.js";
import { createTestActor } from "../utils/core/actor.js";
import { createTestCapsule } from "../utils/helpers/capsule-creation.js";
import { createTestImageMemory } from "../utils/helpers/memory-creation.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function debugQueryParsing() {
  logHeader("🔍 Debugging Query Parameter Parsing");

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
      name: "query_debug_test.png",
      mimeType: "image/png",
      idempotencyKey: `query_debug_${Date.now()}`,
    });
    logSuccess(`✅ Memory created: ${memoryId}`);

    // Step 4: Get asset ID
    logInfo("Step 4: Getting asset ID...");
    const memoryResult = await actor.memories_read(memoryId);
    if (memoryResult.Ok && memoryResult.Ok.inline_assets && memoryResult.Ok.inline_assets.length > 0) {
      assetId = memoryResult.Ok.inline_assets[0].asset_id;
      logSuccess(`✅ Asset ID found: ${assetId}`);
    } else {
      logError("❌ No assets found in memory");
      throw new Error("No assets found in memory");
    }

    // Step 5: Mint HTTP token
    logInfo("Step 5: Minting HTTP token...");
    const token = await actor.mint_http_token(memoryId, ["thumbnail"], [], 180);
    logSuccess(`✅ Token minted: ${token.substring(0, 50)}...`);

    // Step 6: Test different URL formats
    logInfo("Step 6: Testing different URL formats...");
    const canisterId = "uxrrr-q7777-77774-qaaaq-cai";

    // Test 1: Token only (no asset ID)
    const url1 = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?token=${token}`;
    logInfo(`Test 1 - Token only: ${url1}`);
    try {
      const { stdout: output1 } = await execAsync(`curl -s -i ${url1}`);
      logInfo(`Response 1:\n${output1}`);
    } catch (error) {
      logError(`Test 1 failed: ${error.message}`);
    }

    // Test 2: Asset ID first, then token
    const url2 = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?id=${assetId}&token=${token}`;
    logInfo(`Test 2 - Asset ID first: ${url2}`);
    try {
      const { stdout: output2 } = await execAsync(`curl -s -i ${url2}`);
      logInfo(`Response 2:\n${output2}`);
    } catch (error) {
      logError(`Test 2 failed: ${error.message}`);
    }

    // Test 3: Token first, then asset ID
    const url3 = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?token=${token}&id=${assetId}`;
    logInfo(`Test 3 - Token first: ${url3}`);
    try {
      const { stdout: output3 } = await execAsync(`curl -s -i ${url3}`);
      logInfo(`Response 3:\n${output3}`);
    } catch (error) {
      logError(`Test 3 failed: ${error.message}`);
    }

    // Test 4: URL encoded token
    const encodedToken = encodeURIComponent(token);
    const url4 = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?id=${assetId}&token=${encodedToken}`;
    logInfo(`Test 4 - URL encoded token: ${url4}`);
    try {
      const { stdout: output4 } = await execAsync(`curl -s -i ${url4}`);
      logInfo(`Response 4:\n${output4}`);
    } catch (error) {
      logError(`Test 4 failed: ${error.message}`);
    }

    return { success: true, memoryId, assetId };
  } catch (error) {
    logError(`❌ Test failed: ${error.message}`);
    return { success: false, reason: "general_error", error: error.message };
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
  logHeader("🚀 Query Parameter Parsing Debug Test");

  const result = await debugQueryParsing();

  logHeader("📊 Debug Results");
  if (result.success) {
    logSuccess("✅ Query Parameter Parsing Debug COMPLETED");
    logInfo(`Memory ID: ${result.memoryId}`);
    logInfo(`Asset ID: ${result.assetId}`);
    logInfo("");
    logInfo("🔍 Check the responses above to see which URL format works");
    logInfo("This will help us identify the exact query parameter parsing issue");
  } else {
    logError(`❌ Test failed: ${result.reason.replace(/_/g, " ")}`);
    if (result.error) {
      logError(`Error: ${result.error}`);
    }
  }
}

main().catch(console.error);

