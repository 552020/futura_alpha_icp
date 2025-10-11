#!/usr/bin/env node

import {
  createTestActor,
  parseTestArgs,
  createTestActorOptions,
  logNetworkConfig,
  getOrCreateTestCapsuleForUpload,
  createTestRunner,
} from "../../utils/index.js";

import {
  uploadFileAsBlob,
  createMemoryFromBlob,
  addAssetToMemory,
  verifyMemoryIntegrity,
  processImageDerivativesPure,
  processImageDerivativesToICP,
  readFileAsBuffer,
} from "../../utils/index.js";

// Parse command line arguments using shared utility
const parsedArgs = parseTestArgs("test_lane_b_image_processing.mjs", "Tests Lane B: Image processing workflows with shared utilities");

// Import the backend interface
import { idlFactory } from "../../declarations/backend/backend.did.js";

// Test configuration
const TEST_NAME = "Lane B: Image Processing Tests";
const TEST_IMAGE_PATH = "assets/input/orange_tiny.jpg";

/**
 * Test Lane B: Image Processing Workflow
 * 
 * This test focuses on the image processing lane of the 2-lane + 4-asset system:
 * 1. Upload original image as blob
 * 2. Create memory with original image
 * 3. Process image derivatives (display, thumb, placeholder)
 * 4. Upload derivatives as separate blobs
 * 5. Add derivatives to existing memory using new backend functions
 * 6. Verify memory has 4 assets total
 */
async function testLaneBImageProcessing(backend, capsuleId) {
  console.log("🧪 Testing Lane B: Image processing workflow...");

  try {
    // Step 1: Upload original image as blob
    console.log("📤 Step 1: Uploading original image...");
    const originalUploadResult = await uploadFileAsBlob(backend, TEST_IMAGE_PATH, capsuleId, {
      chunkSize: 65536,
      idempotencyKey: `lane-b-original-${Date.now()}`,
    });

    if (!originalUploadResult.success) {
      return { success: false, error: `Original upload failed: ${originalUploadResult.error}` };
    }

    const { blobId: originalBlobId } = originalUploadResult;
    console.log(`✅ Original image uploaded: ${originalBlobId}`);

    // Step 2: Create memory with original image
    console.log("📝 Step 2: Creating memory with original image...");
    const fileBuffer = await readFileAsBuffer(TEST_IMAGE_PATH);
    const fileName = "orange_tiny.jpg";
    const fileSize = fileBuffer.length;

    const memoryResult = await createMemoryFromBlob(backend, capsuleId, fileName, fileSize, originalBlobId, originalUploadResult, {
      assetType: "image",
      mimeType: "image/jpeg",
      memoryType: { Image: null },
    });

    if (!memoryResult.success) {
      return { success: false, error: `Memory creation failed: ${memoryResult.error}` };
    }

    const memoryId = memoryResult.memoryId;
    console.log(`✅ Memory created: ${memoryId}`);

    // Step 3: Process image derivatives
    console.log("🖼️ Step 3: Processing image derivatives...");
    const derivatives = await processImageDerivativesPure(fileBuffer, "image/jpeg");

    console.log(`✅ Generated ${Object.keys(derivatives).length} derivatives:`, Object.keys(derivatives));

    // Step 4: Upload derivatives to ICP
    console.log("📤 Step 4: Uploading derivatives to ICP...");
    const derivativeResults = await processImageDerivativesToICP(backend, derivatives, capsuleId, {
      idempotencyKey: `lane-b-derivatives-${Date.now()}`,
    });

    if (!derivativeResults.success) {
      return { success: false, error: `Derivatives upload failed: ${derivativeResults.error}` };
    }

    const { blobIds } = derivativeResults.data;
    console.log(`✅ Derivatives uploaded:`, Object.keys(blobIds));

    // Step 5: Add derivatives to existing memory
    console.log("➕ Step 5: Adding derivatives to existing memory...");
    const addResults = [];

    // Add display derivative
    if (blobIds.display) {
      const displayResult = await addAssetToMemory(backend, memoryId, blobIds.display, {
        assetType: "display",
      mimeType: "image/webp",
        idempotencyKey: `lane-b-display-${Date.now()}`,
      });
      addResults.push({ type: "display", result: displayResult });
    }

    // Add thumbnail derivative
    if (blobIds.thumb) {
      const thumbResult = await addAssetToMemory(backend, memoryId, blobIds.thumb, {
        assetType: "thumb",
      mimeType: "image/webp",
        idempotencyKey: `lane-b-thumb-${Date.now()}`,
      });
      addResults.push({ type: "thumb", result: thumbResult });
    }

    // Add placeholder derivative
    if (blobIds.placeholder) {
      const placeholderResult = await addAssetToMemory(backend, memoryId, blobIds.placeholder, {
        assetType: "placeholder",
      mimeType: "image/webp",
        idempotencyKey: `lane-b-placeholder-${Date.now()}`,
      });
      addResults.push({ type: "placeholder", result: placeholderResult });
    }

    // Check if all additions were successful
    const failedAdditions = addResults.filter(r => !r.result.success);
    if (failedAdditions.length > 0) {
      return { success: false, error: `Failed to add derivatives: ${failedAdditions.map(f => f.type).join(", ")}` };
    }

    console.log(`✅ All derivatives added to memory:`, addResults.map(r => r.type));

    // Step 6: Verify memory has 4 assets total (1 original + 3 derivatives)
    console.log("🔍 Step 6: Verifying memory integrity...");
    const verifyResult = await verifyMemoryIntegrity(backend, memoryId, 4);
    if (!verifyResult) {
      return { success: false, error: "Memory verification failed - expected 4 assets" };
    }

    console.log("✅ Lane B image processing workflow completed successfully");
    return { 
      success: true, 
      data: { 
        memoryId, 
        originalBlobId, 
        derivativeBlobIds: blobIds,
        assetCount: 4 
      } 
    };

  } catch (error) {
    console.error(`❌ Lane B test failed: ${error.message}`);
    return { success: false, error: `Lane B test failed: ${error.message}` };
  }
}

