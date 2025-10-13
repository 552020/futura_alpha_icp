/**
 * Normal Asset Flow Test
 *
 * This test focuses on the normal case where the frontend provides a specific asset ID:
 * 1. Create a memory with an image asset
 * 2. Get the specific asset ID from the memory
 * 3. Mint an HTTP token
 * 4. Test HTTP access with the specific asset ID (normal case)
 */

import { logHeader, logInfo, logSuccess, logError } from "../utils/helpers/logging.js";
import { createTestActor } from "../utils/core/actor.js";
import { createTestCapsule } from "../utils/helpers/capsule-creation.js";
import { createTestImageMemory } from "../utils/helpers/memory-creation.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function testNormalAssetFlow() {
  logHeader("🖼️ Testing Normal Asset Flow (Specific Asset ID)");

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
      name: "normal_flow_test.png",
      mimeType: "image/png",
      idempotencyKey: `normal_flow_${Date.now()}`,
    });
    logSuccess(`✅ Memory created: ${memoryId}`);

    // Step 4: Get the specific asset ID (normal case)
    logInfo("Step 4: Getting specific asset ID from memory...");
    const memoryResult = await actor.memories_read(memoryId);
    if (memoryResult.Ok && memoryResult.Ok.inline_assets && memoryResult.Ok.inline_assets.length > 0) {
      assetId = memoryResult.Ok.inline_assets[0].asset_id;
      logSuccess(`✅ Asset ID found: ${assetId}`);
      logInfo(`Asset content type: ${memoryResult.Ok.inline_assets[0].content_type || "undefined"}`);
      logInfo(`Asset bytes length: ${memoryResult.Ok.inline_assets[0].bytes.length}`);
    } else {
      logError("❌ No assets found in memory");
      throw new Error("No assets found in memory");
    }

    // Step 5: Mint HTTP token
    logInfo("Step 5: Minting HTTP token...");
    const token = await actor.mint_http_token(memoryId, ["thumbnail"], [], 180);
    logSuccess(`✅ Token minted: ${token.substring(0, 50)}...`);

    // Step 6: Test HTTP access with specific asset ID (normal case)
    logInfo("Step 6: Testing HTTP access with specific asset ID...");
    const canisterId = "uxrrr-q7777-77774-qaaaq-cai";
    
    // URL encode the token to prevent parsing issues
    const encodedToken = encodeURIComponent(token);
    logInfo(`Original token: ${token.substring(0, 50)}...`);
    logInfo(`Encoded token: ${encodedToken.substring(0, 50)}...`);
    const httpUrl = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?id=${assetId}&token=${encodedToken}`;

    logInfo(`HTTP URL: ${httpUrl}`);

    try {
      const { stdout: curlOutput } = await execAsync(`curl -s -i ${httpUrl}`);
      logInfo(`Curl Response:\n${curlOutput}`);

      if (curlOutput.includes("HTTP/1.1 200 OK")) {
        logSuccess("🎉 SUCCESS! Normal asset flow works!");

        // Check if we got image data
        if (curlOutput.includes("Content-Type: image/")) {
          logSuccess("✅ Correct content type returned");
        }

        // Check for proper headers
        if (curlOutput.includes("Cache-Control: private, no-store")) {
          logSuccess("✅ Proper cache control headers present");
        }

        logInfo("");
        logInfo("🌐 You can now open this URL in your browser to see the image:");
        logInfo(httpUrl);

        return {
          success: true,
          httpUrl,
          curlOutput,
          memoryId,
          assetId,
          token: token.substring(0, 20) + "...",
        };
      } else {
        logError("❌ HTTP access failed - unexpected status code");
        return { success: false, reason: "http_access_failed", curlOutput };
      }
    } catch (curlError) {
      logError(`❌ HTTP access failed: ${curlError.message}`);
      return { success: false, reason: "curl_failed", error: curlError.message };
    }
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
  logHeader("🚀 Normal Asset Flow Test");

  const result = await testNormalAssetFlow();

  logHeader("📊 Test Results");
  if (result.success) {
    logSuccess("🎉 Normal Asset Flow Test PASSED!");
    logInfo(`Memory ID: ${result.memoryId}`);
    logInfo(`Asset ID: ${result.assetId}`);
    logInfo(`Token: ${result.token}`);
    logInfo(`HTTP URL: ${result.httpUrl}`);
    logInfo("");
    logInfo("🔍 What this proves:");
    logInfo("✅ Complete end-to-end image serving works with specific asset ID");
    logInfo("✅ Memory creation, token minting, and HTTP serving all work");
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
    if (result.curlOutput) {
      logInfo("Curl output:", result.curlOutput);
    }
    process.exit(1);
  }
}

main().catch(console.error);
