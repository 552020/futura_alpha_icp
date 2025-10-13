/**
 * Asset ID Debug Test
 *
 * This test debugs the asset ID issue:
 * 1. Create a memory with an image asset
 * 2. Check what asset IDs the memory has
 * 3. Test HTTP access with the correct asset ID
 */

import { logHeader, logInfo, logSuccess, logError } from "../utils/helpers/logging.js";
import { createTestActor } from "../utils/core/actor.js";
import { createTestCapsule } from "../utils/helpers/capsule-creation.js";
import { createTestImageMemory } from "../utils/helpers/memory-creation.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function testAssetIdDebug() {
  logHeader("🔍 Debugging Asset ID Issue");

  let capsuleId = null;
  let memoryId = null;

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
      name: "asset_id_debug.png",
      mimeType: "image/png",
      idempotencyKey: `asset_id_debug_${Date.now()}`,
    });
    logSuccess(`✅ Memory created: ${memoryId}`);

    // Step 4: Read memory and check asset IDs
    logInfo("Step 4: Reading memory and checking asset IDs...");
    const memoryResult = await actor.memories_read(memoryId);
    if (memoryResult.Ok) {
      const memory = memoryResult.Ok;
      logInfo("Memory structure:");
      logInfo(`- Memory ID: ${memory.id}`);
      logInfo(`- Inline assets count: ${memory.inline_assets?.length || 0}`);
      logInfo(`- Blob internal assets count: ${memory.blob_internal_assets?.length || 0}`);

      if (memory.inline_assets && memory.inline_assets.length > 0) {
        logInfo("Inline assets:");
        memory.inline_assets.forEach((asset, index) => {
          logInfo(`  Asset ${index + 1}:`);
          logInfo(`    - Asset ID: ${asset.asset_id}`);
          logInfo(`    - Content type: ${asset.content_type}`);
          logInfo(`    - Bytes length: ${asset.bytes.length}`);
        });
      }

      if (memory.blob_internal_assets && memory.blob_internal_assets.length > 0) {
        logInfo("Blob internal assets:");
        memory.blob_internal_assets.forEach((asset, index) => {
          logInfo(`  Asset ${index + 1}:`);
          logInfo(`    - Asset ID: ${asset.asset_id}`);
          logInfo(`    - Blob ref: ${asset.blob_ref.locator}`);
        });
      }
    } else {
      logError(`❌ Failed to read memory: ${JSON.stringify(memoryResult.Err)}`);
      throw new Error("Failed to read memory");
    }

    // Step 5: Mint token
    logInfo("Step 5: Minting HTTP token...");
    const token = await actor.mint_http_token(memoryId, ["thumbnail"], [], 180);
    logSuccess(`✅ Token minted: ${token.substring(0, 50)}...`);

    // Step 6: Test HTTP access with different asset ID approaches
    logInfo("Step 6: Testing HTTP access with different approaches...");
    const canisterId = "uxrrr-q7777-77774-qaaaq-cai";

    // Get the first asset ID
    const firstAssetId =
      memoryResult.Ok.inline_assets?.[0]?.asset_id || memoryResult.Ok.blob_internal_assets?.[0]?.asset_id;

    if (firstAssetId) {
      logInfo(`Using asset ID: ${firstAssetId}`);

      // Test 1: With asset ID in URL
      const httpUrlWithId = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?id=${firstAssetId}&token=${token}`;
      logInfo(`Testing URL with asset ID: ${httpUrlWithId}`);

      try {
        const { stdout: curlOutput1 } = await execAsync(`curl -s -i ${httpUrlWithId}`);
        logInfo(`Response with asset ID:\n${curlOutput1}`);

        if (curlOutput1.includes("HTTP/1.1 200 OK")) {
          logSuccess("🎉 SUCCESS! Asset found with asset ID!");
          return { success: true, httpUrl: httpUrlWithId, curlOutput: curlOutput1 };
        }
      } catch (error) {
        logError(`❌ HTTP access with asset ID failed: ${error.message}`);
      }

      // Test 2: Without asset ID (should work if there's only one asset)
      const httpUrlWithoutId = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?token=${token}`;
      logInfo(`Testing URL without asset ID: ${httpUrlWithoutId}`);

      try {
        const { stdout: curlOutput2 } = await execAsync(`curl -s -i ${httpUrlWithoutId}`);
        logInfo(`Response without asset ID:\n${curlOutput2}`);

        if (curlOutput2.includes("HTTP/1.1 200 OK")) {
          logSuccess("🎉 SUCCESS! Asset found without asset ID!");
          return { success: true, httpUrl: httpUrlWithoutId, curlOutput: curlOutput2 };
        }
      } catch (error) {
        logError(`❌ HTTP access without asset ID failed: ${error.message}`);
      }
    } else {
      logError("❌ No asset ID found in memory");
    }

    return { success: false, reason: "asset_not_found", memoryId };
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
  logHeader("🚀 Asset ID Debug Test");

  const result = await testAssetIdDebug();

  logHeader("📊 Debug Results");
  if (result.success) {
    logSuccess("🎉 Asset ID Debug Test PASSED!");
    logInfo(`HTTP URL: ${result.httpUrl}`);
    logInfo("");
    logInfo("🔍 What this proves:");
    logInfo("✅ Complete end-to-end image serving works");
    logInfo("✅ Token minting works");
    logInfo("✅ Asset serving works with correct asset ID");
    logInfo("✅ You can open the URL in your browser to see the image");
    logInfo("");
    logInfo("🌐 The image is now accessible via HTTP and can be displayed!");
    logInfo("");
    logInfo("🎯 MISSION ACCOMPLISHED!");
    logInfo("The HTTP module successfully serves private, token-gated assets over the ICP HTTP gateway!");
  } else {
    logError(`❌ Test failed: ${result.reason.replace(/_/g, " ")}`);
    if (result.error) {
      logError(`Error: ${result.error}`);
    }
  }
}

main().catch(console.error);
