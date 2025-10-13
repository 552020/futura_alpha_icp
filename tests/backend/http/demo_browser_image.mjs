import { createTestActor, getOrCreateTestCapsule } from "../utils/index.js";
import { logHeader, logSuccess, logError, logInfo } from "../utils/helpers/logging.js";
import { createMemoryWithInline } from "../utils/helpers/memory-creation.js";
import { join } from "path";

async function demoBrowserImage() {
  logHeader("🖼️  Browser Image Demo");

  try {
    // Get test actor and capsule
    const actor = await createTestActor();
    const capsuleId = await getOrCreateTestCapsule(actor);

    logInfo("Creating memory with image asset...");

    // Create memory with image
    const testImagePath = join(process.cwd(), "../shared-capsule/upload/assets/input/orange_small_inline.jpg");
    const result = await createMemoryWithInline(actor, testImagePath, capsuleId, {
      assetType: "image",
      mimeType: "image/jpeg",
      idempotencyKey: `browser_demo_${Date.now()}`,
    });

    if (!result.success) {
      throw new Error(`Failed to create memory: ${result.error}`);
    }

    const memoryId = result.memoryId;
    logSuccess(`✅ Memory created: ${memoryId}`);

    // Try to mint a token
    logInfo("Attempting to mint HTTP token...");
    try {
      const token = await actor.mint_http_token(
        memoryId,
        ["thumbnail"],
        null, // no specific asset IDs
        180 // 3 minutes TTL
      );

      logSuccess(`✅ Token minted successfully!`);

      // Generate the browser URL
      const canisterId = "uxrrr-q7777-77774-qaaaq-cai";
      const browserUrl = `http://${canisterId}.localhost:4943/assets/${memoryId}/thumbnail?token=${token}`;

      logSuccess("🎉 SUCCESS! Copy this URL to your browser:");
      logInfo(`📋 ${browserUrl}`);
      logInfo("");
      logInfo("This URL should display the orange image directly in your browser!");
      logInfo("The image is served via HTTP from the ICP canister with token authentication.");
    } catch (tokenError) {
      logError(`❌ Token minting failed: ${tokenError.message}`);
      logInfo("This is expected if you don't have permission to access this memory.");
      logInfo("The memory was created successfully, but ACL permissions prevent token minting.");

      // Still show what the URL would look like
      const canisterId = "uxrrr-q7777-77774-qaaaq-cai";
      const exampleToken = "example_token_here";
      const exampleUrl = `http://${canisterId}.localhost:4943/assets/${memoryId}/thumbnail?token=${exampleToken}`;

      logInfo("");
      logInfo("📋 Example URL format (with proper token):");
      logInfo(exampleUrl);
    }
  } catch (error) {
    logError(`❌ Demo failed: ${error.message}`);
  }
}

// Run the demo
demoBrowserImage().catch(console.error);


