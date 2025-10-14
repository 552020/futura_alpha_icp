import { createTestActor, getCurrentPrincipal } from "../utils/index.js";
import { logHeader, logSuccess, logError, logInfo } from "../utils/helpers/logging.js";
import { createMemoryWithInline } from "../utils/helpers/memory-creation.js";
import { join } from "path";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function testAuthenticatedWithUtils() {
  logHeader("🔐 Testing Authenticated Image Serving with Utils");
  
  try {
    // Get current identity info
    const principal = getCurrentPrincipal();
    logInfo(`Current principal: ${principal}`);
    
    // Create test actor
    const actor = await createTestActor();
    logSuccess("✅ Test actor created");
    
    // Create a test capsule first (we need this for memory creation)
    logInfo("Creating test capsule...");
    const capsuleResult = await actor.capsules_create("test_capsule_http_auth", "Test capsule for HTTP auth");
    
    if (!("Ok" in capsuleResult)) {
      throw new Error(`Failed to create capsule: ${JSON.stringify(capsuleResult)}`);
    }
    
    const capsuleId = capsuleResult.Ok;
    logSuccess(`✅ Test capsule created: ${capsuleId}`);
    
    // Create memory with image
    logInfo("Creating memory with image asset...");
    const testImagePath = join(process.cwd(), "../shared-capsule/upload/assets/input/orange_small_inline.jpg");
    
    const memoryResult = await createMemoryWithInline(actor, testImagePath, capsuleId, {
      assetType: "image",
      mimeType: "image/jpeg",
      idempotencyKey: `auth_test_${Date.now()}`,
    });
    
    if (!memoryResult.success) {
      throw new Error(`Failed to create memory: ${memoryResult.error}`);
    }
    
    const memoryId = memoryResult.memoryId;
    logSuccess(`✅ Memory created: ${memoryId}`);
    
    // Now try to mint a token (this should work since we own the memory)
    logInfo("Minting HTTP token...");
    try {
      const token = await actor.mint_http_token(
        memoryId,
        ["thumbnail"],
        null, // no specific asset IDs
        180   // 3 minutes TTL
      );
      
      logSuccess(`✅ Token minted successfully!`);
      logInfo(`Token: ${token.substring(0, 50)}...`);
      
      // Test the image serving with the token
      logInfo("Testing image serving with token...");
      
      const canisterId = "uxrrr-q7777-77774-qaaaq-cai";
      const imageUrl = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?token=${token}`;
      
      logInfo(`Testing URL: ${imageUrl}`);
      
      // Test with curl
      const { stdout: curlResult } = await execAsync(`curl -s -i "${imageUrl}"`);
      
      const lines = curlResult.trim().split('\n');
      const statusLine = lines.find(line => line.startsWith('HTTP/'));
      const body = lines[lines.length - 1];
      
      if (statusLine && statusLine.includes('200')) {
        logSuccess("🎉 SUCCESS! Image serving with token works!");
        logInfo(`Status: ${statusLine}`);
        logInfo(`Body size: ${body.length} bytes`);
        
        // Check if it's actually image data
        if (body.includes('\xff\xd8\xff') || body.includes('PNG')) {
          logSuccess("✅ Response contains valid image data!");
        }
        
        // Show the browser URL
        logInfo("");
        logSuccess("🌐 Copy this URL to your browser to see the image:");
        logInfo(imageUrl);
        
        return true;
        
      } else {
        logError(`❌ Image serving failed`);
        logInfo(`Status: ${statusLine || 'Unknown'}`);
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
  }
}

// Run the test
testAuthenticatedWithUtils().then(success => {
  if (success) {
    logSuccess("🎉 Authenticated image serving test completed successfully!");
  } else {
    logError("❌ Authenticated image serving test failed");
  }
}).catch(console.error);





