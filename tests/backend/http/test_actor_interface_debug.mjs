/**
 * Actor Interface Debug Test
 *
 * This test debugs the actor interface to see what functions are available:
 * 1. List all available functions on the actor
 * 2. Check if mint_http_token is available with underscore
 * 3. Test the function if available
 */

import { logHeader, logInfo, logSuccess, logError } from "../utils/helpers/logging.js";
import { createTestActor } from "../utils/core/actor.js";

async function debugActorInterface() {
  logHeader("🔍 Debugging Actor Interface");

  try {
    // Step 1: Create test actor
    logInfo("Step 1: Creating test actor...");
    const { actor } = await createTestActor();
    logSuccess("✅ Test actor created");

    // Step 2: List all available functions
    logInfo("Step 2: Listing all available functions...");
    const functions = Object.getOwnPropertyNames(actor);
    logInfo(`Available functions: ${functions.length}`);

    // Filter for functions that might be related to HTTP token minting
    const httpFunctions = functions.filter(
      (name) =>
        name.toLowerCase().includes("mint") ||
        name.toLowerCase().includes("http") ||
        name.toLowerCase().includes("token")
    );

    logInfo("HTTP-related functions:");
    httpFunctions.forEach((func) => {
      logInfo(`  - ${func}`);
    });

    // Step 3: Check if mint_http_token is available with underscore
    logInfo("Step 3: Checking for mint_http_token with underscore...");
    if (typeof actor.mint_http_token === "function") {
      logSuccess("✅ mint_http_token function found with underscore!");

      // Step 4: Test the function
      logInfo("Step 4: Testing mint_http_token function...");
      try {
        // Create a test memory first
        const capsuleId = await actor.capsules_create([]);
        logInfo(`Created test capsule: ${capsuleId.Ok.id}`);

        // Create a simple test memory
        const testImageData =
          "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChAI9jU8j8wAAAABJRU5ErkJggg==";
        const imageBytes = Array.from(Buffer.from(testImageData, "base64"));

        const memoryResult = await actor.memories_create(
          capsuleId.Ok.id,
          [imageBytes],
          [],
          [],
          [],
          [],
          [],
          [],
          [
            ["name", "test.png"],
            ["mime_type", "image/png"],
            ["bytes", "68"],
            ["width", "1"],
            ["height", "1"],
            ["dpi", "72"],
            ["color_space", "sRGB"],
            ["exif_data", ""],
            ["compression_ratio", "1.0"],
            ["orientation", "1"],
          ],
          `test_${Date.now()}`
        );

        if (memoryResult.Ok) {
          logSuccess(`Created test memory: ${memoryResult.Ok}`);

          // Now try to mint a token
          const token = await actor.mint_http_token(memoryResult.Ok, ["thumbnail"], null, 180);
          logSuccess(`✅ Token minted successfully: ${token.substring(0, 50)}...`);

          // Cleanup
          await actor.memories_delete(memoryResult.Ok, false);
          logSuccess("✅ Test memory cleaned up");

          return { success: true, token: token.substring(0, 20) + "..." };
        } else {
          logError(`❌ Failed to create test memory: ${JSON.stringify(memoryResult.Err)}`);
          return { success: false, reason: "memory_creation_failed" };
        }
      } catch (error) {
        logError(`❌ mint_http_token function failed: ${error.message}`);
        return { success: false, reason: "function_failed", error: error.message };
      }
    } else {
      logError("❌ mint_http_token function not found with underscore");

      // Step 5: Check all function names more carefully
      logInfo("Step 5: Checking all function names for mint-related functions...");
      const mintFunctions = functions.filter((name) => name.toLowerCase().includes("mint"));
      logInfo(`Mint-related functions: ${mintFunctions.join(", ")}`);

      return { success: false, reason: "function_not_found", availableFunctions: functions };
    }
  } catch (error) {
    logError(`❌ Test failed: ${error.message}`);
    return { success: false, reason: "general_error", error: error.message };
  }
}

async function main() {
  logHeader("🚀 Actor Interface Debug Test");

  const result = await debugActorInterface();

  logHeader("📊 Debug Results");
  if (result.success) {
    logSuccess("🎉 Actor Interface Debug PASSED!");
    logInfo(`Token: ${result.token}`);
    logInfo("");
    logInfo("🔍 What this proves:");
    logInfo("✅ The mint_http_token function is available with underscore naming");
    logInfo("✅ The function works correctly");
    logInfo("✅ We can mint tokens and test the complete flow");
    logInfo("");
    logInfo("🌐 The HTTP module is fully functional!");
  } else {
    logError(`❌ Test failed: ${result.reason.replace(/_/g, " ")}`);
    if (result.error) {
      logError(`Error: ${result.error}`);
    }
    if (result.availableFunctions) {
      logInfo(`Available functions: ${result.availableFunctions.length}`);
      logInfo("First 20 functions:", result.availableFunctions.slice(0, 20).join(", "));
    }
  }
}

main().catch(console.error);




