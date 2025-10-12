/**
 * Asset Metadata Utilities
 *
 * Shared utilities for creating asset metadata structures used in upload tests.
 */

/**
 * Creates default memory metadata structure
 * @param {Object} options - Memory metadata options
 * @param {string} options.title - Memory title
 * @param {string} options.description - Memory description
 * @param {string[]} options.tags - Memory tags
 * @param {string} options.contentType - Content type (default: "image/jpeg")
 * @param {Object} options.memoryType - Memory type (default: { Image: null })
 * @returns {Object} Memory metadata structure
 */
export function createMemoryMetadata(options = {}) {
  const {
    title = "Test Memory",
    description = "Test memory created by upload system",
    tags = ["test"],
    contentType = "image/jpeg",
    memoryType = { Image: null },
  } = options;

  const now = BigInt(Date.now() * 1000000); // Convert to nanoseconds

  return {
    title: [title], // opt text - wrapped in array for Some(value)
    description: [description], // opt text
    tags,
    created_at: now,
    updated_at: now,
    date_of_memory: [],
    memory_type: memoryType,
    content_type: contentType,
    people_in_memory: [],
    database_storage_edges: [],
    created_by: [],
    parent_folder_id: [],
    deleted_at: [],
    file_created_at: [],
    location: [],
    memory_notes: [],
    uploaded_at: now,
    sharing_status: { Private: null },
    has_thumbnails: false,
    has_previews: false,
    total_size: BigInt(0), // Will be updated with actual size
    thumbnail_url: [],
    asset_count: 0, // Will be updated with actual count
    primary_asset_url: [],
    shared_count: 0,
  };
}

/**
 * Creates default image asset metadata structure
 * @param {Object} options - Asset metadata options
 * @param {string} options.name - Asset name
 * @param {number} options.size - Asset size in bytes
 * @param {string} options.mimeType - MIME type (default: "image/jpeg")
 * @param {string} options.assetType - Asset type: "Original" or "Derivative" (default: "Original")
 * @param {string[]} options.tags - Asset tags
 * @param {string} options.description - Asset description
 * @returns {Object} Image asset metadata structure
 */
export function createImageAssetMetadata(options = {}) {
  const {
    name = "asset",
    size = 0,
    mimeType = "image/jpeg",
    assetType = "Original",
    tags = ["test"],
    description = "Test asset",
  } = options;

  const now = BigInt(Date.now() * 1000000); // Convert to nanoseconds

  return {
    Image: {
      dpi: [],
      color_space: [],
      base: {
        url: [],
        height: [],
        updated_at: now,
        asset_type: { [assetType]: null },
        sha256: [],
        name,
        storage_key: [],
        tags,
        processing_error: [],
        mime_type: mimeType,
        description: [description],
        created_at: now,
        deleted_at: [],
        bytes: BigInt(size),
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
 * Creates derivative asset metadata (convenience function)
 * @param {Object} options - Asset metadata options
 * @param {string} options.name - Asset name (default: "derivative")
 * @param {number} options.size - Asset size in bytes (default: 0)
 * @param {string} options.mimeType - MIME type (default: "image/webp")
 * @param {string[]} options.tags - Asset tags (default: ["test", "derivative"])
 * @param {string} options.description - Asset description (default: "Derivative asset")
 * @returns {Object} Derivative image asset metadata structure
 */
export function createDerivativeAssetMetadata(options = {}) {
  return createImageAssetMetadata({
    name: "derivative",
    size: 0,
    mimeType: "image/webp",
    assetType: "Derivative",
    tags: ["test", "derivative"],
    description: "Derivative asset",
    ...options,
  });
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

