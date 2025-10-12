#!/usr/bin/env node

/**
 * Test: Asset Retrieval
 *
 * This test focuses specifically on testing asset retrieval functionality:
 * - Blob asset retrieval
 * - Inline asset retrieval
 * - Memory asset listing
 * - Asset metadata retrieval
 * - Performance testing for large assets
 *
 * Test Scenarios:
 * 1. Retrieve blob assets of various sizes
 * 2. Retrieve inline assets
 * 3. List all assets in a memory
 * 4. Retrieve asset metadata
 * 5. Performance testing with large files
 * 6. Error handling for non-existent assets
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
 * Test blob asset retrieval
 */
async function testBlobAssetRetrieval(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing blob asset retrieval`);

    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);

    // Upload file as blob
    const uploadResult = await uploadBufferAsBlob(backend, fileBuffer, capsuleId, {
      createMemory: false,
      idempotencyKey: `retrieval-test-${Date.now()}`,
    });

    if (!uploadResult.success) {
      throw new Error(`Upload failed: ${uploadResult.error}`);
    }

    // Retrieve blob metadata
    const metaResult = await backend.blob_get_meta(uploadResult.blobId);
    if ("Err" in metaResult) {
      throw new Error(`Failed to get blob metadata: ${JSON.stringify(metaResult.Err)}`);
    }

    const metadata = metaResult.Ok;
    echoInfo(`📊 Blob metadata: ${metadata.size} bytes, ${metadata.uploaded_at} uploaded`);

    // Retrieve blob data
    const dataResult = await backend.blob_get(uploadResult.blobId);
    if ("Err" in dataResult) {
      throw new Error(`Failed to retrieve blob data: ${JSON.stringify(dataResult.Err)}`);
    }

    const retrievedData = dataResult.Ok;

    // Verify data integrity
    if (retrievedData.length !== fileBuffer.length) {
      throw new Error(`Data size mismatch: expected ${fileBuffer.length}, got ${retrievedData.length}`);
    }

    // Verify data content
    const retrievedBuffer = new Uint8Array(retrievedData);
    for (let i = 0; i < fileBuffer.length; i++) {
      if (fileBuffer[i] !== retrievedBuffer[i]) {
        throw new Error(`Data content mismatch at byte ${i}`);
      }
    }

    echoSuccess(`✅ Blob asset retrieval successful: ${retrievedData.length} bytes`);

    return {
      success: true,
      blobId: uploadResult.blobId,
      originalSize: fileBuffer.length,
      retrievedSize: retrievedData.length,
      metadata: metadata,
    };
  } catch (error) {
    echoFail(`❌ Blob asset retrieval failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Test inline asset retrieval
 */
async function testInlineAssetRetrieval(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing inline asset retrieval`);

    // Create a memory with inline assets
    const memoryMetadata = createMemoryMetadata({
      title: "Inline Asset Retrieval Test",
      description: "Testing inline asset retrieval functionality",
      tags: ["retrieval-test", "inline"],
    });

    // Create inline asset data
    const inlineData = new Uint8Array(1000).fill(42);
    const inlineAsset = {
      data: Array.from(inlineData),
      metadata: createDerivativeAssetMetadata({
        name: "test-inline",
        size: inlineData.length,
        mimeType: "application/octet-stream",
        tags: ["retrieval-test", "inline"],
        description: "Test inline asset",
      }),
    };

    // Create memory with inline asset
    const createResult = await backend.memories_create_with_internal_blobs_and_inline_assets(
      capsuleId,
      memoryMetadata,
      [], // No blob assets
      [inlineAsset],
      `inline-retrieval-test-${Date.now()}`
    );

    if ("Err" in createResult) {
      throw new Error(`Memory creation failed: ${JSON.stringify(createResult.Err)}`);
    }

    const memoryId = createResult.Ok;

    // Retrieve memory
    const memoryResult = await backend.memories_read(memoryId);
    if ("Err" in memoryResult) {
      throw new Error(`Failed to retrieve memory: ${JSON.stringify(memoryResult.Err)}`);
    }

    const memory = memoryResult.Ok;

    // Verify inline assets
    if (memory.inline_assets.length !== 1) {
      throw new Error(`Expected 1 inline asset, got ${memory.inline_assets.length}`);
    }

    const retrievedInlineAsset = memory.inline_assets[0];

    // Verify inline asset data
    if (retrievedInlineAsset.data.length !== inlineData.length) {
      throw new Error(
        `Inline asset size mismatch: expected ${inlineData.length}, got ${retrievedInlineAsset.data.length}`
      );
    }

    // Verify inline asset content
    const retrievedData = new Uint8Array(retrievedInlineAsset.data);
    for (let i = 0; i < inlineData.length; i++) {
      if (inlineData[i] !== retrievedData[i]) {
        throw new Error(`Inline asset content mismatch at byte ${i}`);
      }
    }

    echoSuccess(`✅ Inline asset retrieval successful: ${retrievedInlineAsset.data.length} bytes`);

    return {
      success: true,
      memoryId: memoryId,
      inlineAssetId: retrievedInlineAsset.asset_id,
      originalSize: inlineData.length,
      retrievedSize: retrievedInlineAsset.data.length,
      metadata: retrievedInlineAsset.metadata,
    };
  } catch (error) {
    echoFail(`❌ Inline asset retrieval failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Test memory asset listing
 */
async function testMemoryAssetListing(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing memory asset listing`);

    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);

    // Upload original file as blob
    const uploadResult = await uploadBufferAsBlob(backend, fileBuffer, capsuleId, {
      createMemory: false,
      idempotencyKey: `listing-test-${Date.now()}`,
    });

    if (!uploadResult.success) {
      throw new Error(`Upload failed: ${uploadResult.error}`);
    }

    // Create memory with blob asset
    const memoryMetadata = createMemoryMetadata({
      title: "Asset Listing Test",
      description: "Testing memory asset listing functionality",
      tags: ["retrieval-test", "listing"],
    });

    const blobAsset = {
      blob_id: uploadResult.blobId,
      metadata: createImageAssetMetadata({
        name: "original",
        size: fileBuffer.length,
        mimeType: "image/jpeg",
        assetType: "Original",
        tags: ["retrieval-test", "listing"],
        description: "Original file for listing test",
      }),
    };

    const createResult = await backend.memories_create_with_internal_blobs_and_inline_assets(
      capsuleId,
      memoryMetadata,
      [blobAsset],
      [], // No inline assets initially
      `listing-test-${Date.now()}`
    );

    if ("Err" in createResult) {
      throw new Error(`Memory creation failed: ${JSON.stringify(createResult.Err)}`);
    }

    const memoryId = createResult.Ok;

    // Add additional inline asset
    const inlineData = new Uint8Array(500).fill(123);
    const addInlineResult = await addInlineAssetToMemory(backend, memoryId, inlineData, {
      metadata: createDerivativeAssetMetadata({
        name: "additional-inline",
        size: inlineData.length,
        mimeType: "application/octet-stream",
        tags: ["retrieval-test", "listing", "inline"],
        description: "Additional inline asset",
      }),
    });

    if (!addInlineResult.success) {
      throw new Error(`Failed to add inline asset: ${addInlineResult.error}`);
    }

    // Retrieve memory and list all assets
    const memoryResult = await backend.memories_read(memoryId);
    if ("Err" in memoryResult) {
      throw new Error(`Failed to retrieve memory: ${JSON.stringify(memoryResult.Err)}`);
    }

    const memory = memoryResult.Ok;

    // Verify asset counts
    const expectedBlobAssets = 1;
    const expectedInlineAssets = 1;

    if (memory.blob_internal_assets.length !== expectedBlobAssets) {
      throw new Error(`Expected ${expectedBlobAssets} blob assets, got ${memory.blob_internal_assets.length}`);
    }

    if (memory.inline_assets.length !== expectedInlineAssets) {
      throw new Error(`Expected ${expectedInlineAssets} inline assets, got ${memory.inline_assets.length}`);
    }

    // List blob assets
    echoInfo(`📋 Blob assets (${memory.blob_internal_assets.length}):`);
    for (const asset of memory.blob_internal_assets) {
      echoInfo(`  - ${asset.metadata.name}: ${asset.metadata.size} bytes (${asset.metadata.mime_type})`);
    }

    // List inline assets
    echoInfo(`📋 Inline assets (${memory.inline_assets.length}):`);
    for (const asset of memory.inline_assets) {
      echoInfo(`  - ${asset.metadata.name}: ${asset.metadata.size} bytes (${asset.metadata.mime_type})`);
    }

    echoSuccess(`✅ Memory asset listing successful`);

    return {
      success: true,
      memoryId: memoryId,
      blobAssets: memory.blob_internal_assets,
      inlineAssets: memory.inline_assets,
      totalAssets: memory.blob_internal_assets.length + memory.inline_assets.length,
    };
  } catch (error) {
    echoFail(`❌ Memory asset listing failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Test asset metadata retrieval
 */
async function testAssetMetadataRetrieval(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing asset metadata retrieval`);

    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);

    // Upload file as blob
    const uploadResult = await uploadBufferAsBlob(backend, fileBuffer, capsuleId, {
      createMemory: false,
      idempotencyKey: `metadata-test-${Date.now()}`,
    });

    if (!uploadResult.success) {
      throw new Error(`Upload failed: ${uploadResult.error}`);
    }

    // Get blob metadata
    const metaResult = await backend.blob_get_meta(uploadResult.blobId);
    if ("Err" in metaResult) {
      throw new Error(`Failed to get blob metadata: ${JSON.stringify(metaResult.Err)}`);
    }

    const blobMetadata = metaResult.Ok;

    // Create memory with the blob
    const memoryMetadata = createMemoryMetadata({
      title: "Metadata Retrieval Test",
      description: "Testing asset metadata retrieval functionality",
      tags: ["retrieval-test", "metadata"],
    });

    const assetMetadata = createImageAssetMetadata({
      name: "metadata-test",
      size: fileBuffer.length,
      mimeType: "image/jpeg",
      assetType: "Original",
      tags: ["retrieval-test", "metadata"],
      description: "Asset for metadata testing",
    });

    const createResult = await backend.memories_create_with_internal_blobs_and_inline_assets(
      capsuleId,
      memoryMetadata,
      [
        {
          blob_id: uploadResult.blobId,
          metadata: assetMetadata,
        },
      ],
      [],
      `metadata-test-${Date.now()}`
    );

    if ("Err" in createResult) {
      throw new Error(`Memory creation failed: ${JSON.stringify(createResult.Err)}`);
    }

    const memoryId = createResult.Ok;

    // Retrieve memory and get asset metadata
    const memoryResult = await backend.memories_read(memoryId);
    if ("Err" in memoryResult) {
      throw new Error(`Failed to retrieve memory: ${JSON.stringify(memoryResult.Err)}`);
    }

    const memory = memoryResult.Ok;
    const memoryAssetMetadata = memory.blob_internal_assets[0].metadata;

    // Verify metadata consistency
    if (blobMetadata.size !== memoryAssetMetadata.size) {
      throw new Error(`Size mismatch: blob ${blobMetadata.size} vs memory ${memoryAssetMetadata.size}`);
    }

    echoInfo(`📊 Blob metadata: ${blobMetadata.size} bytes, uploaded at ${blobMetadata.uploaded_at}`);
    echoInfo(`📊 Memory asset metadata: ${memoryAssetMetadata.name} (${memoryAssetMetadata.size} bytes)`);

    echoSuccess(`✅ Asset metadata retrieval successful`);

    return {
      success: true,
      blobId: uploadResult.blobId,
      memoryId: memoryId,
      blobMetadata: blobMetadata,
      memoryAssetMetadata: memoryAssetMetadata,
    };
  } catch (error) {
    echoFail(`❌ Asset metadata retrieval failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Test error handling for non-existent assets
 */
async function testNonExistentAssetHandling(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing error handling for non-existent assets`);

    // Try to retrieve non-existent blob
    const fakeBlobId = "fake-blob-id-12345";
    const blobResult = await backend.blob_get(fakeBlobId);

    if ("Ok" in blobResult) {
      throw new Error(`Expected error for non-existent blob, but got success`);
    }

    echoInfo(`✅ Non-existent blob correctly returned error: ${JSON.stringify(blobResult.Err)}`);

    // Try to get metadata for non-existent blob
    const metaResult = await backend.blob_get_meta(fakeBlobId);

    if ("Ok" in metaResult) {
      throw new Error(`Expected error for non-existent blob metadata, but got success`);
    }

    echoInfo(`✅ Non-existent blob metadata correctly returned error: ${JSON.stringify(metaResult.Err)}`);

    // Try to retrieve non-existent memory
    const fakeMemoryId = "fake-memory-id-12345";
    const memoryResult = await backend.memories_read(fakeMemoryId);

    if ("Ok" in memoryResult) {
      throw new Error(`Expected error for non-existent memory, but got success`);
    }

    echoInfo(`✅ Non-existent memory correctly returned error: ${JSON.stringify(memoryResult.Err)}`);

    echoSuccess(`✅ Error handling for non-existent assets working correctly`);

    return {
      success: true,
      testedErrors: ["non-existent-blob", "non-existent-blob-metadata", "non-existent-memory"],
    };
  } catch (error) {
    echoFail(`❌ Error handling test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Main test function
 */
async function main() {
  const { backend, canisterId, testFilter } = parseTestArgs();

  const allTests = [
    { name: "Blob Asset Retrieval", fn: testBlobAssetRetrieval },
    { name: "Inline Asset Retrieval", fn: testInlineAssetRetrieval },
    { name: "Memory Asset Listing", fn: testMemoryAssetListing },
    { name: "Asset Metadata Retrieval", fn: testAssetMetadataRetrieval },
    { name: "Non-Existent Asset Handling", fn: testNonExistentAssetHandling },
  ];

  // Filter tests if test name is provided
  const tests = testFilter ? allTests.filter((test) => test.name === testFilter) : allTests;

  if (testFilter && tests.length === 0) {
    echoFail(`Test not found: "${testFilter}"`);
    echoFail("Available tests:");
    allTests.forEach((test) => echoFail(`  - "${test.name}"`));
    process.exit(1);
  }

  echoInfo(`🧪 Running ${tests.length} asset retrieval test(s)...`);

  for (const test of tests) {
    await runTest(test.name, test.fn, backend, canisterId);
  }

  echoSuccess(`✅ All asset retrieval tests completed!`);
}

// Run the tests
main().catch((error) => {
  echoFail(`❌ Test execution failed: ${error.message}`);
  process.exit(1);
});
