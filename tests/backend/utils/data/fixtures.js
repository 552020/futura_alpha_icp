/**
 * Test Data Fixtures
 *
 * Provides pre-defined test data for consistent testing
 */

/**
 * Standard test capsule data
 */
export const TEST_CAPSULE_DATA = {
  self: {
    subject: null, // null for self-capsule
    description: "Test self-capsule",
  },
  other: {
    subject: "test-subject-123",
    description: "Test other-capsule",
  },
};

/**
 * Standard test memory data
 */
export const TEST_MEMORY_DATA = {
  inline: {
    name: "test_inline_memory",
    description: "Test memory with inline data",
    content: "This is test content for inline memory",
    tags: ["test", "inline"],
    mimeType: "text/plain",
  },
  blob: {
    name: "test_blob_memory",
    description: "Test memory with blob data",
    blobRef: "test-blob-ref-123",
    fileSize: 1024,
    tags: ["test", "blob"],
    mimeType: "application/octet-stream",
  },
  external: {
    name: "test_external_memory",
    description: "Test memory with external storage",
    storageType: "S3",
    storageKey: "test-bucket/test-file.jpg",
    url: "https://s3.amazonaws.com/test-bucket/test-file.jpg",
    fileSize: 2048,
    tags: ["test", "external"],
    mimeType: "image/jpeg",
  },
};

/**
 * Standard test asset data
 */
export const TEST_ASSET_DATA = {
  document: {
    type: "Document",
    name: "test_document.pdf",
    size: 1024,
    mimeType: "application/pdf",
    description: "Test document asset",
    tags: ["test", "document"],
  },
  image: {
    type: "Image",
    name: "test_image.jpg",
    size: 512,
    mimeType: "image/jpeg",
    description: "Test image asset",
    tags: ["test", "image"],
  },
  audio: {
    type: "Audio",
    name: "test_audio.mp3",
    size: 2048,
    mimeType: "audio/mpeg",
    description: "Test audio asset",
    tags: ["test", "audio"],
  },
  video: {
    type: "Video",
    name: "test_video.mp4",
    size: 4096,
    mimeType: "video/mp4",
    description: "Test video asset",
    tags: ["test", "video"],
  },
  note: {
    type: "Note",
    name: "test_note.txt",
    size: 256,
    mimeType: "text/plain",
    description: "Test note asset",
    tags: ["test", "note"],
  },
};

/**
 * Bulk operation test data
 */
export const BULK_TEST_DATA = {
  small: {
    memoryCount: 3,
    prefix: "bulk_small",
    description: "Small bulk test with 3 memories",
  },
  medium: {
    memoryCount: 10,
    prefix: "bulk_medium",
    description: "Medium bulk test with 10 memories",
  },
  large: {
    memoryCount: 50,
    prefix: "bulk_large",
    description: "Large bulk test with 50 memories",
  },
};

/**
 * Performance test data
 */
export const PERFORMANCE_TEST_DATA = {
  memorySizes: [100, 1024, 10240, 102400], // bytes
  memoryCounts: [1, 5, 10, 25, 50, 100],
  assetTypes: ["Document", "Image", "Audio", "Video", "Note"],
};

/**
 * Error test data
 */
export const ERROR_TEST_DATA = {
  invalidCapsuleId: "invalid-capsule-id-12345",
  invalidMemoryId: "invalid-memory-id-12345",
  invalidAssetRef: "invalid-asset-ref-12345",
  invalidBlobRef: "invalid-blob-ref-12345",
  invalidStorageKey: "invalid-storage-key-12345",
};

/**
 * Get test data by type
 * @param {string} type - Data type
 * @param {string} variant - Data variant
 * @returns {Object} Test data
 */
export function getTestData(type, variant = null) {
  switch (type) {
    case "capsule":
      return variant ? TEST_CAPSULE_DATA[variant] : TEST_CAPSULE_DATA;
    case "memory":
      return variant ? TEST_MEMORY_DATA[variant] : TEST_MEMORY_DATA;
    case "asset":
      return variant ? TEST_ASSET_DATA[variant] : TEST_ASSET_DATA;
    case "bulk":
      return variant ? BULK_TEST_DATA[variant] : BULK_TEST_DATA;
    case "performance":
      return PERFORMANCE_TEST_DATA;
    case "error":
      return ERROR_TEST_DATA;
    default:
      throw new Error(`Unknown test data type: ${type}`);
  }
}

/**
 * Generate test data with custom properties
 * @param {string} type - Data type
 * @param {Object} overrides - Property overrides
 * @returns {Object} Generated test data
 */
export function generateTestData(type, overrides = {}) {
  const baseData = getTestData(type);

  if (typeof baseData === "object" && !Array.isArray(baseData)) {
    return { ...baseData, ...overrides };
  }

  return baseData;
}

