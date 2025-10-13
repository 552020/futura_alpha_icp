import { runCompleteHttpAuthFlow, cleanupTestResources } from "../utils/index.js";
import { logHeader, logSuccess, logError } from "../utils/helpers/logging.js";

async function testWithHttpAuthUtils() {
  logHeader("🔐 Testing HTTP Authentication with New Utils");
  
  try {
    // Run the complete HTTP authentication flow
    const result = await runCompleteHttpAuthFlow({
      capsuleName: "test_capsule_http_utils",
      capsuleDescription: "Test capsule using HTTP auth utils",
      memoryOptions: {
        name: "test_image_utils.png",
        mimeType: "image/png",
        width: 1,
        height: 1
      },
      variants: ["thumbnail"],
      ttlSecs: 300 // 5 minutes
    });
    
    if (result.success) {
      logSuccess("🎉 HTTP authentication test completed successfully!");
      logInfo(`Memory ID: ${result.memoryId}`);
      logInfo(`Token: ${result.token.substring(0, 50)}...`);
      logInfo(`Browser URL: ${result.browserUrl}`);
      
      // Test additional variants if needed
      logInfo("Testing additional variants...");
      
      // You can test other variants here
      // const previewResult = await testHttpAssetServing(result.memoryId, result.token, "preview");
      
      // Clean up
      logInfo("Cleaning up test resources...");
      await cleanupTestResources(result.memoryId);
      
      return true;
    } else {
      logError("❌ HTTP authentication test failed");
      logInfo(`Status: ${result.servingResult.status}`);
      logInfo(`Response: ${result.servingResult.body}`);
      return false;
    }
    
  } catch (error) {
    logError(`❌ Test failed: ${error.message}`);
    return false;
  }
}

// Run the test
testWithHttpAuthUtils().then(success => {
  if (success) {
    logSuccess("🎉 HTTP authentication utils test completed successfully!");
  } else {
    logError("❌ HTTP authentication utils test failed");
  }
}).catch(console.error);

