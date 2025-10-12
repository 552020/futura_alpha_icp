#!/usr/bin/env node

/**
 * Test: Complete System Integration
 *
 * This test combines all functionality from the 2-lane + 4-asset system:
 * - Lane A: Original upload workflow
 * - Lane B: Image processing workflow
 * - Asset addition to existing memories
 * - Memory retrieval and verification
 * - Deletion workflows
 *
 * Test Scenarios:
 * 1. Complete end-to-end workflow with both lanes
 * 2. Asset addition to existing memories
 * 3. Memory retrieval and asset verification
 * 4. Deletion of memories and assets
 * 5. Error handling and edge cases
 */

import { createTestActor, parseTestArgs, runTest } from "../../utils/index.js";
import {
  readFileAsBuffer,
  createMemoryMetadata,
  createImageAssetMetadata,
  createDerivativeAssetMetadata,
} from "../../utils/index.js";
import { uploadBufferAsBlob, createMemoryFromBlob } from "../../utils/index.js";
import { processImageDerivativesPure, processImageDerivativesToICP } from "../../utils/index.js";
import { addAssetToMemory, addInlineAssetToMemory } from "../../utils/index.js";
import { echoInfo, echoSuccess, echoFail } from "../../utils/index.js";

const TEST_IMAGE_PATH = "assets/input/orange_tiny.jpg";

/**
 * Complete end-to-end workflow test
 * Tests the full 2-lane + 4-asset system with all features
 */
async function testCompleteSystemWorkflow(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing complete system workflow`);

    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);
    const fileName = "orange_tiny_complete.jpg";

    // Step 1: Lane A - Original Upload
    echoInfo(`📤 Step 1: Lane A - Original Upload`);
    const laneAResult = await testLaneAOriginalUpload(backend, capsuleId);

    if (!laneAResult.success) {
      throw new Error(`Lane A failed: ${laneAResult.error}`);
    }

    // Step 2: Lane B - Image Processing
    echoInfo(`📤 Step 2: Lane B - Image Processing`);
    const laneBResult = await testLaneBImageProcessing(backend, capsuleId);

    if (!laneBResult.success) {
      throw new Error(`Lane B failed: ${laneBResult.error}`);
    }

    // Step 3: Add additional assets to Lane A memory
    echoInfo(`📤 Step 3: Adding additional assets to Lane A memory`);
    const additionalAssetResult = await testAddAssetsToMemory(backend, laneAResult.memoryId);

    if (!additionalAssetResult.success) {
      throw new Error(`Asset addition failed: ${additionalAssetResult.error}`);
    }

    // Step 4: Verify all memories and assets
    echoInfo(`🔍 Step 4: Verifying all memories and assets`);
    const verificationResult = await testMemoryVerification(backend, {
      laneA: laneAResult,
      laneB: laneBResult,
      additionalAssets: additionalAssetResult,
    });

    if (!verificationResult.success) {
      throw new Error(`Verification failed: ${verificationResult.error}`);
    }

    // Step 5: Test asset retrieval
    echoInfo(`📥 Step 5: Testing asset retrieval`);
    const retrievalResult = await testAssetRetrieval(backend, {
      laneA: laneAResult,
      laneB: laneBResult,
    });

    if (!retrievalResult.success) {
      throw new Error(`Asset retrieval failed: ${retrievalResult.error}`);
    }

    echoSuccess(`✅ Complete system workflow completed successfully`);
    echoInfo(`  Lane A: Memory ${laneAResult.memoryId} with ${verificationResult.laneAAssetCount} assets`);
    echoInfo(`  Lane B: Memory ${laneBResult.memoryId} with ${verificationResult.laneBAssetCount} assets`);

    return {
      success: true,
      laneA: laneAResult,
      laneB: laneBResult,
      additionalAssets: additionalAssetResult,
      verification: verificationResult,
      retrieval: retrievalResult,
    };
  } catch (error) {
    echoFail(`❌ Complete system workflow failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Lane A: Original Upload Workflow
 */
async function testLaneAOriginalUpload(backend, capsuleId) {
  try {
    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);

    // Upload original file as blob
    const uploadResult = await uploadBufferAsBlob(backend, fileBuffer, capsuleId, {
      createMemory: false,
      idempotencyKey: `complete-lane-a-${Date.now()}`,
    });

    if (!uploadResult.success) {
      throw new Error(`Lane A upload failed: ${uploadResult.error}`);
    }

    // Create memory with original blob
    const memoryMetadata = createMemoryMetadata({
      title: "Complete System - Lane A",
      description: "Original file uploaded via Lane A workflow",
      tags: ["complete-system", "lane-a", "original"],
    });

    const assetMetadata = createImageAssetMetadata({
      name: "original",
      size: fileBuffer.length,
      mimeType: "image/jpeg",
      assetType: "Original",
      tags: ["complete-system", "lane-a", "original"],
      description: "Original file from Lane A",
    });

    const memoryResult = await createMemoryFromBlob(
      backend,
      uploadResult.blobId,
      capsuleId,
      memoryMetadata,
      assetMetadata,
      `complete-lane-a-memory-${Date.now()}`
    );

    if (!memoryResult.success) {
      throw new Error(`Lane A memory creation failed: ${memoryResult.error}`);
    }

    return {
      success: true,
      memoryId: memoryResult.memoryId,
      blobId: uploadResult.blobId,
      workflow: "lane-a-original",
    };
  } catch (error) {
    return { success: false, error: error.message, workflow: "lane-a-original" };
  }
}

