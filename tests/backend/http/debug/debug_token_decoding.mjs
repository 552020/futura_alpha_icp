/**
 * Debug Token Decoding Test
 * 
 * This test helps us understand if the token decoding is working correctly
 */

import { logHeader, logInfo, logSuccess, logError } from "../utils/helpers/logging.js";
import { createTestActor } from "../utils/core/actor.js";
import { createTestCapsule } from "../utils/helpers/capsule-creation.js";
import { createTestImageMemory } from "../utils/helpers/memory-creation.js";

async function debugTokenDecoding() {
  logHeader("🔍 Debugging Token Decoding");

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
      name: "debug_token_test.png",
      mimeType: "image/png",
      idempotencyKey: `debug_token_${Date.now()}`,
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

    // Step 6: Test token decoding
    logInfo("Step 6: Testing token decoding...");
    
    try {
      // Decode the token to see its contents
      // The token is a single URL-safe base64 string without padding
      logInfo(`Full token: ${token}`);
      
      // Add padding to the base64 string
      let base64Token = token;
      while (base64Token.length % 4) {
        base64Token += '=';
      }
      
      // Convert URL-safe base64 to standard base64
      base64Token = base64Token.replace(/-/g, '+').replace(/_/g, '/');
      
      const tokenData = JSON.parse(atob(base64Token));
      logInfo(`Token data: ${JSON.stringify(tokenData, null, 2)}`);
      
      // Check if the token contains the expected memory ID
      if (tokenData.p && tokenData.p.scope && tokenData.p.scope.memory_id === memoryId) {
        logSuccess("✅ Token contains correct memory ID");
      } else {
        logError(`❌ Token memory ID mismatch: expected ${memoryId}, got ${tokenData.p?.scope?.memory_id}`);
      }
      
      // Check if the token contains the expected variants
      if (tokenData.p && tokenData.p.scope && tokenData.p.scope.variants && tokenData.p.scope.variants.includes("thumbnail")) {
        logSuccess("✅ Token contains correct variant");
      } else {
        logError(`❌ Token variant mismatch: expected ["thumbnail"], got ${JSON.stringify(tokenData.p?.scope?.variants)}`);
      }
      
      // Check if the token contains asset IDs
      if (tokenData.p && tokenData.p.scope && tokenData.p.scope.asset_ids === null) {
        logSuccess("✅ Token has no specific asset IDs (as expected)");
      } else {
        logError(`❌ Token asset IDs mismatch: expected null, got ${JSON.stringify(tokenData.p?.scope?.asset_ids)}`);
      }
      
    } catch (decodeError) {
      logError(`❌ Token decode failed: ${decodeError.message}`);
    }

    // Step 7: Test token with specific asset ID
    logInfo("Step 7: Testing token with specific asset ID...");
    try {
      const tokenWithAsset = await actor.mint_http_token(memoryId, ["thumbnail"], [assetId], 180);
      logSuccess(`✅ Token with asset ID minted: ${tokenWithAsset.substring(0, 50)}...`);
      
      // Decode the token with asset ID
      let base64AssetToken = tokenWithAsset;
      while (base64AssetToken.length % 4) {
        base64AssetToken += '=';
      }
      base64AssetToken = base64AssetToken.replace(/-/g, '+').replace(/_/g, '/');
      
      const tokenWithAssetPayload = JSON.parse(atob(base64AssetToken));
      logInfo(`Token with asset payload: ${JSON.stringify(tokenWithAssetPayload, null, 2)}`);
      
      if (tokenWithAssetPayload.p && tokenWithAssetPayload.p.scope && tokenWithAssetPayload.p.scope.asset_ids && tokenWithAssetPayload.p.scope.asset_ids.includes(assetId)) {
        logSuccess("✅ Token with asset ID contains correct asset ID");
      } else {
        logError(`❌ Token with asset ID mismatch: expected [${assetId}], got ${JSON.stringify(tokenWithAssetPayload.p?.scope?.asset_ids)}`);
      }
      
    } catch (error) {
      logError(`❌ Token with asset ID failed: ${error.message}`);
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
  logHeader("🚀 Token Decoding Debug Test");

  const result = await debugTokenDecoding();

  logHeader("📊 Debug Results");
  if (result.success) {
    logSuccess("✅ Token decoding debug completed");
    logInfo(`Memory ID: ${result.memoryId}`);
    logInfo(`Asset ID: ${result.assetId}`);
    logInfo(`Token: ${result.token.substring(0, 50)}...`);
    logInfo("");
    logInfo("🔍 This test helps us understand:");
    logInfo("✅ Whether tokens are being minted correctly");
    logInfo("✅ Whether token payloads contain the expected data");
    logInfo("✅ Whether the issue is in token creation or token parsing");
  } else {
    logError(`❌ Debug failed: ${result.error}`);
  }
}

main().catch(console.error);
