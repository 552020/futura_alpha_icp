import { logHeader, logSuccess, logError, logInfo } from "../utils/helpers/logging.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function testSimpleHttpAuthFlow() {
  logHeader("🔐 Testing Simple HTTP Authentication Flow");
  
  try {
    // Step 1: Create a capsule using dfx canister call (this works)
    logInfo("Step 1: Creating test capsule...");
    const { stdout: capsuleResult } = await execAsync(`dfx canister call backend capsules_create '(null)' --output raw`);
    const capsuleId = capsuleResult.trim().replace(/"/g, '');
    logSuccess(`✅ Test capsule created: ${capsuleId}`);
    
    // Step 2: Create a memory using the existing test image file
    logInfo("Step 2: Creating memory with test image...");
    
    // Use the existing test image from assets folder
    const path = await import('path');
    const testImagePath = path.join(process.cwd(), "../shared-capsule/upload/assets/input/orange_small_inline.jpg");
    
    // Read the image file and convert to hex for Candid
    const fs = await import('fs');
    const imageBuffer = fs.readFileSync(testImagePath);
    const imageHex = imageBuffer.toString('hex');
    
    // Create a simple hash for the image (this is just for the API, not cryptographically secure)
    const crypto = await import('crypto');
    const hash = crypto.createHash('sha256').update(imageBuffer).digest('hex');
    
    // Create memory using dfx canister call with hex data
    const memoryCmd = `dfx canister call backend memories_create '(
      "${capsuleId}",
      vec { blob "${imageHex}" },
      vec {},
      vec {},
      vec {},
      vec {},
      vec {},
      vec { blob "${hash}" },
      vec { ("name", "test_image.jpg"); ("mime_type", "image/jpeg"); ("bytes", "${imageBuffer.length}"); ("width", "100"); ("height", "100") },
      "test_memory_${Date.now()}"
    )' --output raw`;
    
    const { stdout: memoryResult } = await execAsync(memoryCmd);
    const memoryId = memoryResult.trim().replace(/"/g, '');
    logSuccess(`✅ Test memory created: ${memoryId}`);
    
    // Step 3: Try to mint a token
    logInfo("Step 3: Minting HTTP token...");
    const mintTokenCmd = `dfx canister call backend mint_http_token '("${memoryId}", vec { "thumbnail" }, null, 180)' --output raw`;
    
    try {
      const { stdout: tokenResult } = await execAsync(mintTokenCmd);
      const token = tokenResult.trim().replace(/"/g, '');
      
      logSuccess(`✅ Token minted successfully!`);
      logInfo(`Token: ${token.substring(0, 50)}...`);
      
      // Step 4: Test the image serving with the token
      logInfo("Step 4: Testing image serving with token...");
      
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
testSimpleHttpAuthFlow().then(success => {
  if (success) {
    logSuccess("🎉 Simple HTTP authentication flow test completed successfully!");
  } else {
    logError("❌ Simple HTTP authentication flow test failed");
  }
}).catch(console.error);


