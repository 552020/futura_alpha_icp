/**
 * Memory Creation Helper Functions
 *
 * Provides utilities for creating and managing test memories
 * Based on the working pattern from memories tests
 */

import { logInfo, logSuccess, logError } from "./logging.js";
import crypto from "crypto";

/**
 * Create asset metadata for different asset types
 * @param {string} name - Asset name
 * @param {number} size - Asset size in bytes
 * @param {string} mimeType - MIME type
 * @param {string} assetType - Asset type (Note, Document, Image, etc.)
 * @returns {Object} Asset metadata
 */
export function createAssetMetadata(name, size, mimeType, assetType = "Note") {
  const contentHash = crypto.createHash("sha256").update(name).digest();

  const baseMetadata = {
    name: name,
    description: [`Test memory: ${name}`],
    mime_type: mimeType,
    bytes: size,
    sha256: [Array.from(contentHash)],
    url: [],
    height: [],
    updated_at: BigInt(Date.now() * 1000000),
    asset_type: { Original: null },
    storage_key: [],
    tags: [],
    processing_error: [],
    created_at: BigInt(Date.now() * 1000000),
    deleted_at: [],
    asset_location: [],
    width: [],
    processing_status: [],
    bucket: [],
  };

  // Return different asset type structures
  switch (assetType) {
    case "Document":
      return {
        Document: {
          base: baseMetadata,
          document_type: [],
          language: [],
          page_count: [],
          word_count: [],
        },
      };
    case "Image":
      return {
        Image: {
          base: baseMetadata,
          dpi: [],
          color_space: [],
          exif_data: [],
          compression_ratio: [],
          orientation: [],
        },
      };
    case "Note":
    default:
      return {
        Note: {
          base: baseMetadata,
          language: [],
          word_count: [],
          format: [],
        },
      };
  }
}

/**
 * Create a test memory with inline content
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @param {Object} options - Memory creation options
 * @returns {Promise<string>} Memory ID
 */
export async function createTestMemory(actor, capsuleId, options = {}) {
  const {
    name = "test_memory",
    content = "Hello, this is a test memory!",
    mimeType = "text/plain",
    assetType = "Note",
    idempotencyKey = null,
  } = options;

  logInfo(`Creating test memory: ${name}`);

  try {
    // Convert content to bytes
    const contentBytes = Array.from(Buffer.from(content, "utf8"));
    logInfo(`Content prepared: ${contentBytes.length} bytes`);

    // Create asset metadata
    const assetMetadata = createAssetMetadata(name, contentBytes.length, mimeType, assetType);
    logInfo(`Asset metadata created for type: ${assetType}`);

    // Generate idempotency key if not provided
    const finalIdempotencyKey = idempotencyKey || `test_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    // Create memory using the working pattern
    const memoryResult = await actor.memories_create(
      capsuleId,
      [contentBytes], // inline content
      [], // no blob ref
      [], // no external location
      [], // no external storage key
      [], // no external URL
      [], // no external size
      [], // no external hash
      assetMetadata,
      finalIdempotencyKey
    );

    if (!memoryResult.Ok) {
      throw new Error(`Failed to create memory: ${JSON.stringify(memoryResult.Err)}`);
    }

    const memoryId = memoryResult.Ok;
    logSuccess(`✅ Test memory created: ${memoryId}`);
    return memoryId;
  } catch (error) {
    logError(`❌ Failed to create test memory: ${error.message}`);
    throw error;
  }
}

/**
 * Create a test memory with image content
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @param {Object} options - Memory creation options
 * @returns {Promise<string>} Memory ID
 */
export async function createTestImageMemory(actor, capsuleId, options = {}) {
  const { name = "test_image", width = 1, height = 1, idempotencyKey = null } = options;

  logInfo(`Creating test image memory: ${name}`);

  try {
    // Create minimal image data (1x1 pixel PNG)
    const imageData = Buffer.from([
      0x89,
      0x50,
      0x4e,
      0x47,
      0x0d,
      0x0a,
      0x1a,
      0x0a, // PNG signature
      0x00,
      0x00,
      0x00,
      0x0d, // IHDR chunk length
      0x49,
      0x48,
      0x44,
      0x52, // IHDR
      0x00,
      0x00,
      0x00,
      0x01, // width: 1
      0x00,
      0x00,
      0x00,
      0x01, // height: 1
      0x08,
      0x02,
      0x00,
      0x00,
      0x00, // bit depth, color type, compression, filter, interlace
      0x90,
      0x77,
      0x53,
      0xde, // CRC
      0x00,
      0x00,
      0x00,
      0x0c, // IDAT chunk length
      0x49,
      0x44,
      0x41,
      0x54, // IDAT
      0x08,
      0x99,
      0x01,
      0x01,
      0x00,
      0x00,
      0x00,
      0xff,
      0xff,
      0x00,
      0x00,
      0x00,
      0x02,
      0x00,
      0x01,
      0x00,
      0x00,
      0x00,
      0x00, // compressed data
      0x49,
      0x45,
      0x4e,
      0x44, // IEND
      0xae,
      0x42,
      0x60,
      0x82, // CRC
    ]);

    const contentBytes = Array.from(imageData);
    logInfo(`Image data prepared: ${contentBytes.length} bytes`);

    // Create image asset metadata
    const assetMetadata = createAssetMetadata(name, contentBytes.length, "image/png", "Image");
    logInfo(`Image asset metadata created`);

    // Generate idempotency key if not provided
    const finalIdempotencyKey = idempotencyKey || `image_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    // Create memory
    const memoryResult = await actor.memories_create(
      capsuleId,
      [contentBytes], // inline image content
      [], // no blob ref
      [], // no external location
      [], // no external storage key
      [], // no external URL
      [], // no external size
      [], // no external hash
      assetMetadata,
      finalIdempotencyKey
    );

    if (!memoryResult.Ok) {
      throw new Error(`Failed to create image memory: ${JSON.stringify(memoryResult.Err)}`);
    }

    const memoryId = memoryResult.Ok;
    logSuccess(`✅ Test image memory created: ${memoryId}`);
    return memoryId;
  } catch (error) {
    logError(`❌ Failed to create test image memory: ${error.message}`);
    throw error;
  }
}

