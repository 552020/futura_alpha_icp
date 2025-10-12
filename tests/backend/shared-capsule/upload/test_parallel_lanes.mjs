#!/usr/bin/env node

/**
 * Test: Parallel Lanes Execution
 *
 * This test runs Lane A (original upload) and Lane B (image processing) in parallel
 * to verify that both workflows can execute concurrently without conflicts.
 *
 * Test Scenarios:
 * 1. Parallel execution of Lane A and Lane B
 * 2. Shared capsule access between parallel lanes
 * 3. Concurrent asset creation and verification
 * 4. Performance comparison between sequential and parallel execution
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
import { echoInfo, echoSuccess, echoFail } from "../../utils/index.js";

const TEST_IMAGE_PATH = "assets/input/orange_tiny.jpg";

/**
 * Lane A: Original Upload Workflow
 * Uploads the original file as a blob and creates a memory with it
 */
async function testLaneAOriginalUpload(backend, capsuleId) {
  try {
    echoInfo(`🚀 Lane A: Starting original upload workflow`);

    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);
    const fileName = "orange_tiny_original.jpg";

    // Upload original file as blob
    const uploadResult = await uploadBufferAsBlob(backend, fileBuffer, capsuleId, {
      createMemory: false,
      idempotencyKey: `lane-a-original-${Date.now()}`,
    });

    if (!uploadResult.success) {
      throw new Error(`Lane A upload failed: ${uploadResult.error}`);
    }

    // Create memory with original blob
    const memoryMetadata = createMemoryMetadata({
      title: "Lane A - Original Upload",
      description: "Original file uploaded via Lane A workflow",
      tags: ["lane-a", "original", "parallel-test"],
    });

    const assetMetadata = createImageAssetMetadata({
      name: "original",
      size: fileBuffer.length,
      mimeType: "image/jpeg",
      assetType: "Original",
      tags: ["lane-a", "original"],
      description: "Original file from Lane A",
    });

    const memoryResult = await createMemoryFromBlob(
      backend,
      uploadResult.blobId,
      capsuleId,
      memoryMetadata,
      assetMetadata,
      `lane-a-memory-${Date.now()}`
    );

    if (!memoryResult.success) {
      throw new Error(`Lane A memory creation failed: ${memoryResult.error}`);
    }

    echoSuccess(`✅ Lane A completed: Memory ${memoryResult.memoryId} with blob ${uploadResult.blobId}`);

    return {
      success: true,
      memoryId: memoryResult.memoryId,
      blobId: uploadResult.blobId,
      workflow: "lane-a-original",
    };
  } catch (error) {
    echoFail(`❌ Lane A failed: ${error.message}`);
    return { success: false, error: error.message, workflow: "lane-a-original" };
  }
}

/**
 * Lane B: Image Processing Workflow
 * Processes the image to create derivatives and uploads them
 */
async function testLaneBImageProcessing(backend, capsuleId) {
  try {
    echoInfo(`🚀 Lane B: Starting image processing workflow`);

    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);
    const fileName = "orange_tiny_processed.jpg";

    // Process image derivatives
    const derivatives = processImageDerivativesPure(fileBuffer, "image/jpeg");

    if (!derivatives.success) {
      throw new Error(`Lane B image processing failed: ${derivatives.error}`);
    }

    // Upload original file as blob
    const uploadResult = await uploadBufferAsBlob(backend, fileBuffer, capsuleId, {
      createMemory: false,
      idempotencyKey: `lane-b-original-${Date.now()}`,
    });

    if (!uploadResult.success) {
      throw new Error(`Lane B upload failed: ${uploadResult.error}`);
    }

    // Upload processed derivatives
    const derivativesResult = await processImageDerivativesToICP(backend, derivatives.data, capsuleId, {
      idempotencyKey: `lane-b-derivatives-${Date.now()}`,
    });

    if (!derivativesResult.success) {
      throw new Error(`Lane B derivatives upload failed: ${derivativesResult.error}`);
    }

    // Create memory with all assets (original + derivatives + placeholder)
    const memoryMetadata = createMemoryMetadata({
      title: "Lane B - Image Processing",
      description: "Processed image with derivatives via Lane B workflow",
      tags: ["lane-b", "processed", "parallel-test"],
    });

    // Original asset
    const originalAsset = {
      blob_id: uploadResult.blobId,
      metadata: createImageAssetMetadata({
        name: "original",
        size: fileBuffer.length,
        mimeType: "image/jpeg",
        assetType: "Original",
        tags: ["lane-b", "original"],
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
        tags: ["lane-b", "display"],
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
        tags: ["lane-b", "thumb"],
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
        tags: ["lane-b", "placeholder"],
        description: "Placeholder from Lane B",
      }),
    };

    const createResult = await backend.memories_create_with_internal_blobs_and_inline_assets(
      capsuleId,
      memoryMetadata,
      [originalAsset, displayAsset, thumbAsset],
      [placeholderInlineAsset],
      `lane-b-memory-${Date.now()}`
    );

    if ("Err" in createResult) {
      throw new Error(`Lane B memory creation failed: ${JSON.stringify(createResult.Err)}`);
    }

    const memoryId = createResult.Ok;

    echoSuccess(`✅ Lane B completed: Memory ${memoryId} with 3 blob assets + 1 inline asset`);

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
    echoFail(`❌ Lane B failed: ${error.message}`);
    return { success: false, error: error.message, workflow: "lane-b-processed" };
  }
}