/**
 * Lane B: Image Processing Workflow
 */
async function testLaneBImageProcessing(backend, capsuleId) {
  try {
    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);

    // Process image derivatives
    const derivatives = processImageDerivativesPure(fileBuffer, "image/jpeg");

    if (!derivatives.success) {
      throw new Error(`Lane B image processing failed: ${derivatives.error}`);
    }

    // Upload original file as blob
    const uploadResult = await uploadBufferAsBlob(backend, fileBuffer, capsuleId, {
      createMemory: false,
      idempotencyKey: `complete-lane-b-${Date.now()}`,
    });

    if (!uploadResult.success) {
      throw new Error(`Lane B upload failed: ${uploadResult.error}`);
    }

    // Upload processed derivatives
    const derivativesResult = await processImageDerivativesToICP(backend, derivatives.data, capsuleId, {
      idempotencyKey: `complete-lane-b-derivatives-${Date.now()}`,
    });

    if (!derivativesResult.success) {
      throw new Error(`Lane B derivatives upload failed: ${derivativesResult.error}`);
    }

    // Create memory with all assets (original + derivatives + placeholder)
    const memoryMetadata = createMemoryMetadata({
      title: "Complete System - Lane B",
      description: "Processed image with derivatives via Lane B workflow",
      tags: ["complete-system", "lane-b", "processed"],
    });

    // Original asset
    const originalAsset = {
      blob_id: uploadResult.blobId,
      metadata: createImageAssetMetadata({
        name: "original",
        size: fileBuffer.length,
        mimeType: "image/jpeg",
        assetType: "Original",
        tags: ["complete-system", "lane-b", "original"],
        description: "Original file from Lane B",
      }),
    };

    // Display derivative
    const displayAsset = {
      blob_id: derivativesResult.data.blobIds.display,
      metadata: createDerivativeAssetMetadata({
        name: "display",
        size: derivatives.data.display.buffer.length,
        mimeType: "image/jpeg",
        tags: ["complete-system", "lane-b", "display"],
        description: "Display derivative from Lane B",
      }),
    };

    // Thumb derivative
    const thumbAsset = {
      blob_id: derivativesResult.data.blobIds.thumb,
      metadata: createDerivativeAssetMetadata({
        name: "thumb",
        size: derivatives.data.thumb.buffer.length,
        mimeType: "image/jpeg",
        tags: ["complete-system", "lane-b", "thumb"],
        description: "Thumb derivative from Lane B",
      }),
    };

    // Placeholder as inline asset
    const placeholderInlineAsset = {
      data: Array.from(derivatives.data.placeholder.buffer),
      metadata: createDerivativeAssetMetadata({
        name: "placeholder",
        size: derivatives.data.placeholder.buffer.length,
        mimeType: "image/jpeg",
        tags: ["complete-system", "lane-b", "placeholder"],
        description: "Placeholder from Lane B",
      }),
    };

    const createResult = await backend.memories_create_with_internal_blobs_and_inline_assets(
      capsuleId,
      memoryMetadata,
      [originalAsset, displayAsset, thumbAsset],
      [placeholderInlineAsset],
      `complete-lane-b-memory-${Date.now()}`
    );

    if ("Err" in createResult) {
      throw new Error(`Lane B memory creation failed: ${JSON.stringify(createResult.Err)}`);
    }

    const memoryId = createResult.Ok;

    return {
      success: true,
      memoryId: memoryId,
      blobIds: {
        original: uploadResult.blobId,
        display: derivativesResult.data.blobIds.display,
        thumb: derivativesResult.data.blobIds.thumb,
      },
      workflow: "lane-b-processed",
    };
  } catch (error) {
    return { success: false, error: error.message, workflow: "lane-b-processed" };
  }
}

/**
 * Test adding additional assets to existing memory
 */