/**
 * Test image processing with different quality settings
 */
async function testImageProcessingQuality(backend, capsuleId) {
  console.log("🧪 Testing image processing with different quality settings...");

  try {
    // Upload original image
    const originalUploadResult = await uploadFileAsBlob(backend, TEST_IMAGE_PATH, capsuleId, {
      idempotencyKey: `quality-test-original-${Date.now()}`,
    });

    if (!originalUploadResult.success) {
      return { success: false, error: `Original upload failed: ${originalUploadResult.error}` };
    }

    const { blobId: originalBlobId } = originalUploadResult;

    // Create memory
    const fileBuffer = await readFileAsBuffer(TEST_IMAGE_PATH);
    const memoryResult = await createMemoryFromBlob(backend, capsuleId, "quality_test.jpg", fileBuffer.length, originalBlobId, originalUploadResult, {
      assetType: "image",
      mimeType: "image/jpeg",
      memoryType: { Image: null },
    });

    if (!memoryResult.success) {
      return { success: false, error: `Memory creation failed: ${memoryResult.error}` };
    }

    const memoryId = memoryResult.memoryId;

    // Test different quality settings
    const qualityTests = [
      { name: "high", quality: 95 },
      { name: "medium", quality: 75 },
      { name: "low", quality: 50 },
    ];

    const qualityResults = [];
    for (const test of qualityTests) {
      console.log(`🖼️ Testing ${test.name} quality (${test.quality}%)...`);
      
      const derivatives = await processImageDerivativesPure(fileBuffer, "image/jpeg");

      const derivativeResults = await processImageDerivativesToICP(backend, derivatives, capsuleId, {
        idempotencyKey: `quality-test-${test.name}-${Date.now()}`,
      });

      if (derivativeResults.success && derivativeResults.data.blobIds.display) {
        const addResult = await addAssetToMemory(backend, memoryId, derivativeResults.data.blobIds.display, {
          assetType: "display",
          mimeType: "image/webp",
          idempotencyKey: `quality-test-${test.name}-add-${Date.now()}`,
        });

        qualityResults.push({
          quality: test.name,
          success: addResult.success,
          assetId: addResult.assetId,
        });
      }
    }

    const successfulQualityTests = qualityResults.filter(r => r.success);
    console.log(`✅ Quality tests completed: ${successfulQualityTests.length}/${qualityTests.length} successful`);

    return { 
      success: true, 
      data: { 
        memoryId, 
        qualityResults: successfulQualityTests 
      } 
    };

  } catch (error) {
    console.error(`❌ Quality test failed: ${error.message}`);
    return { success: false, error: `Quality test failed: ${error.message}` };
  }
}

