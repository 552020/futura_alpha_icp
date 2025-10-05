/**
 * Asset Data Creation Utilities
 *
 * Provides utilities for creating and managing test assets
 */

/**
 * Create test asset data for different types
 * @param {string} assetType - Type of asset (inline, blob, external)
 * @param {Object} options - Asset options
 * @returns {Object} Asset data
 */
export function createTestAssetData(assetType, options = {}) {
  const { size = 100, name = "test_asset", mimeType = "text/plain" } = options;

  switch (assetType) {
    case "inline":
      return {
        type: "inline",
        data: `blob "${Buffer.from("A".repeat(size)).toString("base64")}"`,
        size: size,
        name: name,
        mimeType: mimeType,
      };

    case "blob":
      return {
        type: "blob",
        blobRef: {
          locator: `test_blob_${Date.now()}`,
          len: BigInt(size),
          hash: [],
        },
        size: size,
        name: name,
        mimeType: mimeType,
      };

    case "external":
      return {
        type: "external",
        storageKey: `s3://test-bucket/${name}`,
        url: `https://s3.amazonaws.com/test-bucket/${name}`,
        size: size,
        name: name,
        mimeType: mimeType,
      };

    default:
      throw new Error(`Unsupported asset type: ${assetType}`);
  }
}

/**
 * Create asset metadata for different types
 * @param {string} assetType - Type of asset
 * @param {Object} options - Asset options
 * @returns {Object} Asset metadata
 */
export function createAssetMetadata(assetType, options = {}) {
  const {
    name = "test_asset",
    description = "Test asset",
    tags = ["test"],
    size = 100,
    mimeType = "text/plain",
  } = options;

  const now = BigInt(Date.now() * 1000000);

  const baseMetadata = {
    name: name,
    description: [description],
    tags: tags,
    asset_type: { Original: null },
    bytes: BigInt(size),
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
    created_at: now,
    updated_at: now,
    deleted_at: [],
  };

  switch (assetType) {
    case "Document":
      return {
        Document: {
          base: baseMetadata,
          page_count: [],
          document_type: [],
          language: [],
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

    case "Audio":
      return {
        Audio: {
          base: baseMetadata,
          duration: [],
          bitrate: [],
          sample_rate: [],
          channels: [],
          format: [],
        },
      };

    case "Video":
      return {
        Video: {
          base: baseMetadata,
          duration: [],
          bitrate: [],
          frame_rate: [],
          resolution: [],
          format: [],
        },
      };

    case "Note":
      return {
        Note: {
          base: baseMetadata,
          language: [],
          word_count: [],
          format: [],
        },
      };

    default:
      throw new Error(`Unsupported asset type: ${assetType}`);
  }
}

/**
 * Create blob reference for ICP backend
 * @param {string} blobId - Blob ID
 * @param {number} fileSize - File size in bytes
 * @param {string} hash - Optional hash
 * @returns {Object} Blob reference
 */
export function createBlobReference(blobId, fileSize, hash = null) {
  return {
    locator: blobId,
    len: BigInt(fileSize),
    hash: hash ? [hash] : [],
  };
}

/**
 * Create storage edge blob type
 * @param {string} storageType - Storage type (S3, VercelBlob, etc.)
 * @returns {Object} Storage edge blob type
 */
export function createStorageEdgeBlobType(storageType) {
  return { [storageType]: null };
}

/**
 * Generate test asset references
 * @param {number} count - Number of assets to generate
 * @param {string} prefix - Asset prefix
 * @returns {string[]} Array of asset references
 */
export function generateTestAssetReferences(count, prefix = "test_asset") {
  return Array.from({ length: count }, (_, i) => `${prefix}_${i + 1}_${Date.now()}`);
}

/**
 * Create test asset with specific properties
 * @param {string} assetType - Type of asset
 * @param {Object} properties - Asset properties
 * @returns {Object} Test asset
 */
export function createTestAsset(assetType, properties = {}) {
  const {
    name = "test_asset",
    size = 100,
    mimeType = "text/plain",
    description = "Test asset",
    tags = ["test"],
  } = properties;

  return {
    type: assetType,
    name: name,
    size: size,
    mimeType: mimeType,
    description: description,
    tags: tags,
    metadata: createAssetMetadata(assetType, {
      name,
      size,
      mimeType,
      description,
      tags,
    }),
  };
}