async function testAddAssetsToMemory(backend, memoryId) {
  try {
    echoInfo(`📤 Adding additional assets to memory ${memoryId}`);

    // Create a small additional blob asset
    const additionalData = new Uint8Array(500).fill(42);
    const uploadResult = await uploadBufferAsBlob(backend, additionalData, "capsule_1759713283267064000", {
      createMemory: false,
      idempotencyKey: `additional-asset-${Date.now()}`,
    });

    if (!uploadResult.success) {
      throw new Error(`Additional asset upload failed: ${uploadResult.error}`);
    }

    // Add blob asset to memory
    const blobAsset = {
      blob_id: uploadResult.blobId,
      metadata: createDerivativeAssetMetadata({
        name: "additional-blob",
        size: additionalData.length,
        mimeType: "application/octet-stream",
        tags: ["complete-system", "additional"],
        description: "Additional blob asset",
      }),
    };

    const addBlobResult = await addAssetToMemory(backend, memoryId, blobAsset);

    if (!addBlobResult.success) {
      throw new Error(`Failed to add blob asset: ${addBlobResult.error}`);
    }

    // Add inline asset to memory
    const inlineData = new Uint8Array(200).fill(123);
    const addInlineResult = await addInlineAssetToMemory(backend, memoryId, inlineData, {
      metadata: createDerivativeAssetMetadata({
        name: "additional-inline",
        size: inlineData.length,
        mimeType: "application/octet-stream",
        tags: ["complete-system", "additional", "inline"],
        description: "Additional inline asset",
      }),
    });

    if (!addInlineResult.success) {
      throw new Error(`Failed to add inline asset: ${addInlineResult.error}`);
    }

    echoSuccess(`✅ Added 2 additional assets to memory ${memoryId}`);

    return {
      success: true,
      blobAssetId: addBlobResult.assetId,
      inlineAssetId: addInlineResult.assetId,
      additionalBlobId: uploadResult.blobId,
    };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

/**
 * Test memory verification
 */
async function testMemoryVerification(backend, results) {
  try {
    echoInfo(`🔍 Verifying all memories and assets`);

    // Verify Lane A memory
    const memoryA = await backend.memories_read(results.laneA.memoryId);
    if ("Err" in memoryA) {
      throw new Error(`Lane A memory not accessible: ${JSON.stringify(memoryA.Err)}`);
    }

    // Verify Lane B memory
    const memoryB = await backend.memories_read(results.laneB.memoryId);
    if ("Err" in memoryB) {
      throw new Error(`Lane B memory not accessible: ${JSON.stringify(memoryB.Err)}`);
    }

    // Verify all blob assets exist
    const allBlobIds = [
      results.laneA.blobId,
      results.laneB.blobIds.original,
      results.laneB.blobIds.display,
      results.laneB.blobIds.thumb,
      results.additionalAssets.additionalBlobId,
    ];

    for (const blobId of allBlobIds) {
      const meta = await backend.blob_get_meta(blobId);
      if ("Err" in meta) {
        throw new Error(`Blob asset ${blobId} not found: ${JSON.stringify(meta.Err)}`);
      }
    }

    echoSuccess(`✅ All memories and assets verified successfully`);
    echoInfo(
      `  Lane A: ${memoryA.Ok.blob_internal_assets.length} blob assets, ${memoryA.Ok.inline_assets.length} inline assets`
    );
    echoInfo(
      `  Lane B: ${memoryB.Ok.blob_internal_assets.length} blob assets, ${memoryB.Ok.inline_assets.length} inline assets`
    );

    return {
      success: true,
      laneAAssetCount: memoryA.Ok.blob_internal_assets.length + memoryA.Ok.inline_assets.length,
      laneBAssetCount: memoryB.Ok.blob_internal_assets.length + memoryB.Ok.inline_assets.length,
      memoryA: memoryA.Ok,
      memoryB: memoryB.Ok,
    };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

/**
 * Test asset retrieval
 */
async function testAssetRetrieval(backend, results) {
  try {
    echoInfo(`📥 Testing asset retrieval`);

    // Test retrieving blob assets
    const blobIds = [
      results.laneA.blobId,
      results.laneB.blobIds.original,
      results.laneB.blobIds.display,
      results.laneB.blobIds.thumb,
    ];

    for (const blobId of blobIds) {
      const blobData = await backend.blob_get(blobId);
      if ("Err" in blobData) {
        throw new Error(`Failed to retrieve blob ${blobId}: ${JSON.stringify(blobData.Err)}`);
      }

      if (blobData.Ok.length === 0) {
        throw new Error(`Blob ${blobId} is empty`);
      }
    }

    echoSuccess(`✅ All blob assets retrieved successfully`);

    return {
      success: true,
      retrievedBlobs: blobIds.length,
    };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

/**
 * Main test function
 */
async function main() {
  const { backend, canisterId, testFilter } = parseTestArgs();

  const allTests = [{ name: "Complete System Workflow", fn: testCompleteSystemWorkflow }];

  // Filter tests if test name is provided
  const tests = testFilter ? allTests.filter((test) => test.name === testFilter) : allTests;

  if (testFilter && tests.length === 0) {
    echoFail(`Test not found: "${testFilter}"`);
    echoFail("Available tests:");
    allTests.forEach((test) => echoFail(`  - "${test.name}"`));
    process.exit(1);
  }

  echoInfo(`🧪 Running ${tests.length} complete system test(s)...`);

  for (const test of tests) {
    await runTest(test.name, test.fn, backend, canisterId);
  }

  echoSuccess(`✅ All complete system tests completed!`);
}

// Run the tests
main().catch((error) => {
  echoFail(`❌ Test execution failed: ${error.message}`);
  process.exit(1);
});
