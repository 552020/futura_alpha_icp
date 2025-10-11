#!/usr/bin/env node

/**
 * Test for memories_add_asset and memories_add_inline_asset functions
 *
 * This test verifies the new backend functionality for adding assets to existing memories.
 * It tests both blob assets and inline assets.
 */

import {
  parseTestArgs,
  createTestActorOptions,
  createTestActor,
  logNetworkConfig,
  getOrCreateTestCapsuleForUpload,
  createTestRunner,
  uploadFileAsBlob,
  createMemoryFromBlob,
  addAssetToMemory,
  addInlineAssetToMemory,
  verifyMemoryIntegrity,
  readFileAsBuffer,
  computeSHA256Hash,
} from "../../utils/index.js";

const TEST_IMAGE_PATH = "./assets/input/orange_tiny.jpg";

/**
 * Test adding a blob asset to an existing memory
 */
async function testAddBlobAsset(backend, capsuleId) {
  console.log("🧪 Testing memories_add_asset (blob asset)...");

  try {
    // Step 1: Upload a file as a blob
    const uploadResult = await uploadFileAsBlob(backend, TEST_IMAGE_PATH, capsuleId, {
      createMemory: false,
      idempotencyKey: `test-blob-${Date.now()}`,
    });

    if (!uploadResult.success) {
      return { success: false, error: `Upload failed: ${uploadResult.error}` };
    }

    const { blobId } = uploadResult;

    // Step 2: Create a memory with the original blob
    const fileBuffer = await readFileAsBuffer(TEST_IMAGE_PATH);
    const fileName = "orange_tiny.jpg";
    const fileSize = fileBuffer.length;
    const fileHash = await computeSHA256Hash(fileBuffer);

    console.log(`📝 Creating memory with blob ${blobId}...`);
    const memoryResult = await createMemoryFromBlob(backend, capsuleId, fileName, fileSize, blobId, uploadResult, {
      assetType: "image",
      mimeType: "image/jpeg",
      memoryType: { Image: null },
    });

    if (!memoryResult.success) {
      console.error(`❌ Memory creation failed: ${memoryResult.error}`);
      return { success: false, error: `Memory creation failed: ${memoryResult.error}` };
    }

    const memoryId = memoryResult.memoryId;

    // Step 3: Upload another blob for the derivative
    const derivativeUploadResult = await uploadFileAsBlob(backend, TEST_IMAGE_PATH, capsuleId, {
      createMemory: false,
      idempotencyKey: `test-derivative-${Date.now()}`,
    });

    if (!derivativeUploadResult.success) {
      return { success: false, error: `Derivative upload failed: ${derivativeUploadResult.error}` };
    }

    const { blobId: derivativeBlobId } = derivativeUploadResult;

    // Step 4: Add the derivative as a display asset to the existing memory
    console.log(`➕ Adding asset ${derivativeBlobId} to memory ${memoryId}...`);
    const addResult = await addAssetToMemory(backend, memoryId, derivativeBlobId, {
      assetType: "display",
      mimeType: "image/webp",
      idempotencyKey: `test-add-display-${Date.now()}`,
    });

    if (!addResult.success) {
      console.error(`❌ Add asset failed: ${addResult.error}`);
      return { success: false, error: `Add asset failed: ${addResult.error}` };
    }

    // Step 5: Verify the memory now has 2 assets
    const verifyResult = await verifyMemoryIntegrity(backend, memoryId, 2);
    if (!verifyResult) {
      return { success: false, error: "Memory verification failed after adding asset" };
    }

    console.log(`✅ Successfully added blob asset ${addResult.assetId} to memory ${memoryId}`);
    return { success: true, data: { memoryId, assetId: addResult.assetId } };
  } catch (error) {
    console.error(`❌ Test failed with error: ${error.message}`);
    console.error(`❌ Error stack: ${error.stack}`);
    return { success: false, error: `Test failed: ${error.message}` };
  }
}

/**
 * Test adding an inline asset to an existing memory
 */
