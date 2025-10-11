/**
 * Asset Metadata Utilities
 *
 * Shared utilities for creating asset metadata structures used in upload tests.
 */

/**
 * Creates asset metadata for Document type assets
 * @param {string} fileName - Name of the file
 * @param {number} fileSize - Size of the file in bytes
 * @param {string} mimeType - MIME type of the file (default: "application/octet-stream")
 * @returns {Object} Document asset metadata structure
 */
export function createDocumentAssetMetadata(fileName, fileSize, mimeType = "application/octet-stream") {
  return {
    Document: {
      document_type: [],
      base: {
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000), // Convert to nanoseconds
        asset_type: { Original: null },
        sha256: [],
        name: fileName,
        storage_key: [],
        tags: ["upload-test", "file", `size-${fileSize}`],
        processing_error: [],
        mime_type: mimeType,
        description: [`Upload test file - ${fileSize} bytes`],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(fileSize),
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      language: [],
      page_count: [],
      word_count: [],
    },
  };
}

/**
 * Creates asset metadata for Image type assets
 * @param {string} fileName - Name of the file
 * @param {number} fileSize - Size of the file in bytes
 * @param {string} mimeType - MIME type of the file (default: "image/jpeg")
 * @returns {Object} Image asset metadata structure
 */
export function createImageAssetMetadata(fileName, fileSize, mimeType = "image/jpeg") {
  return {
    Image: {
      dpi: [],
      color_space: [],
      base: {
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000),
        asset_type: { Original: null },
        sha256: [],
        name: fileName,
        storage_key: [],
        tags: ["upload-test", "image", `size-${fileSize}`],
        processing_error: [],
        mime_type: mimeType,
        description: [`Upload test image - ${fileSize} bytes`],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(fileSize),
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      exif_data: [],
      compression_ratio: [],
      orientation: [],
    },
  };
}

/**
 * Creates asset metadata for Video type assets
 * @param {string} fileName - Name of the file
 * @param {number} fileSize - Size of the file in bytes
 * @param {string} mimeType - MIME type of the file (default: "video/mp4")
 * @returns {Object} Video asset metadata structure
 */
export function createVideoAssetMetadata(fileName, fileSize, mimeType = "video/mp4") {
  return {
    Video: {
      duration: [],
      fps: [],
      base: {
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000),
        asset_type: { Original: null },
        sha256: [],
        name: fileName,
        storage_key: [],
        tags: ["upload-test", "video", `size-${fileSize}`],
        processing_error: [],
        mime_type: mimeType,
        description: [`Upload test video - ${fileSize} bytes`],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(fileSize),
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      codec: [],
      bitrate: [],
    },
  };
}

/**
 * Creates asset metadata for Audio type assets
 * @param {string} fileName - Name of the file
 * @param {number} fileSize - Size of the file in bytes
 * @param {string} mimeType - MIME type of the file (default: "audio/mpeg")
 * @returns {Object} Audio asset metadata structure
 */
export function createAudioAssetMetadata(fileName, fileSize, mimeType = "audio/mpeg") {
  return {
    Audio: {
      duration: [],
      bitrate: [],
      base: {
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000),
        asset_type: { Original: null },
        sha256: [],
        name: fileName,
        storage_key: [],
        tags: ["upload-test", "audio", `size-${fileSize}`],
        processing_error: [],
        mime_type: mimeType,
        description: [`Upload test audio - ${fileSize} bytes`],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(fileSize),
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      sample_rate: [],
      channels: [],
    },
  };
}