/**
 * Test image processing with different sizes
 */
async function testImageProcessingSizes(backend, capsuleId) {
  console.log("🧪 Testing image processing with different sizes...");

  try {
    // Upload original image
    const originalUploadResult = await uploadFileAsBlob(backend, TEST_IMAGE_PATH, capsuleId, {
      idempotencyKey: `size-test-original-${Date.now()}`,
    });

    if (!originalUploadResult.success) {
      return { success: false, error: `Original upload failed: ${originalUploadResult.error}` };
    }

    const { blobId: originalBlobId } = originalUploadResult;

    // Create memory
    const fileBuffer = await readFileAsBuffer(TEST_IMAGE_PATH);
    const memoryResult = await createMemoryFromBlob(backend, capsuleId, "size_test.jpg", fileBuffer.length, originalBlobId, originalUploadResult, {
      assetType: "image",
      mimeType: "image/jpeg",
      memoryType: { Image: null },
    });

    if (!memoryResult.success) {
      return { success: false, error: `Memory creation failed: ${memoryResult.error}` };
    }

    const memoryId = memoryResult.memoryId;

    // Test different sizes
    const sizeTests = [
      { name: "large", width: 1920, height: 1080 },
      { name: "medium", width: 800, height: 600 },
      { name: "small", width: 300, height: 200 },
      { name: "tiny", width: 100, height: 75 },
    ];

    const sizeResults = [];
    for (const test of sizeTests) {
      console.log(`🖼️ Testing ${test.name} size (${test.width}x${test.height})...`);
      
      const derivatives = await processImageDerivativesPure(fileBuffer, "image/jpeg");

      const derivativeResults = await processImageDerivativesToICP(backend, derivatives, capsuleId, {
        idempotencyKey: `size-test-${test.name}-${Date.now()}`,
      });

      if (derivativeResults.success && derivativeResults.data.blobIds.display) {
        const addResult = await addAssetToMemory(backend, memoryId, derivativeResults.data.blobIds.display, {
          assetType: "display",
          mimeType: "image/webp",
          idempotencyKey: `size-test-${test.name}-add-${Date.now()}`,
        });

        sizeResults.push({
          size: test.name,
          dimensions: `${test.width}x${test.height}`,
          success: addResult.success,
          assetId: addResult.assetId,
        });
      }
    }

    const successfulSizeTests = sizeResults.filter(r => r.success);
    console.log(`✅ Size tests completed: ${successfulSizeTests.length}/${sizeTests.length} successful`);

    return { 
      success: true, 
      data: { 
        memoryId, 
        sizeResults: successfulSizeTests 
      } 
    };

  } catch (error) {
    console.error(`❌ Size test failed: ${error.message}`);
    return { success: false, error: `Size test failed: ${error.message}` };
  }
}

/**
 * Main test function
 */
async function main() {
  console.log("=========================================");
  console.log(`Starting ${TEST_NAME}`);
  console.log("=========================================");
  console.log("");

  try {
    // Create test actor using shared utilities
    console.log("Loading DFX identity...");
    const options = createTestActorOptions(parsedArgs);
    const { actor: backend, canisterId: actualCanisterId } = await createTestActor(options);

    // Log network configuration using shared utility
    logNetworkConfig(parsedArgs, actualCanisterId);

    // Get or create a test capsule using shared utility
    const capsuleId = await getOrCreateTestCapsuleForUpload(backend);

    // Create test runner using shared utility
    const runner = createTestRunner(TEST_NAME);

    // Run all tests in order
    await runner.runTest("Lane B: Image processing workflow", testLaneBImageProcessing, backend, capsuleId);
    await runner.runTest("Image processing quality tests", testImageProcessingQuality, backend, capsuleId);
    await runner.runTest("Image processing size tests", testImageProcessingSizes, backend, capsuleId);

    // Print test summary using shared utility
    const allPassed = runner.printTestSummary();

    if (allPassed) {
      process.exit(0);
    } else {
      process.exit(1);
    }
  } catch (error) {
    console.error("❌ Test execution failed:", error.message);
    process.exit(1);
  }
}

// Run the tests
main().catch((error) => {
  console.error("❌ Unhandled error:", error);
  process.exit(1);
});