async function testAddInlineAsset(backend, capsuleId) {
  console.log("🧪 Testing memories_add_inline_asset (inline asset)...");

  try {
    // Step 1: Upload a file as a blob and create memory
    const uploadResult = await uploadFileAsBlob(backend, TEST_IMAGE_PATH, capsuleId, {
      createMemory: false,
      idempotencyKey: `test-inline-base-${Date.now()}`,
    });

    if (!uploadResult.success) {
      return { success: false, error: `Upload failed: ${uploadResult.error}` };
    }

    const { blobId } = uploadResult;

    // Step 2: Create a memory with the original blob
    const fileBuffer = await readFileAsBuffer(TEST_IMAGE_PATH);
    const fileName = "orange_tiny.jpg";
    const fileSize = fileBuffer.length;

    const memoryResult = await createMemoryFromBlob(backend, capsuleId, fileName, fileSize, blobId, uploadResult, {
      assetType: "image",
      mimeType: "image/jpeg",
      memoryType: { Image: null },
    });

    if (!memoryResult.success) {
      return { success: false, error: `Memory creation failed: ${memoryResult.error}` };
    }

    const memoryId = memoryResult.memoryId;

    // Step 3: Create a small inline asset (placeholder)
    const placeholderData = new Uint8Array([0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46]); // Small JPEG header

    // Step 4: Add the inline asset to the existing memory
    console.log(`➕ Adding inline asset to memory ${memoryId}...`);
    const addResult = await addInlineAssetToMemory(backend, memoryId, placeholderData, {
      assetType: "placeholder",
      mimeType: "image/jpeg",
      idempotencyKey: `test-add-placeholder-${Date.now()}`,
    });

    if (!addResult.success) {
      console.error(`❌ Add inline asset failed: ${addResult.error}`);
      return { success: false, error: `Add inline asset failed: ${addResult.error}` };
    }

    // Step 5: Verify the memory now has 1 blob asset + 1 inline asset
    console.log(`🔍 Verifying memory has 1 blob asset + 1 inline asset...`);
    const memoryReadResult = await backend.memories_read(memoryId);
    if ("Err" in memoryReadResult) {
      return { success: false, error: `Failed to read memory: ${JSON.stringify(memoryReadResult.Err)}` };
    }

    const memory = memoryReadResult.Ok;
    const blobAssetCount = memory.blob_internal_assets.length;
    const inlineAssetCount = memory.inline_assets.length;

    console.log(`📦 Blob assets: ${blobAssetCount}, Inline assets: ${inlineAssetCount}`);

    if (blobAssetCount !== 1 || inlineAssetCount !== 1) {
      return {
        success: false,
        error: `Asset count mismatch: expected 1 blob + 1 inline, got ${blobAssetCount} blob + ${inlineAssetCount} inline`,
      };
    }

    console.log(`✅ Successfully added inline asset ${addResult.assetId} to memory ${memoryId}`);
    return { success: true, data: { memoryId, assetId: addResult.assetId } };
  } catch (error) {
    console.error(`❌ Test failed with error: ${error.message}`);
    console.error(`❌ Error stack: ${error.stack}`);
    return { success: false, error: `Test failed: ${error.message}` };
  }
}

/**
 * Main test function
 */
async function main() {
  const parsedArgs = parseTestArgs();
  const canisterId = parsedArgs.canisterId;

  if (!canisterId) {
    console.error("❌ Canister ID is required");
    process.exit(1);
  }

  // Create test actor
  const actorOptions = createTestActorOptions(parsedArgs);
  const { actor: backend, canisterId: actualCanisterId } = await createTestActor(actorOptions);

  // Log network configuration
  logNetworkConfig(parsedArgs, actualCanisterId);

  // Get or create test capsule
  const capsuleId = await getOrCreateTestCapsuleForUpload(backend, parsedArgs);

  // Create test runner
  const runner = createTestRunner("memories_add_asset");

  // Run tests
  await runner.runTest("Add blob asset to memory", testAddBlobAsset, backend, capsuleId);
  await runner.runTest("Add inline asset to memory", testAddInlineAsset, backend, capsuleId);

  // Print summary
  runner.printTestSummary();
}

// Run the test
main().catch((error) => {
  console.error("❌ Test execution failed:", error.message);
  process.exit(1);
});
