#!/usr/bin/env node

/**
 * Test: Deletion Workflows
 *
 * This test focuses specifically on testing deletion workflows:
 * - Memory deletion
 * - Blob asset deletion
 * - Inline asset deletion
 * - Cascade deletion behavior
 * - Error handling for deletion operations
 *
 * Test Scenarios:
 * 1. Delete memory with blob assets
 * 2. Delete memory with inline assets
 * 3. Delete memory with mixed assets
 * 4. Verify cascade deletion of assets
 * 5. Error handling for non-existent resources
 * 6. Performance testing with large numbers of assets
 */

import { createTestActor, parseTestArgs, runTest } from "../../utils/index.js";
import {
  readFileAsBuffer,
  createMemoryMetadata,
  createImageAssetMetadata,
  createDerivativeAssetMetadata,
  computeSHA256Hash,
} from "../../utils/index.js";
import { uploadBufferAsBlob, createMemoryFromBlob } from "../../utils/index.js";
import { processImageDerivativesPure, processImageDerivativesToICP } from "../../utils/index.js";
import { addAssetToMemory, addInlineAssetToMemory } from "../../utils/index.js";
import { verifyBlobIntegrity, verifyMemoryIntegrity, verifyBlobMetadata } from "../../utils/index.js";
import { createTestMemory } from "../../utils/index.js";
import { echoInfo, echoSuccess, echoFail } from "../../utils/index.js";

const TEST_IMAGE_PATH = "assets/input/orange_tiny.jpg";
const CHUNK_SIZE = 65536; // 64KB chunks

/**
 * Shared helper function to create a memory with blob asset for deletion testing
 * @param {Object} backend - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @param {string} testName - Test name for metadata
 * @param {Object} options - Additional options
 * @returns {Promise<{success: boolean, memoryId?: string, blobId?: string, error?: string}>}
 */
