/**
 * Complete HTTP Flow Test
 *
 * This test demonstrates the complete end-to-end flow:
 * 1. Create a capsule with proper permissions
 * 2. Create a memory with an image asset
 * 3. Mint an HTTP token for the memory
 * 4. Access the asset via HTTP URL
 * 5. Validate the response
 */

import { logHeader, logInfo, logSuccess, logError } from "../utils/helpers/logging.js";
import { createTestActor } from "../utils/core/actor.js";
import { createTestCapsule } from "../utils/helpers/capsule-creation.js";
import { createTestImageMemory } from "../utils/helpers/memory-creation.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function testCompleteHttpFlow() {
  logHeader("🌐 Testing Complete HTTP Flow");

  let capsuleId = null;
  let memoryId = null;
  let token = null;
  let actor = null;

  try {
    // Step 1: Create test actor
    logInfo("Step 1: Creating test actor...");
    const actorResult = await createTestActor();
    actor = actorResult.actor;
    logSuccess("✅ Test actor created");

    // Step 2: Create capsule
    logInfo("Step 2: Creating capsule...");
    capsuleId = await createTestCapsule(actor);
    logSuccess(`✅ Capsule created: ${capsuleId}`);

    // Step 3: Create memory with image asset
    logInfo("Step 3: Creating memory with image asset...");
    memoryId = await createTestImageMemory(actor, capsuleId, {
      name: "complete_flow_test.png",
      mimeType: "image/png",
      idempotencyKey: `complete_flow_${Date.now()}`,
    });
    logSuccess(`✅ Memory created: ${memoryId}`);

    // Step 4: Verify we can read the memory (this confirms we have access)
    logInfo("Step 4: Verifying memory access...");
    try {
      const memoryResult = await actor.memories_read(memoryId);
      if (memoryResult.Ok) {
        logSuccess("✅ Memory read successful - we have proper access");
      } else {
        logError(`❌ Memory read failed: ${JSON.stringify(memoryResult.Err)}`);
        throw new Error("No access to created memory");
      }
    } catch (error) {
      logError(`❌ Memory read failed: ${error.message}`);
      throw error;
    }

    // Step 5: Mint HTTP token
    logInfo("Step 5: Minting HTTP token...");
    try {
      token = await actor.mint_http_token(memoryId, ["thumbnail"], [], 180);
      logSuccess(`✅ Token minted: ${token.substring(0, 50)}...`);
    } catch (error) {
      logError(`❌ Token minting failed: ${error.message}`);

      // This might be expected if ACL doesn't grant automatic permissions
      // Let's check what permissions we have
      logInfo("Checking memory permissions...");
      try {
        const memory = await actor.memories_read(memoryId);
        if (memory.Ok) {
          logInfo("Memory details:", JSON.stringify(memory.Ok, null, 2));
        }
      } catch (permError) {
        logError(`Permission check failed: ${permError.message}`);
      }

      throw error;
    }

    // Step 6: Test HTTP access
    logInfo("Step 6: Testing HTTP access...");
    const canisterId = "uxrrr-q7777-77774-qaaaq-cai";
    const httpUrl = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?token=${token}`;

    logInfo(`HTTP URL: ${httpUrl}`);

    try {
      const { stdout: curlOutput } = await execAsync(`curl -s -i ${httpUrl}`);
      logInfo(`Curl Response:\n${curlOutput}`);

      if (curlOutput.includes("HTTP/1.1 200 OK")) {
        logSuccess("✅ HTTP access successful! Complete flow works!");

        // Check if we got image data
        if (curlOutput.includes("Content-Type: image/")) {
          logSuccess("✅ Correct content type returned");
        }

        // Check for proper headers
        if (curlOutput.includes("Cache-Control: private, no-store")) {
          logSuccess("✅ Proper cache control headers present");
        }

        return {
          success: true,
          httpUrl,
          curlOutput,
          memoryId,
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
    if (memoryId && actor) {
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
  logHeader("🚀 Complete HTTP Flow Test");

  const result = await testCompleteHttpFlow();

  logHeader("📊 Test Results");
  if (result.success) {
    logSuccess("🎉 Complete HTTP flow test PASSED!");
    logInfo(`Memory ID: ${result.memoryId}`);
    logInfo(`Token: ${result.token}`);
    logInfo(`HTTP URL: ${result.httpUrl}`);
    logInfo("");
    logInfo("🔍 What this proves:");
    logInfo("✅ Memory creation with proper permissions");
    logInfo("✅ HTTP token minting with ACL validation");
    logInfo("✅ Asset serving via HTTP gateway");
    logInfo("✅ Complete end-to-end flow works");
  } else {
    logError(`❌ Test failed: ${result.reason.replace(/_/g, " ")}`);
    if (result.error) {
      logError(`Error: ${result.error}`);
    }
    if (result.curlOutput) {
      logInfo("Curl output:", result.curlOutput);
    }

    logInfo("");
    logInfo("🔍 Analysis:");
    if (result.reason === "general_error") {
      logInfo("- This could be a permission issue");
      logInfo("- The ACL system might require explicit permission setup");
      logInfo("- This is actually correct security behavior");
    }

    process.exit(1);
  }
}

main().catch(console.error);
