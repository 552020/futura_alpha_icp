/**
 * Memory Data Creation Utilities
 *
 * Provides utilities for creating and managing test memories
 */

/**
 * Create a test memory with inline data
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @param {Object} options - Memory creation options
 * @returns {Promise<string>} Memory ID
 */
export async function createTestMemory(actor, capsuleId, options = {}) {
  const {
    name = "test_memory",
    description = "Test memory for testing",
    tags = ["test"],
    content = "Test Memory Content",
    mimeType = "text/plain",
    idempotencyKey = null,
  } = options;

  try {
    // Convert content to bytes
    const contentBytes = new TextEncoder().encode(content);
    const inlineData = Array.from(contentBytes);

    // Create asset metadata
    const assetMetadata = {
      Document: {
        base: {
          name: name,
          description: [description],
          tags: tags,
          asset_type: { Original: null },
          bytes: BigInt(inlineData.length),
          mime_type: mimeType,
          sha256: [],
          width: [],
          height: [],
          url: [],
          storage_key: [],
          bucket: [],
          asset_location: [],
          processing_status: [],
          processing_error: [],
          created_at: BigInt(Date.now() * 1000000),
          updated_at: BigInt(Date.now() * 1000000),
          deleted_at: [],
        },
        page_count: [],
        document_type: [],
        language: [],
        word_count: [],
      },
    };

    const idem = idempotencyKey || `test_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    const result = await actor.memories_create(
      capsuleId,
      [inlineData], // opt blob - inline data
      [], // opt BlobRef - no blob reference
      [], // opt StorageEdgeBlobType - no storage type
      [], // opt text - no storage key
      [], // opt text - no bucket
      [], // opt nat64 - no file_created_at
      [], // opt blob - no sha256 hash
      assetMetadata,
      idem
    );

    if (result.Ok) {
      return result.Ok;
    } else {
      throw new Error(`Memory creation failed: ${JSON.stringify(result)}`);
    }
  } catch (error) {
    throw new Error(`Failed to create test memory: ${error.message}`);
  }
}

/**
 * Create multiple test memories for bulk operations
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @param {number} count - Number of memories to create
 * @param {Object} options - Options
 * @returns {Promise<string[]>} Array of memory IDs
 */
export async function createTestMemoriesBatch(actor, capsuleId, count, options = {}) {
  const memoryIds = [];
  const { prefix = "bulk_test", baseContent = "Test Memory" } = options;

  for (let i = 1; i <= count; i++) {
    const memoryId = await createTestMemory(actor, capsuleId, {
      name: `${prefix}_memory_${i}`,
      description: `Test memory ${i} for bulk operations`,
      content: `${baseContent} ${i}`,
      tags: ["test", "bulk"],
      idempotencyKey: `bulk_test_${i}_${Date.now()}`,
    });
    memoryIds.push(memoryId);
  }

  return memoryIds;
}

/**
 * Create a test memory with blob data
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @param {Object} options - Memory creation options
 * @returns {Promise<string>} Memory ID
 */
export async function createTestMemoryWithBlob(actor, capsuleId, options = {}) {
  const {
    name = "test_blob_memory",
    description = "Test memory with blob data",
    tags = ["test", "blob"],
    blobRef = null,
    fileSize = 1024,
    mimeType = "application/octet-stream",
    idempotencyKey = null,
  } = options;

  try {
    if (!blobRef) {
      throw new Error("Blob reference is required for blob memory");
    }

    // Create asset metadata for blob
    const assetMetadata = {
      Document: {
        base: {
          name: name,
          description: [description],
          tags: tags,
          asset_type: { Original: null },
          bytes: BigInt(fileSize),
          mime_type: mimeType,
          sha256: [],
          width: [],
          height: [],
          url: [],
          storage_key: [],
          bucket: [],
          asset_location: [],
          processing_status: [],
          processing_error: [],
          created_at: BigInt(Date.now() * 1000000),
          updated_at: BigInt(Date.now() * 1000000),
          deleted_at: [],
        },
        page_count: [],
        document_type: [],
        language: [],
        word_count: [],
      },
    };

    const idem = idempotencyKey || `test_blob_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    const result = await actor.memories_create(
      capsuleId,
      [], // opt blob - no inline data
      [blobRef], // opt BlobRef - blob reference
      [], // opt StorageEdgeBlobType - no storage type
      [], // opt text - no storage key
      [], // opt text - no bucket
      [], // opt nat64 - no file_created_at
      [], // opt blob - no sha256 hash
      assetMetadata,
      idem
    );

    if (result.Ok) {
      return result.Ok;
    } else {
      throw new Error(`Blob memory creation failed: ${JSON.stringify(result)}`);
    }
  } catch (error) {
    throw new Error(`Failed to create test blob memory: ${error.message}`);
  }
}