/**
 * Create a test memory with document content
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @param {Object} options - Memory creation options
 * @returns {Promise<string>} Memory ID
 */
export async function createTestDocumentMemory(actor, capsuleId, options = {}) {
  const { name = "test_document", content = "This is a test document content.", idempotencyKey = null } = options;

  logInfo(`Creating test document memory: ${name}`);

  try {
    // Convert content to bytes
    const contentBytes = Array.from(Buffer.from(content, "utf8"));
    logInfo(`Document content prepared: ${contentBytes.length} bytes`);

    // Create document asset metadata
    const assetMetadata = createAssetMetadata(name, contentBytes.length, "text/plain", "Document");
    logInfo(`Document asset metadata created`);

    // Generate idempotency key if not provided
    const finalIdempotencyKey = idempotencyKey || `document_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    // Create memory
    const memoryResult = await actor.memories_create(
      capsuleId,
      [contentBytes], // inline document content
      [], // no blob ref
      [], // no external location
      [], // no external storage key
      [], // no external URL
      [], // no external size
      [], // no external hash
      assetMetadata,
      finalIdempotencyKey
    );

    if (!memoryResult.Ok) {
      throw new Error(`Failed to create document memory: ${JSON.stringify(memoryResult.Err)}`);
    }

    const memoryId = memoryResult.Ok;
    logSuccess(`✅ Test document memory created: ${memoryId}`);
    return memoryId;
  } catch (error) {
    logError(`❌ Failed to create test document memory: ${error.message}`);
    throw error;
  }
}

/**
 * Create multiple test memories for bulk operations
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @param {number} count - Number of memories to create
 * @param {Object} options - Memory creation options
 * @returns {Promise<string[]>} Array of memory IDs
 */
export async function createTestMemoriesBatch(actor, capsuleId, count, options = {}) {
  const { prefix = "bulk_test", assetType = "Note" } = options;

  logInfo(`Creating ${count} test memories with prefix: ${prefix}`);

  const memoryIds = [];

  for (let i = 1; i <= count; i++) {
    const memoryId = await createTestMemory(actor, capsuleId, {
      name: `${prefix}_memory_${i}`,
      content: `Test memory ${i} for bulk operations`,
      assetType: assetType,
      idempotencyKey: `${prefix}_${i}_${Date.now()}`,
    });
    memoryIds.push(memoryId);
  }

  logSuccess(`✅ Created ${memoryIds.length} test memories`);
  return memoryIds;
}

/**
 * Create a memory from a blob (for upload-download compatibility)
 * @param {Object} backend - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @param {string} fileName - File name
 * @param {number} fileSize - File size
 * @param {string} blobId - Blob ID
 * @param {Object} result - Upload result
 * @param {Object} options - Options
 * @returns {Promise<Object>} Result with success and memoryId or error
 */
export async function createMemoryFromBlob(backend, capsuleId, fileName, fileSize, blobId, result, options = {}) {
  try {
    logInfo(`Creating memory from blob: ${fileName}`);

    // Create asset metadata based on file type
    let assetMetadata;
    if (fileName.toLowerCase().match(/\.(jpg|jpeg|png|gif|webp)$/)) {
      assetMetadata = createAssetMetadata(fileName, fileSize, "image/jpeg", "Image");
    } else {
      assetMetadata = createAssetMetadata(fileName, fileSize, "application/octet-stream", "Document");
    }

    const idempotencyKey = `blob_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    // Create memory with blob reference
    const memoryResult = await backend.memories_create(
      capsuleId,
      [], // no inline content
      [blobId], // blob reference
      [], // no external location
      [], // no external storage key
      [], // no external URL
      [], // no external size
      [], // no external hash
      assetMetadata,
      idempotencyKey
    );

    if (!memoryResult.Ok) {
      throw new Error(`Failed to create memory from blob: ${JSON.stringify(memoryResult.Err)}`);
    }

    const memoryId = memoryResult.Ok;
    logSuccess(`✅ Memory created from blob: ${memoryId}`);
    return { success: true, memoryId };
  } catch (error) {
    logError(`❌ Failed to create memory from blob: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Clean up test memories
 * @param {Object} actor - Backend actor
 * @param {string[]} memoryIds - Array of memory IDs to delete
 * @returns {Promise<void>}
 */
export async function cleanupTestMemories(actor, memoryIds) {
  logInfo(`Cleaning up ${memoryIds.length} test memories`);

  for (const memoryId of memoryIds) {
    try {
      await actor.memories_delete(memoryId, true); // true for cleanup assets
      logInfo(`Deleted memory: ${memoryId}`);
    } catch (error) {
      logError(`Failed to cleanup memory ${memoryId}: ${error.message}`);
    }
  }

  logSuccess(`✅ Cleanup completed for ${memoryIds.length} memories`);
}
