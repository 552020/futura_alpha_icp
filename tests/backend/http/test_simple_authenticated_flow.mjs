import { logHeader, logSuccess, logError, logInfo } from "../utils/helpers/logging.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function testSimpleAuthenticatedFlow() {
  logHeader("🔐 Testing Simple Authenticated Image Flow");
  
  try {
    // Step 1: Create a capsule (this should work since we're authenticated)
    logInfo("Step 1: Creating test capsule...");
    
    const createCapsuleCmd = `dfx canister call backend capsules_create '("test_capsule_http_auth", "Test capsule for HTTP auth")' --output raw`;
    
    const { stdout: capsuleResult } = await execAsync(createCapsuleCmd);
    const capsuleId = capsuleResult.trim().replace(/"/g, '');
    
    logSuccess(`✅ Test capsule created: ${capsuleId}`);
    
    // Step 2: Create a memory with a simple test image
    logInfo("Step 2: Creating memory with test image...");
    
    // Create a simple 1x1 PNG image (68 bytes)
    const pngData = "\\x89\\x50\\x4e\\x47\\x0d\\x0a\\x1a\\x0a\\x00\\x00\\x00\\x0d\\x49\\x48\\x44\\x52\\x00\\x00\\x00\\x01\\x00\\x00\\x00\\x01\\x08\\x02\\x00\\x00\\x00\\x90\\x77\\x53\\xde\\x00\\x00\\x00\\x0c\\x49\\x44\\x41\\x54\\x08\\x99\\x01\\x01\\x00\\x00\\x00\\xff\\xff\\x00\\x00\\x00\\x02\\x00\\x01\\x00\\x00\\x00\\x00\\x49\\x45\\x4e\\x44\\xae\\x42\\x60\\x82";
    const hashData = "\\x2c\\xf2\\x4d\\xba\\x4f\\x8a\\x6c\\xba\\x1f\\x86\\xb8\\xe7\\xfe\\x74\\xfa\\x8d\\x80\\x31\\x24\\xca\\x06\\x62\\xea\\x4a\\x06\\x97\\x3f\\x8a\\x3e\\x4c\\x06\\x97";
    
    const createMemoryCmd = `dfx canister call backend memories_create '(
      "${capsuleId}",
      vec { blob "${pngData}" },
      vec {},
      vec {},
      vec {},
      vec {},
      vec {},
      vec { blob "${hashData}" },
      vec { ("name", "test_image.png"); ("mime_type", "image/png"); ("bytes", "68"); ("width", "1"); ("height", "1") },
      "test_memory_${Date.now()}"
    )' --output raw`;
    
    const { stdout: memoryResult } = await execAsync(createMemoryCmd);
    const memoryId = memoryResult.trim().replace(/"/g, '');
    
    logSuccess(`✅ Memory created: ${memoryId}`);
    
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
        if (body.includes('PNG') || body.includes('\x89PNG')) {
          logSuccess("✅ Response contains valid PNG data!");
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
testSimpleAuthenticatedFlow().then(success => {
  if (success) {
    logSuccess("🎉 Simple authenticated image serving test completed successfully!");
  } else {
    logError("❌ Simple authenticated image serving test failed");
  }
}).catch(console.error);