async function createTestMemoryWithBlob(backend, capsuleId, testName, options = {}) {
  try {
    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);

    // Upload file as blob
    const uploadResult = await uploadBufferAsBlob(backend, fileBuffer, capsuleId, {
      createMemory: false,
      idempotencyKey: `${testName}-${Date.now()}`,
    });

    if (!uploadResult.success) {
      throw new Error(`Upload failed: ${uploadResult.error}`);
    }

    // Create memory with blob asset
    const memoryMetadata = createMemoryMetadata({
      title: `${testName} Memory`,
      description: `Testing ${testName} with blob assets`,
      tags: ["deletion-test", testName.toLowerCase()],
    });

    const assetMetadata = createImageAssetMetadata({
      name: "original",
      size: fileBuffer.length,
      mimeType: "image/jpeg",
      assetType: "Original",
      tags: ["deletion-test", testName.toLowerCase()],
      description: `Original file for ${testName}`,
    });

    const memoryResult = await createMemoryFromBlob(
      backend,
      uploadResult.blobId,
      capsuleId,
      memoryMetadata,
      assetMetadata,
      `${testName.toLowerCase()}-test-${Date.now()}`
    );

    if (!memoryResult.success) {
      throw new Error(`Memory creation failed: ${memoryResult.error}`);
    }

    return {
      success: true,
      memoryId: memoryResult.memoryId,
      blobId: uploadResult.blobId,
      fileBuffer: fileBuffer,
    };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

/**
 * Shared helper function to verify memory and blob exist before deletion
 * @param {Object} backend - Backend actor
 * @param {string} memoryId - Memory ID
 * @param {string} blobId - Blob ID
 * @returns {Promise<{success: boolean, error?: string}>}
 */
async function verifyMemoryAndBlobExist(backend, memoryId, blobId) {
  try {
    // Verify memory exists
    const memoryBefore = await backend.memories_read(memoryId);
    if ("Err" in memoryBefore) {
      throw new Error(`Memory not found before deletion: ${JSON.stringify(memoryBefore.Err)}`);
    }

    // Verify blob exists
    const blobBefore = await backend.blob_get_meta(blobId);
    if ("Err" in blobBefore) {
      throw new Error(`Blob not found before deletion: ${JSON.stringify(blobBefore.Err)}`);
    }

    echoInfo(`✅ Memory and blob exist before deletion`);
    echoInfo(`  Memory: ${memoryId} with ${memoryBefore.Ok.blob_internal_assets.length} blob assets`);
    echoInfo(`  Blob: ${blobId} (${blobBefore.Ok.size} bytes)`);

    return { success: true };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

/**
 * Shared helper function to verify memory and blob are deleted after deletion
 * @param {Object} backend - Backend actor
 * @param {string} memoryId - Memory ID
 * @param {string} blobId - Blob ID
 * @param {boolean} expectBlobDeleted - Whether blob should be deleted (cascade deletion)
 * @returns {Promise<{success: boolean, error?: string}>}
 */
async function verifyMemoryAndBlobDeleted(backend, memoryId, blobId, expectBlobDeleted = true) {
  try {
    // Verify memory is deleted
    const memoryAfter = await backend.memories_read(memoryId);
    if ("Ok" in memoryAfter) {
      throw new Error(`Memory still exists after deletion: ${memoryId}`);
    }
    echoInfo(`✅ Memory correctly deleted`);

    // Verify blob status
    const blobAfter = await backend.blob_get_meta(blobId);
    if (expectBlobDeleted) {
      if ("Ok" in blobAfter) {
        throw new Error(`Blob still exists after memory deletion (expected cascade deletion)`);
      }
      echoInfo(`✅ Blob correctly deleted via cascade deletion`);
    } else {
      if ("Err" in blobAfter) {
        throw new Error(`Blob was deleted when it should be preserved: ${JSON.stringify(blobAfter.Err)}`);
      }
      echoInfo(`✅ Blob correctly preserved (${blobAfter.Ok.size} bytes)`);
    }

    return { success: true };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

/**
 * Test deletion of memory with blob assets
 */
async function testDeleteMemoryWithBlobAssets(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing deletion of memory with blob assets`);

    // Create test memory with blob using shared helper
    const createResult = await createTestMemoryWithBlob(backend, capsuleId, "Delete Blob");
    if (!createResult.success) {
      throw new Error(createResult.error);
    }

    const { memoryId, blobId } = createResult;

    // Verify memory and blob exist before deletion using shared helper
    const verifyBefore = await verifyMemoryAndBlobExist(backend, memoryId, blobId);
    if (!verifyBefore.success) {
      throw new Error(verifyBefore.error);
    }

    // Delete memory
    const deleteResult = await backend.memories_delete(memoryId);
    if ("Err" in deleteResult) {
      throw new Error(`Memory deletion failed: ${JSON.stringify(deleteResult.Err)}`);
    }
    echoInfo(`✅ Memory deleted successfully`);

    // Verify memory and blob are deleted using shared helper
    const verifyAfter = await verifyMemoryAndBlobDeleted(backend, memoryId, blobId, true);
    if (!verifyAfter.success) {
      throw new Error(verifyAfter.error);
    }

    echoSuccess(`✅ Memory with blob assets deletion test completed`);

    return {
      success: true,
      deletedMemoryId: memoryId,
      deletedBlobId: blobId,
      cascadeDeletion: true,
    };
  } catch (error) {
    echoFail(`❌ Memory with blob assets deletion failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Test deletion of memory with inline assets
 */
async function testDeleteMemoryWithInlineAssets(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing deletion of memory with inline assets`);

    // Create test memory with inline data using shared utility
    const memoryId = await createTestMemory(backend, capsuleId, {
      name: "delete-inline-test",
      description: "Testing deletion of memory with inline assets",
      tags: ["deletion-test", "inline"],
      content: "Test inline content for deletion testing",
      mimeType: "text/plain",
      idempotencyKey: `delete-inline-test-${Date.now()}`,
    });

    // Verify memory exists before deletion
    const memoryBefore = await backend.memories_read(memoryId);
    if ("Err" in memoryBefore) {
      throw new Error(`Memory not found before deletion: ${JSON.stringify(memoryBefore.Err)}`);
    }

    echoInfo(`✅ Memory exists before deletion`);
    echoInfo(`  Memory: ${memoryId} with ${memoryBefore.Ok.inline_assets.length} inline assets`);

    // Delete memory
    const deleteResult = await backend.memories_delete(memoryId);
    if ("Err" in deleteResult) {
      throw new Error(`Memory deletion failed: ${JSON.stringify(deleteResult.Err)}`);
    }
    echoInfo(`✅ Memory deleted successfully`);

    // Verify memory is deleted
    const memoryAfter = await backend.memories_read(memoryId);
    if ("Ok" in memoryAfter) {
      throw new Error(`Memory still exists after deletion`);
    }
    echoInfo(`✅ Memory correctly deleted`);

    echoSuccess(`✅ Memory with inline assets deletion test completed`);

    return {
      success: true,
      deletedMemoryId: memoryId,
      inlineAssetsDeleted: 1,
    };
  } catch (error) {
    echoFail(`❌ Memory with inline assets deletion failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Test deletion of memory with mixed assets
 */
async function testDeleteMemoryWithMixedAssets(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing deletion of memory with mixed assets`);

    // Create test memory with blob using shared helper
    const createResult = await createTestMemoryWithBlob(backend, capsuleId, "Delete Mixed");
    if (!createResult.success) {
      throw new Error(createResult.error);
    }

    const { memoryId, blobId } = createResult;

    // Add inline asset to create mixed assets
    const inlineData = new Uint8Array(500).fill(123);
    const addInlineResult = await addInlineAssetToMemory(backend, memoryId, inlineData, {
      metadata: createDerivativeAssetMetadata({
        name: "additional-inline",
        size: inlineData.length,
        mimeType: "application/octet-stream",
        tags: ["deletion-test", "mixed", "inline"],
        description: "Additional inline asset",
      }),
    });

    if (!addInlineResult.success) {
      throw new Error(`Failed to add inline asset: ${addInlineResult.error}`);
    }

    // Verify memory exists with mixed assets before deletion
    const memoryBefore = await backend.memories_read(memoryId);
    if ("Err" in memoryBefore) {
      throw new Error(`Memory not found before deletion: ${JSON.stringify(memoryBefore.Err)}`);
    }

    echoInfo(`✅ Memory exists before deletion`);
    echoInfo(
      `  Memory: ${memoryId} with ${memoryBefore.Ok.blob_internal_assets.length} blob assets and ${memoryBefore.Ok.inline_assets.length} inline assets`
    );

    // Delete memory
    const deleteResult = await backend.memories_delete(memoryId);
    if ("Err" in deleteResult) {
      throw new Error(`Memory deletion failed: ${JSON.stringify(deleteResult.Err)}`);
    }
    echoInfo(`✅ Memory deleted successfully`);

    // Verify memory and blob are deleted using shared helper
    const verifyAfter = await verifyMemoryAndBlobDeleted(backend, memoryId, blobId, true);
    if (!verifyAfter.success) {
      throw new Error(verifyAfter.error);
    }

    echoSuccess(`✅ Memory with mixed assets deletion test completed`);

    return {
      success: true,
      deletedMemoryId: memoryId,
      deletedBlobId: blobId,
      blobAssetsDeleted: 1,
      inlineAssetsDeleted: 1,
      cascadeDeletion: true,
    };
  } catch (error) {
    echoFail(`❌ Memory with mixed assets deletion failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Test error handling for deletion operations
 */
async function testDeletionErrorHandling(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing error handling for deletion operations`);

    // Try to delete non-existent memory
    const fakeMemoryId = "fake-memory-id-12345";
    const deleteResult = await backend.memories_delete(fakeMemoryId);

    if ("Ok" in deleteResult) {
      throw new Error(`Expected error for non-existent memory deletion, but got success`);
    }

    echoInfo(`✅ Non-existent memory deletion correctly returned error: ${JSON.stringify(deleteResult.Err)}`);

    // Try to delete non-existent blob
    const fakeBlobId = "fake-blob-id-12345";
    const blobDeleteResult = await backend.blob_delete(fakeBlobId);

    if ("Ok" in blobDeleteResult) {
      throw new Error(`Expected error for non-existent blob deletion, but got success`);
    }

    echoInfo(`✅ Non-existent blob deletion correctly returned error: ${JSON.stringify(blobDeleteResult.Err)}`);

    echoSuccess(`✅ Deletion error handling test completed`);

    return {
      success: true,
      testedErrors: ["non-existent-memory", "non-existent-blob"],
    };
  } catch (error) {
    echoFail(`❌ Deletion error handling test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Test performance with multiple assets
 */
async function testDeletionPerformanceWithMultipleAssets(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing deletion performance with multiple assets`);

    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);

    // Upload original file as blob
    const uploadResult = await uploadBufferAsBlob(backend, fileBuffer, capsuleId, {
      createMemory: false,
      idempotencyKey: `performance-test-${Date.now()}`,
    });

    if (!uploadResult.success) {
      throw new Error(`Upload failed: ${uploadResult.error}`);
    }

    // Create memory with original blob
    const memoryMetadata = createMemoryMetadata({
      title: "Performance Deletion Test",
      description: "Testing deletion performance with multiple assets",
      tags: ["deletion-test", "performance"],
    });

    const originalAsset = {
      blob_id: uploadResult.blobId,
      metadata: createImageAssetMetadata({
        name: "original",
        size: fileBuffer.length,
        mimeType: "image/jpeg",
        assetType: "Original",
        tags: ["deletion-test", "performance"],
        description: "Original file for performance test",
      }),
    };

    const createResult = await backend.memories_create_with_internal_blobs_and_inline_assets(
      capsuleId,
      memoryMetadata,
      [originalAsset],
      [], // No inline assets initially
      `performance-test-${Date.now()}`
    );

    if ("Err" in createResult) {
      throw new Error(`Memory creation failed: ${JSON.stringify(createResult.Err)}`);
    }

    const memoryId = createResult.Ok;

    // Add multiple inline assets
    const inlineAssetCount = 5;
    const inlineAssetPromises = [];

    for (let i = 0; i < inlineAssetCount; i++) {
      const inlineData = new Uint8Array(200).fill(i + 1);
      inlineAssetPromises.push(
        addInlineAssetToMemory(backend, memoryId, inlineData, {
          metadata: createDerivativeAssetMetadata({
            name: `inline-${i}`,
            size: inlineData.length,
            mimeType: "application/octet-stream",
            tags: ["deletion-test", "performance", "inline"],
            description: `Inline asset ${i}`,
          }),
        })
      );
    }

    const inlineResults = await Promise.all(inlineAssetPromises);

    // Check if all inline assets were added successfully
    const failedInlineAssets = inlineResults.filter((result) => !result.success);
    if (failedInlineAssets.length > 0) {
      throw new Error(`${failedInlineAssets.length} inline assets failed to be added`);
    }

    echoInfo(`✅ Added ${inlineAssetCount} inline assets to memory`);

    // Verify memory exists with all assets before deletion
    const memoryBefore = await backend.memories_read(memoryId);
    if ("Err" in memoryBefore) {
      throw new Error(`Memory not found before deletion: ${JSON.stringify(memoryBefore.Err)}`);
    }

    const totalAssets = memoryBefore.Ok.blob_internal_assets.length + memoryBefore.Ok.inline_assets.length;
    echoInfo(`✅ Memory exists with ${totalAssets} total assets before deletion`);

    // Measure deletion performance
    const startTime = Date.now();

    // Delete memory
    const deleteResult = await backend.memories_delete(memoryId);
    if ("Err" in deleteResult) {
      throw new Error(`Memory deletion failed: ${JSON.stringify(deleteResult.Err)}`);
    }

    const endTime = Date.now();
    const deletionTime = endTime - startTime;

    echoInfo(`✅ Memory deleted in ${deletionTime}ms`);

    // Verify memory is deleted
    const memoryAfter = await backend.memories_read(memoryId);
    if ("Ok" in memoryAfter) {
      throw new Error(`Memory still exists after deletion`);
    }

    echoInfo(`✅ Memory correctly deleted`);

    // Verify blob is also deleted (cascade deletion)
    const blobAfter = await backend.blob_get_meta(uploadResult.blobId);
    if ("Ok" in blobAfter) {
      throw new Error(`Blob still exists after memory deletion (expected cascade deletion)`);
    }

    echoInfo(`✅ Blob correctly deleted via cascade deletion`);

    echoSuccess(`✅ Deletion performance test completed`);
    echoInfo(`  Deleted ${totalAssets} assets in ${deletionTime}ms`);
    echoInfo(`  Average time per asset: ${(deletionTime / totalAssets).toFixed(2)}ms`);

    return {
      success: true,
      deletedMemoryId: memoryId,
      deletedBlobId: uploadResult.blobId,
      totalAssetsDeleted: totalAssets,
      deletionTime: deletionTime,
      averageTimePerAsset: deletionTime / totalAssets,
    };
  } catch (error) {
    echoFail(`❌ Deletion performance test failed: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Test full deletion workflow (memory + all assets)
 */
async function testFullDeletionWorkflow(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing full deletion workflow`);

    // Create test memory with blob using shared helper
    const createResult = await createTestMemoryWithBlob(backend, capsuleId, "Full Deletion");
    if (!createResult.success) {
      throw new Error(createResult.error);
    }

    const { memoryId, blobId } = createResult;
    echoInfo(`✅ Created memory: ${memoryId} with blob: ${blobId}`);

    // Verify memory and blob exist before deletion using shared helper
    const verifyBefore = await verifyMemoryAndBlobExist(backend, memoryId, blobId);
    if (!verifyBefore.success) {
      throw new Error(verifyBefore.error);
    }

    // Delete memory with assets (full deletion)
    echoInfo(`🗑️ Deleting memory with assets (delete_assets: true)...`);
    const deleteResult = await backend.memories_delete(memoryId, true);
    if ("Err" in deleteResult) {
      throw new Error(`Memory deletion failed: ${JSON.stringify(deleteResult.Err)}`);
    }
    echoInfo(`✅ Memory deleted successfully`);

    // Verify memory and blob are deleted using shared helper
    const verifyAfter = await verifyMemoryAndBlobDeleted(backend, memoryId, blobId, true);
    if (!verifyAfter.success) {
      throw new Error(verifyAfter.error);
    }

    echoInfo(`✅ Full deletion workflow completed successfully - memory and all assets deleted`);
    return { success: true };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

/**
 * Test selective deletion workflow (memory only, preserve assets)
 */
async function testSelectiveDeletionWorkflow(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing selective deletion workflow`);

    // Create test memory with blob using shared helper
    const createResult = await createTestMemoryWithBlob(backend, capsuleId, "Selective Deletion");
    if (!createResult.success) {
      throw new Error(createResult.error);
    }

    const { memoryId, blobId } = createResult;
    echoInfo(`✅ Created memory: ${memoryId} with blob: ${blobId}`);

    // Verify memory and blob exist before deletion using shared helper
    const verifyBefore = await verifyMemoryAndBlobExist(backend, memoryId, blobId);
    if (!verifyBefore.success) {
      throw new Error(verifyBefore.error);
    }

    // Delete memory without assets (metadata-only deletion)
    echoInfo(`🗑️ Deleting memory without assets (delete_assets: false)...`);
    const deleteResult = await backend.memories_delete(memoryId, false);
    if ("Err" in deleteResult) {
      throw new Error(`Memory deletion failed: ${JSON.stringify(deleteResult.Err)}`);
    }
    echoInfo(`✅ Memory metadata deleted successfully`);

    // Verify memory is deleted but blob is preserved using shared helper
    const verifyAfter = await verifyMemoryAndBlobDeleted(backend, memoryId, blobId, false);
    if (!verifyAfter.success) {
      throw new Error(verifyAfter.error);
    }

    echoInfo(`✅ Selective deletion workflow completed successfully - memory deleted, assets preserved`);
    return { success: true };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

/**
 * Test delete function unit with multiple assets
 */
async function testDeleteFunctionUnit(backend, capsuleId) {
  try {
    echoInfo(`🧪 Testing delete function unit test with multiple assets`);

    // Step 1: Create a memory with multiple internal blob assets using shared utilities
    echoInfo(`📤 Creating memory with 4 internal blob assets...`);

    // Upload original file using shared utility
    const fileBuffer = readFileAsBuffer(TEST_IMAGE_PATH);
    const uploadResult = await uploadBufferAsBlob(backend, fileBuffer, capsuleId, {
      createMemory: false,
      idempotencyKey: `delete-unit-original-${Date.now()}`,
    });

    if (!uploadResult.success) {
      throw new Error(`Original upload failed: ${uploadResult.error}`);
    }

    // Create 3 additional small blob assets (simulating derivatives) using shared utility
    const derivativeBlobIds = [];
    for (let i = 0; i < 3; i++) {
      const derivativeData = new Uint8Array(1000).fill(i + 1);
      const derivativeUploadResult = await uploadBufferAsBlob(backend, derivativeData, capsuleId, {
        createMemory: false,
        idempotencyKey: `delete-unit-derivative-${i}-${Date.now()}`,
      });

      if (!derivativeUploadResult.success) {
        throw new Error(`Derivative ${i} upload failed: ${derivativeUploadResult.error}`);
      }

      derivativeBlobIds.push(derivativeUploadResult.blobId);
    }

    // Create memory with all 4 blob assets using helper function
    const memoryMetadata = createMemoryMetadata({
      title: "Delete Unit Test",
      description: "Testing delete function with multiple assets",
      tags: ["test", "delete-unit"],
    });

    const internalBlobAssets = [
      {
        blob_id: uploadResult.blobId,
        metadata: createImageAssetMetadata({
          name: "original",
          size: fileBuffer.length,
          mimeType: "image/jpeg",
          assetType: "Original",
          tags: ["test", "delete-unit"],
          description: "Original file",
        }),
      },
      ...derivativeBlobIds.map((blobId, i) => ({
        blob_id: blobId,
        metadata: createDerivativeAssetMetadata({
          name: `derivative-${i}`,
          size: 1000,
          mimeType: "image/jpeg",
          tags: ["test", "delete-unit"],
          description: `Derivative ${i}`,
        }),
      })),
    ];

    // Create a placeholder inline asset
    const placeholderData = new Uint8Array(100).fill(255); // Small placeholder
    const inlineAssets = [
      {
        data: Array.from(placeholderData),
        metadata: createDerivativeAssetMetadata({
          name: "placeholder",
          size: placeholderData.length,
          mimeType: "image/jpeg",
          tags: ["test", "delete-unit", "placeholder"],
          description: "Placeholder asset",
        }),
      },
    ];

    const createResult = await backend.memories_create_with_internal_blobs_and_inline_assets(
      capsuleId,
      memoryMetadata,
      internalBlobAssets,
      inlineAssets,
      `delete-unit-test-${Date.now()}`
    );

    if ("Err" in createResult) {
      throw new Error(`Memory creation failed: ${JSON.stringify(createResult.Err)}`);
    }

    const memoryId = createResult.Ok;
    echoInfo(`✅ Created memory: ${memoryId}`);

    const allBlobIds = [uploadResult.blobId, ...derivativeBlobIds];
    echoInfo(`✅ Created ${allBlobIds.length} assets: ${allBlobIds.join(", ")}`);

    // Step 2: Verify all assets exist using shared verification utility
    echoInfo(`🔍 Verifying all ${allBlobIds.length} assets exist before deletion...`);
    for (const blobId of allBlobIds) {
      const verified = await verifyBlobMetadata(
        backend,
        blobId,
        blobId === uploadResult.blobId ? fileBuffer.length : 1000
      );
      if (!verified) {
        throw new Error(`Asset ${blobId} verification failed before deletion`);
      }
      echoInfo(`  ✅ Asset ${blobId} verified`);
    }

    // Step 3: Test full deletion (delete_assets: true)
    echoInfo(`🗑️ Testing full deletion (delete_assets: true)...`);
    const deleteResult = await backend.memories_delete(memoryId, true);
    if ("Err" in deleteResult) {
      throw new Error(`Memory deletion failed: ${JSON.stringify(deleteResult.Err)}`);
    }
    echoInfo(`✅ Memory deleted successfully`);

    // Step 4: Verify memory is gone
    const memoryReadAfter = await backend.memories_read(memoryId);
    if ("Ok" in memoryReadAfter) {
      throw new Error(`Memory still exists after deletion: ${memoryId}`);
    }
    echoInfo(`✅ Memory confirmed deleted`);

    // Step 5: Verify ALL assets are deleted
    echoInfo(`🔍 Verifying all ${allBlobIds.length} assets are deleted...`);
    for (const blobId of allBlobIds) {
      const meta = await backend.blob_get_meta(blobId);
      if ("Ok" in meta) {
        throw new Error(`Asset ${blobId} still exists after deletion - should be deleted!`);
      }
      if ("Err" in meta && "NotFound" in meta.Err) {
        echoInfo(`  ✅ Asset ${blobId} confirmed deleted`);
      } else {
        throw new Error(`Asset ${blobId} deletion check failed: ${JSON.stringify(meta)}`);
      }
    }

    echoInfo(`✅ Delete function unit test completed successfully - all ${allBlobIds.length} assets deleted`);
    return { success: true };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

/**
 * Main test function
 */
async function main() {
  const { backend, canisterId, testFilter } = parseTestArgs();

  const allTests = [
    { name: "Delete Memory with Blob Assets", fn: testDeleteMemoryWithBlobAssets },
    { name: "Delete Memory with Inline Assets", fn: testDeleteMemoryWithInlineAssets },
    { name: "Delete Memory with Mixed Assets", fn: testDeleteMemoryWithMixedAssets },
    { name: "Full Deletion Workflow", fn: testFullDeletionWorkflow },
    { name: "Selective Deletion Workflow", fn: testSelectiveDeletionWorkflow },
    { name: "Deletion Error Handling", fn: testDeletionErrorHandling },
    { name: "Deletion Performance with Multiple Assets", fn: testDeletionPerformanceWithMultipleAssets },
    { name: "Delete Function Unit Test", fn: testDeleteFunctionUnit },
  ];

  // Filter tests if test name is provided
  const tests = testFilter ? allTests.filter((test) => test.name === testFilter) : allTests;

  if (testFilter && tests.length === 0) {
    echoFail(`Test not found: "${testFilter}"`);
    echoFail("Available tests:");
    allTests.forEach((test) => echoFail(`  - "${test.name}"`));
    process.exit(1);
  }

  echoInfo(`🧪 Running ${tests.length} deletion workflow test(s)...`);

  for (const test of tests) {
    await runTest(test.name, test.fn, backend, canisterId);
  }

  echoSuccess(`✅ All deletion workflow tests completed!`);
}

// Run the tests
main().catch((error) => {
  echoFail(`❌ Test execution failed: ${error.message}`);
  process.exit(1);
});
