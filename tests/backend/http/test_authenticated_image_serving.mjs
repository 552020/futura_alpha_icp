import { createTestActor, getOrCreateTestCapsule } from "../utils/index.js";
import { logHeader, logSuccess, logError, logInfo } from "../utils/helpers/logging.js";
import { createMemoryWithInline } from "../utils/helpers/memory-creation.js";
import { join } from "path";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function testAuthenticatedImageServing() {
  logHeader("🔐 Testing Authenticated Image Serving");
  
  try {
    // Get test actor and capsule
    const actor = await createTestActor();
    const capsuleId = await getOrCreateTestCapsule(actor);
    
    logInfo("Step 1: Creating memory with image asset...");
    
    // Create memory with image
    const testImagePath = join(process.cwd(), "../shared-capsule/upload/assets/input/orange_small_inline.jpg");
    const result = await createMemoryWithInline(actor, testImagePath, capsuleId, {
      assetType: "image",
      mimeType: "image/jpeg",
      idempotencyKey: `auth_test_${Date.now()}`,
    });
    
    if (!result.success) {
      throw new Error(`Failed to create memory: ${result.error}`);
    }
    
    const memoryId = result.memoryId;
    logSuccess(`✅ Memory created: ${memoryId}`);
    
    // Step 2: Mint HTTP token (this should work since we own the memory)
    logInfo("Step 2: Minting HTTP token...");
    try {
      const token = await actor.mint_http_token(
        memoryId,
        ["thumbnail"],
        null, // no specific asset IDs
        180   // 3 minutes TTL
      );
      
      logSuccess(`✅ Token minted successfully!`);
      logInfo(`Token: ${token.substring(0, 50)}...`);
      
      // Step 3: Test the image serving with the token
      logInfo("Step 3: Testing image serving with token...");
      
      const canisterId = "uxrrr-q7777-77774-qaaaq-cai";
      const imageUrl = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?token=${token}`;
      
      logInfo(`Testing URL: ${imageUrl}`);
      
      // Test with curl to get detailed response
      const { stdout, stderr } = await execAsync(`curl -s -i "${imageUrl}"`);
      
      if (stderr) {
        logError(`Curl error: ${stderr}`);
      }
      
      const lines = stdout.trim().split('\n');
      const statusLine = lines.find(line => line.startsWith('HTTP/'));
      const body = lines[lines.length - 1];
      
      if (statusLine && statusLine.includes('200')) {
        logSuccess("🎉 SUCCESS! Image serving with token works!");
        logInfo(`Status: ${statusLine}`);
        logInfo(`Body size: ${body.length} bytes`);
        
        // Check if it's actually image data
        if (body.startsWith('\xff\xd8\xff')) {
          logSuccess("✅ Response contains valid JPEG data!");
        } else {
          logInfo(`Response preview: ${body.substring(0, 100)}...`);
        }
        
        // Show the browser URL
        logInfo("");
        logSuccess("🌐 Copy this URL to your browser to see the image:");
        logInfo(imageUrl);
        
      } else {
        logError(`❌ Image serving failed`);
        logInfo(`Status: ${statusLine || 'Unknown'}`);
        logInfo(`Response: ${body}`);
      }
      
    } catch (tokenError) {
      logError(`❌ Token minting failed: ${tokenError.message}`);
      logInfo("This suggests ACL permissions are not properly configured.");
      logInfo("The memory was created, but we can't mint tokens for it.");
    }
    
  } catch (error) {
    logError(`❌ Test failed: ${error.message}`);
  }
}

// Run the test
testAuthenticatedImageServing().catch(console.error);





