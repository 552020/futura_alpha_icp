import { logHeader, logSuccess, logError, logInfo } from "../utils/helpers/logging.js";
import {
  createTestCapsule,
  createTestMemoryWithImage,
  mintHttpToken,
  cleanupTestResources,
} from "../utils/helpers/http-auth.js";

async function testSimpleHttpAuthFlow() {
  logHeader("🔐 Testing Simple HTTP Authentication Flow");

  let capsuleId = null;
  let memoryId = null;
  let token = null;

  try {
    // Step 1: Create a capsule using existing utility
    logInfo("Step 1: Creating test capsule...");
    capsuleId = await createTestCapsule();
    logSuccess(`✅ Test capsule created: ${capsuleId}`);

    // Step 2: Create a memory using existing utility
    logInfo("Step 2: Creating memory with test image...");
    memoryId = await createTestMemoryWithImage(capsuleId, {
      name: "simple_auth_test.jpg",
      mimeType: "image/jpeg",
    });
    logSuccess(`✅ Test memory created: ${memoryId}`);

    // Step 3: Try to mint a token using existing utility
    logInfo("Step 3: Minting HTTP token...");
    try {
      token = await mintHttpToken(memoryId, ["thumbnail"], null, 180);
      logSuccess(`✅ Token minted successfully!`);
      logInfo(`Token: ${token.substring(0, 50)}...`);

      // Step 4: Test the image serving with the token
      logInfo("Step 4: Testing image serving with token...");

      const canisterId = "uxrrr-q7777-77774-qaaaq-cai";
      const imageUrl = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?token=${token}`;

      logInfo(`Testing URL: ${imageUrl}`);

      // Test with curl
      const { exec } = await import("child_process");
      const { promisify } = await import("util");
      const execAsync = promisify(exec);

      const { stdout: curlResult } = await execAsync(`curl -s -i "${imageUrl}"`);

      const lines = curlResult.trim().split("\n");
      const statusLine = lines.find((line) => line.startsWith("HTTP/"));
      const body = lines[lines.length - 1];

      if (statusLine && statusLine.includes("200")) {
        logSuccess("🎉 SUCCESS! Image serving with token works!");
        logInfo(`Status: ${statusLine}`);
        logInfo(`Body size: ${body.length} bytes`);

        // Check if it's actually image data
        if (body.includes("\xff\xd8\xff") || body.includes("PNG")) {
          logSuccess("✅ Response contains valid image data!");
        }

        // Show the browser URL
        logInfo("");
        logSuccess("🌐 Copy this URL to your browser to see the image:");
        logInfo(imageUrl);

        return true;
      } else {
        logError(`❌ Image serving failed`);
        logInfo(`Status: ${statusLine || "Unknown"}`);
        logInfo(`Response: ${body}`);
        return false;
      }
    } catch (tokenError) {
      logError(`❌ Token minting failed: ${tokenError.message}`);
      logInfo("This suggests ACL permissions are not properly configured.");
      logInfo("The memory was created, but we can't mint tokens for it.");

      // Still show what the URL would look like
      const canisterId = "uxrrr-q7777-77774-qaaaq-cai";
      const exampleToken = "example_token_here";
      const exampleUrl = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?token=${exampleToken}`;

      logInfo("");
      logInfo("📋 Example URL format (with proper token):");
      logInfo(exampleUrl);
      return false;
    }
  } catch (error) {
    logError(`❌ Test failed: ${error.message}`);
    return false;
  } finally {
    // Clean up test resources
    if (memoryId) {
      logInfo("Cleaning up memory...");
      try {
        await cleanupTestResources(memoryId);
        logSuccess("✅ Memory cleaned up");
      } catch (cleanupError) {
        logError(`❌ Cleanup failed: ${cleanupError.message}`);
      }
    }
  }
}

// Run the test
testSimpleHttpAuthFlow()
  .then((success) => {
    if (success) {
      logSuccess("🎉 Simple HTTP authentication flow test completed successfully!");
    } else {
      logError("❌ Simple HTTP authentication flow test failed");
    }
  })
  .catch(console.error);