/**
 * Create a test memory with external storage
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @param {Object} options - Memory creation options
 * @returns {Promise<string>} Memory ID
 */
export async function createTestMemoryWithExternal(actor, capsuleId, options = {}) {
  const {
    name = "test_external_memory",
    description = "Test memory with external storage",
    tags = ["test", "external"],
    storageType = "S3",
    storageKey = "test-bucket/test-file.jpg",
    url = "https://s3.amazonaws.com/test-bucket/test-file.jpg",
    fileSize = 2048,
    mimeType = "image/jpeg",
    idempotencyKey = null,
  } = options;

  try {
    // Create asset metadata for external storage
    const assetMetadata = {
      Document: {
        base: {
          name: name,
          description: [description],
          tags: tags,
          asset_type: { Original: null },
          bytes: BigInt(fileSize),
          mime_type: mimeType,
          sha256: [],
          width: [],
          height: [],
          url: [url],
          storage_key: [storageKey],
          bucket: [],
          asset_location: [],
          processing_status: [],
          processing_error: [],
          created_at: BigInt(Date.now() * 1000000),
          updated_at: BigInt(Date.now() * 1000000),
          deleted_at: [],
        },
        page_count: [],
        document_type: [],
        language: [],
        word_count: [],
      },
    };

    const idem = idempotencyKey || `test_external_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    const result = await actor.memories_create(
      capsuleId,
      [], // opt blob - no inline data
      [], // opt BlobRef - no blob reference
      [{ [storageType]: null }], // opt StorageEdgeBlobType - storage type
      [storageKey], // opt text - storage key
      [], // opt text - no bucket
      [], // opt nat64 - no file_created_at
      [], // opt blob - no sha256 hash
      assetMetadata,
      idem
    );

    if (result.Ok) {
      return result.Ok;
    } else {
      throw new Error(`External memory creation failed: ${JSON.stringify(result)}`);
    }
  } catch (error) {
    throw new Error(`Failed to create test external memory: ${error.message}`);
  }
}

/**
 * Get memory information
 * @param {Object} actor - Backend actor
 * @param {string} memoryId - Memory ID
 * @returns {Promise<Object>} Memory information
 */
export async function getMemoryInfo(actor, memoryId) {
  try {
    const result = await actor.memories_read(memoryId);

    if (result.Ok) {
      return result.Ok;
    } else {
      throw new Error(`Failed to get memory info: ${JSON.stringify(result)}`);
    }
  } catch (error) {
    throw new Error(`Failed to get memory info: ${error.message}`);
  }
}

/**
 * List memories in a capsule
 * @param {Object} actor - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @returns {Promise<Object[]>} Array of memories
 */
export async function listMemories(actor, capsuleId) {
  try {
    const result = await actor.memories_list(capsuleId);

    if (Array.isArray(result)) {
      return result;
    } else if (result.Ok && Array.isArray(result.Ok)) {
      return result.Ok;
    } else {
      return [];
    }
  } catch (error) {
    throw new Error(`Failed to list memories: ${error.message}`);
  }
}

/**
 * Delete a test memory
 * @param {Object} actor - Backend actor
 * @param {string} memoryId - Memory ID
 * @returns {Promise<boolean>} Success status
 */
export async function deleteTestMemory(actor, memoryId) {
  try {
    const result = await actor.memories_delete(memoryId);
    return result.Ok || false;
  } catch (error) {
    console.warn(`Failed to delete memory ${memoryId}: ${error.message}`);
    return false;
  }
}

/**
 * Clean up test memories
 * @param {Object} actor - Backend actor
 * @param {string[]} memoryIds - Array of memory IDs to delete
 * @returns {Promise<number>} Number of memories deleted
 */
export async function cleanupTestMemories(actor, memoryIds) {
  let deletedCount = 0;

  for (const memoryId of memoryIds) {
    if (await deleteTestMemory(actor, memoryId)) {
      deletedCount++;
    }
  }

  return deletedCount;
}

