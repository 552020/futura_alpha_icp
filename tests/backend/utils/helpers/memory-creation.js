/**
 * =============================================================================
 * MEMORY CREATION HELPER FUNCTIONS
 * =============================================================================
 *
 * This module provides utilities for creating memory records in the ICP backend.
 * It supports both inline memory creation (for small files) and blob-based
 * memory creation (for files uploaded as separate blobs).
 *
 * ## MEMORY CREATION METHODS
 *
 * ### Inline Memory Creation
 * - `createMemoryWithInline()` - Creates a memory with file data embedded inline
 *   - Best for files ≤ 32KB
 *   - Single atomic operation (file data → memory creation)
 *   - No separate blob storage needed
 *
 * ### Blob-Based Memory Creation
 * - `createMemoryFromBlob()` - Creates memory referencing an existing blob
 *   - Used after blob upload when `createMemory: false`
 *   - Links memory record to previously uploaded blob
 *   - Supports both Document and Image memory types
 *
 * ## USAGE PATTERNS
 *
 * ### Inline Memory Creation:
 * ```javascript
 * const result = await createMemoryWithInline(backend, filePath, capsuleId);
 * // Memory is created with file data embedded inline
 * ```
 *
 * ### Blob-Based Memory Creation:
 * ```javascript
 * const blobResult = await uploadFileAsBlob(backend, filePath, capsuleId, { createMemory: false });
 * const memoryResult = await createMemoryFromBlob(backend, capsuleId, fileName, fileSize, blobResult.blobId);
 * ```
 *
 * ## DEPENDENCIES
 * - file-operations.js - File I/O, hashing
 * - asset-metadata.js - Asset metadata creation for different file types
 *
 * =============================================================================
 */

import path from "node:path";
import { readFileAsBuffer, computeSHA256Hash } from "./file-operations.js";
import { createDocumentAssetMetadata, createImageAssetMetadata } from "./asset-metadata.js";

/**
 * Creates a memory with inline file data (for small files)
 * @param {Object} backend - Backend actor
 * @param {string} filePath - Path to the file to embed inline
 * @param {string} capsuleId - Capsule ID
 * @param {Object} options - Memory creation options
 * @returns {Promise<{success: boolean, memoryId?: string, error?: string}>}
 */
export async function createMemoryWithInline(backend, filePath, capsuleId, options = {}) {
  const fileName = path.basename(filePath);
  const fileBuffer = readFileAsBuffer(filePath);
  const fileSize = fileBuffer.length;

  console.log(`📄 Creating memory with inline data for ${fileName} (${fileSize} bytes)`);

  try {
    // Create asset metadata based on file type
    const assetMetadata =
      options.assetType === "image"
        ? createImageAssetMetadata(fileName, fileSize, options.mimeType)
        : createDocumentAssetMetadata(fileName, fileSize, options.mimeType);

    const idempotencyKey = options.idempotencyKey || `test_inline_${Date.now()}`;

    // Compute SHA256 hash
    const fileHash = computeSHA256Hash(fileBuffer);
    const hashBuffer = Buffer.from(fileHash, "hex");

    const result = await backend.memories_create(
      capsuleId,
      [new Uint8Array(fileBuffer)], // opt blob - inline data
      [], // opt BlobRef - no blob reference for inline
      [], // opt StorageEdgeBlobType - no storage edge for inline
      [], // opt text - no storage key for inline
      [], // opt text - no bucket for inline
      [], // opt nat64 - no file_created_at
      [new Uint8Array(hashBuffer)], // opt blob - sha256 hash
      assetMetadata,
      idempotencyKey
    );

    if (!("Ok" in result)) {
      console.error("❌ Memory creation with inline data failed:", result);
      return { success: false, error: "memories_create failed: " + JSON.stringify(result) };
    }

    const memoryId = result.Ok;
    console.log(`✅ Memory created with inline data - Memory ID: ${memoryId}`);
    return { success: true, memoryId };
  } catch (error) {
    console.error("❌ Memory creation with inline data error:", error.message);
    return { success: false, error: error.message };
  }
}

/**
 * Creates a memory record referencing an existing blob
 * @param {Object} backend - Backend actor
 * @param {string} capsuleId - Capsule ID
 * @param {string} fileName - Name of the file
 * @param {number} fileSize - Size of the file in bytes
 * @param {string} blobId - ID of the uploaded blob
 * @param {Object} uploadResult - Result from blob upload
 * @param {Object} options - Memory creation options
 * @returns {Promise<{success: boolean, memoryId?: string, error?: string}>}
 */
export async function createMemoryFromBlob(backend, capsuleId, fileName, fileSize, blobId, uploadResult, options = {}) {
  console.log(`📝 Creating memory from blob ${blobId} for ${fileName}`);

  try {
    // Create asset metadata based on file type
    const assetMetadata =
      options.assetType === "image"
        ? createImageAssetMetadata(fileName, fileSize, options.mimeType)
        : createDocumentAssetMetadata(fileName, fileSize, options.mimeType);

    const idempotencyKey = options.idempotencyKey || `test_blob_${Date.now()}`;

    // Create memory metadata
    const memoryMetadata = {
      memory_type: options.memoryType || { Document: null },
      title: [fileName], // opt text
      description: [`Memory created from blob upload - ${fileName}`],
      content_type: options.mimeType || "application/octet-stream",
      created_at: BigInt(Date.now() * 1000000), // nanoseconds
      updated_at: BigInt(Date.now() * 1000000),
      uploaded_at: BigInt(Date.now() * 1000000),
      date_of_memory: [],
      file_created_at: [],
      parent_folder_id: [],
      tags: ["test", "blob-upload"],
      deleted_at: [],
      people_in_memory: [],
      location: [],
      memory_notes: [],
      created_by: [],
      database_storage_edges: [],
      sharing_status: { Private: null },
      has_thumbnails: false,
      has_previews: false,
      total_size: BigInt(fileSize),
      thumbnail_url: [],
      asset_count: 1,
      primary_asset_url: [],
      shared_count: 0,
    };

    // Create internal blob asset
    const internalBlobAsset = {
      blob_id: blobId,
      metadata: assetMetadata,
    };

    const result = await backend.memories_create_with_internal_blobs(
      capsuleId,
      memoryMetadata,
      [internalBlobAsset],
      idempotencyKey
    );

    if (!("Ok" in result)) {
      console.error("❌ Memory creation from blob failed:", result);
      return { success: false, error: "memories_create_with_internal_blobs failed: " + JSON.stringify(result) };
    }

    const memoryId = result.Ok;
    console.log(`✅ Memory created from blob - Memory ID: ${memoryId}`);
    return { success: true, memoryId };
  } catch (error) {
    console.error("❌ Memory creation from blob error:", error.message);
    return { success: false, error: error.message };
  }
}
