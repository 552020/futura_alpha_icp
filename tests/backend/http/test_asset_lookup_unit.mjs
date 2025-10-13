/**
 * Asset Lookup Unit Test
 *
 * This test focuses on the unit-level testing of asset lookup functionality:
 * 1. Create a memory with an image asset
 * 2. Test direct memory access and asset ID extraction
 * 3. Verify ACL context consistency
 * 4. Test asset store functionality directly
 */

import { logHeader, logInfo, logSuccess, logError } from "../utils/helpers/logging.js";
import { createTestActor } from "../utils/core/actor.js";
import { createTestCapsule } from "../utils/helpers/capsule-creation.js";
import { createTestImageMemory } from "../utils/helpers/memory-creation.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function testAssetLookupUnit() {
  logHeader("🧪 Testing Asset Lookup Unit Functionality");

  let capsuleId = null;
  let memoryId = null;
  let assetId = null;

  try {
    // Step 1: Create test actor
    logInfo("Step 1: Creating test actor...");
    const { actor } = await createTestActor();
    logSuccess("✅ Test actor created");

    // Step 2: Get actor identity
    logInfo("Step 2: Getting actor identity...");
    const actorIdentity = await actor.whoami();
    logSuccess(`✅ Actor identity: ${actorIdentity}`);

    // Step 3: Create capsule
    logInfo("Step 3: Creating capsule...");
    capsuleId = await createTestCapsule(actor);
    logSuccess(`✅ Capsule created: ${capsuleId}`);

    // Step 4: Create memory
    logInfo("Step 4: Creating memory...");
    memoryId = await createTestImageMemory(actor, capsuleId, {
      name: "unit_test_image.png",
      mimeType: "image/png",
      idempotencyKey: `unit_test_${Date.now()}`,
    });
    logSuccess(`✅ Memory created: ${memoryId}`);

    // Step 5: Test direct memory access
    logInfo("Step 5: Testing direct memory access...");
    const memoryResult = await actor.memories_read(memoryId);
    if (memoryResult.Ok) {
      logSuccess("✅ Memory read successful");
      logInfo(`Memory ID: ${memoryResult.Ok.id}`);
      logInfo(`Capsule ID: ${memoryResult.Ok.capsule_id}`);
      logInfo(`Inline assets count: ${memoryResult.Ok.inline_assets?.length || 0}`);

      if (memoryResult.Ok.inline_assets && memoryResult.Ok.inline_assets.length > 0) {
        assetId = memoryResult.Ok.inline_assets[0].asset_id;
        logSuccess(`✅ Asset ID extracted: ${assetId}`);
        logInfo(`Asset content type: ${memoryResult.Ok.inline_assets[0].content_type || "undefined"}`);
        logInfo(`Asset bytes length: ${memoryResult.Ok.inline_assets[0].bytes.length}`);
      } else {
        logError("❌ No inline assets found");
        throw new Error("No inline assets found");
      }
    } else {
      logError(`❌ Memory read failed: ${JSON.stringify(memoryResult.Err)}`);
      throw new Error("Memory read failed");
    }

    // Step 6: Test ACL context consistency
    logInfo("Step 6: Testing ACL context consistency...");

    // Test 1: Can we read the memory directly?
    logInfo("Test 6.1: Direct memory read...");
    const directRead = await actor.memories_read(memoryId);
    if (directRead.Ok) {
      logSuccess("✅ Direct memory read works");
    } else {
      logError(`❌ Direct memory read failed: ${JSON.stringify(directRead.Err)}`);
    }

    // Test 2: Can we access the asset directly?
    logInfo("Test 6.2: Direct asset access...");
    try {
      const assetResult = await actor.asset_get_by_id(memoryId, assetId);
      if (assetResult.Ok) {
        logSuccess("✅ Direct asset access works");
        logInfo(`Asset type: ${assetResult.Ok.Inline ? "Inline" : "Other"}`);
        if (assetResult.Ok.Inline) {
          logInfo(`Asset bytes length: ${assetResult.Ok.Inline.bytes.length}`);
          logInfo(`Asset content type: ${assetResult.Ok.Inline.content_type}`);
        }
      } else {
        logError(`❌ Direct asset access failed: ${JSON.stringify(assetResult.Err)}`);
      }
    } catch (error) {
      logError(`❌ Direct asset access error: ${error.message}`);
    }

    // Step 7: Test token minting with the same context
    logInfo("Step 7: Testing token minting...");
    try {
      const token = await actor.mint_http_token(memoryId, ["thumbnail"], [], 180);
      logSuccess(`✅ Token minted successfully: ${token.substring(0, 50)}...`);

      // Decode the token to check its contents
      try {
        const tokenPayload = JSON.parse(atob(token.split(".")[0] + "=="));
        logInfo(`Token payload: ${JSON.stringify(tokenPayload, null, 2)}`);

        if (tokenPayload.p && tokenPayload.p.sub) {
          logInfo(`Token subject: ${tokenPayload.p.sub}`);
          if (tokenPayload.p.sub === actorIdentity) {
            logSuccess("✅ Token subject matches actor identity");
          } else {
            logError(`❌ Token subject mismatch: ${tokenPayload.p.sub} vs ${actorIdentity}`);
          }
        }
      } catch (decodeError) {
        logError(`❌ Token decode failed: ${decodeError.message}`);
      }
    } catch (tokenError) {
      logError(`❌ Token minting failed: ${tokenError.message}`);
    }

    // Step 8: Test capsule access
    logInfo("Step 8: Testing capsule access...");
    try {
      const capsuleResult = await actor.capsules_read_basic(capsuleId);
      if (capsuleResult.Ok) {
        logSuccess("✅ Capsule access works");
        logInfo(`Capsule ID: ${capsuleResult.Ok.id}`);
        logInfo(`Capsule name: ${capsuleResult.Ok.name}`);
      } else {
        logError(`❌ Capsule access failed: ${JSON.stringify(capsuleResult.Err)}`);
      }
    } catch (capsuleError) {
      logError(`❌ Capsule access error: ${capsuleError.message}`);
    }

    return {
      success: true,
      memoryId,
      assetId,
      capsuleId,
      actorIdentity,
      analysis: "Unit tests completed - check logs for detailed results",
    };
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
  logHeader("🚀 Asset Lookup Unit Test");

  const result = await testAssetLookupUnit();

  logHeader("📊 Unit Test Results");
  if (result.success) {
    logSuccess("✅ Asset Lookup Unit Test COMPLETED");
    logInfo(`Memory ID: ${result.memoryId}`);
    logInfo(`Asset ID: ${result.assetId}`);
    logInfo(`Capsule ID: ${result.capsuleId}`);
    logInfo(`Actor Identity: ${result.actorIdentity}`);
    logInfo(`Analysis: ${result.analysis}`);
    logInfo("");
    logInfo("🔍 This unit test helps us understand:");
    logInfo("✅ Whether memory creation and access work correctly");
    logInfo("✅ Whether asset IDs can be extracted properly");
    logInfo("✅ Whether ACL context is consistent");
    logInfo("✅ Whether the issue is in the HTTP layer or the core logic");
    logInfo("");
    logInfo("💡 Next steps:");
    logInfo("1. If all unit tests pass, the issue is in the HTTP layer");
    logInfo("2. If unit tests fail, we need to fix the core logic first");
    logInfo("3. This helps us isolate the problem and fix it efficiently");
  } else {
    logError(`❌ Test failed: ${result.reason.replace(/_/g, " ")}`);
    if (result.error) {
      logError(`Error: ${result.error}`);
    }
  }
}

main().catch(console.error);