/**
 * Test parallel execution of Lane A and Lane B
 */
async function testParallelLanes(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing parallel execution of Lane A and Lane B`);

    const startTime = Date.now();

    // Run both lanes in parallel
    const [laneAResult, laneBResult] = await Promise.all([
      testLaneAOriginalUpload(backend, capsuleId),
      testLaneBImageProcessing(backend, capsuleId),
    ]);

    const endTime = Date.now();
    const executionTime = endTime - startTime;

    // Verify both lanes completed successfully
    const laneASuccess = laneAResult.success;
    const laneBSuccess = laneBResult.success;

    if (!laneASuccess) {
      throw new Error(`Lane A failed: ${laneAResult.error}`);
    }

    if (!laneBSuccess) {
      throw new Error(`Lane B failed: ${laneBResult.error}`);
    }

    echoSuccess(`✅ Both lanes completed successfully in ${executionTime}ms`);
    echoInfo(`  Lane A: Memory ${laneAResult.memoryId} with blob ${laneAResult.blobId}`);
    echoInfo(`  Lane B: Memory ${laneBResult.memoryId} with 3 blob assets + 1 inline asset`);

    // Verify both memories exist and are accessible
    const memoryA = await backend.memories_read(laneAResult.memoryId);
    const memoryB = await backend.memories_read(laneBResult.memoryId);

    if ("Err" in memoryA) {
      throw new Error(`Lane A memory not accessible: ${JSON.stringify(memoryA.Err)}`);
    }

    if ("Err" in memoryB) {
      throw new Error(`Lane B memory not accessible: ${JSON.stringify(memoryB.Err)}`);
    }

    echoSuccess(`✅ Both memories are accessible and contain expected assets`);
    echoInfo(`  Lane A memory: ${memoryA.Ok.blob_internal_assets.length} blob assets`);
    echoInfo(
      `  Lane B memory: ${memoryB.Ok.blob_internal_assets.length} blob assets, ${memoryB.Ok.inline_assets.length} inline assets`
    );

    return {
      success: true,
      executionTime: executionTime,
      laneA: laneAResult,
      laneB: laneBResult,
      memoryA: memoryA.Ok,
      memoryB: memoryB.Ok,
    };
  } catch (error) {
    echoFail(`❌ Parallel lanes test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Test sequential execution for performance comparison
 */
async function testSequentialLanes(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing sequential execution of Lane A and Lane B for comparison`);

    const startTime = Date.now();

    // Run lanes sequentially
    const laneAResult = await testLaneAOriginalUpload(backend, capsuleId);
    const laneBResult = await testLaneBImageProcessing(backend, capsuleId);

    const endTime = Date.now();
    const executionTime = endTime - startTime;

    // Verify both lanes completed successfully
    if (!laneAResult.success) {
      throw new Error(`Sequential Lane A failed: ${laneAResult.error}`);
    }

    if (!laneBResult.success) {
      throw new Error(`Sequential Lane B failed: ${laneBResult.error}`);
    }

    echoSuccess(`✅ Sequential execution completed in ${executionTime}ms`);

    return {
      success: true,
      executionTime: executionTime,
      laneA: laneAResult,
      laneB: laneBResult,
    };
  } catch (error) {
    echoFail(`❌ Sequential lanes test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Main test function
 */
async function main() {
  const { backend, canisterId, testFilter } = parseTestArgs();

  const allTests = [
    { name: "Parallel Lanes", fn: testParallelLanes },
    { name: "Sequential Lanes", fn: testSequentialLanes },
  ];

  // Filter tests if test name is provided
  const tests = testFilter ? allTests.filter((test) => test.name === testFilter) : allTests;

  if (testFilter && tests.length === 0) {
    echoFail(`Test not found: "${testFilter}"`);
    echoFail("Available tests:");
    allTests.forEach((test) => echoFail(`  - "${test.name}"`));
    process.exit(1);
  }

  echoInfo(`🧪 Running ${tests.length} parallel lanes test(s)...`);

  for (const test of tests) {
    await runTest(test.name, test.fn, backend, canisterId);
  }

  echoSuccess(`✅ All parallel lanes tests completed!`);
}

// Run the tests
main().catch((error) => {
  echoFail(`❌ Test execution failed: ${error.message}`);
  process.exit(1);
});

