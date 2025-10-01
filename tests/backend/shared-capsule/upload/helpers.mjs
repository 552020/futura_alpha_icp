/**
 * Helper functions for ICP upload processing
 *
 * Adapted from frontend S3 processing functions for Node.js environment.
 * These functions provide the core processing logic without browser dependencies.
 */

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

// ============================================================================
// FILE VALIDATION HELPERS
// ============================================================================

/**
 * Validate file size against ICP limits
 * Adapted from frontend validateUploadFiles()
 */
export function validateFileSize(fileSize, limits = { maxFileSize: 100 * 1024 * 1024 }) {
  if (fileSize > limits.maxFileSize) {
    throw new Error(
      `File too large: ${(fileSize / (1024 * 1024)).toFixed(2)}MB exceeds ${(
        limits.maxFileSize /
        (1024 * 1024)
      ).toFixed(2)}MB limit`
    );
  }
  return true;
}

/**
 * Validate file type for image processing
 * Adapted from frontend supported formats check
 */
export function validateImageType(mimeType) {
  const supportedFormats = ["image/jpeg", "image/png", "image/webp"];
  if (!supportedFormats.includes(mimeType)) {
    throw new Error(`Unsupported image format: ${mimeType}. Supported: ${supportedFormats.join(", ")}`);
  }
  return true;
}

// ============================================================================
// FILE PROCESSING HELPERS
// ============================================================================

/**
 * Calculate file hash (SHA-256)
 * Adapted from frontend crypto operations
 */
export function calculateFileHash(fileBuffer) {
  return crypto.createHash("sha256").update(fileBuffer).digest();
}

/**
 * Generate unique file identifier
 * Adapted from frontend idempotency key generation
 */
export function generateFileId(prefix = "file") {
  const timestamp = Date.now();
  const random = Math.random().toString(36).substring(2);
  return `${prefix}-${timestamp}-${random}`;
}

/**
 * Extract file extension from filename
 * Adapted from frontend file handling
 */
export function getFileExtension(filename) {
  return path.extname(filename).toLowerCase();
}

/**
 * Generate derivative filename
 * Adapted from frontend asset naming convention
 */
export function generateDerivativeFilename(baseFilename, type) {
  const ext = getFileExtension(baseFilename);
  const baseName = path.basename(baseFilename, ext);
  return `${baseName}-${type}.webp`;
}

// ============================================================================
// IMAGE PROCESSING HELPERS
// ============================================================================

/**
 * Calculate optimal dimensions for image derivatives
 * Adapted from frontend image processing logic
 */
export function calculateDerivativeDimensions(originalWidth, originalHeight, maxWidth, maxHeight) {
  const aspectRatio = originalWidth / originalHeight;

  let width = originalWidth;
  let height = originalHeight;

  // Scale down if exceeds max dimensions
  if (width > maxWidth) {
    width = maxWidth;
    height = Math.round(width / aspectRatio);
  }

  if (height > maxHeight) {
    height = maxHeight;
    width = Math.round(height * aspectRatio);
  }

  return { width, height };
}

/**
 * Calculate derivative sizes for different asset types
 * Adapted from frontend size calculations
 */
export function calculateDerivativeSizes(originalSize) {
  return {
    display: {
      maxSize: 200 * 1024, // 200KB
      maxWidth: 1920,
      maxHeight: 1080,
      quality: 0.82,
    },
    thumb: {
      maxSize: 50 * 1024, // 50KB
      maxWidth: 300,
      maxHeight: 300,
      quality: 0.82,
    },
    placeholder: {
      maxSize: 2 * 1024, // 2KB
      maxWidth: 32,
      maxHeight: 18,
      quality: 0.6,
    },
  };
}

/**
 * Estimate file size from dimensions and quality
 * Adapted from frontend size estimation
 */
export function estimateFileSize(width, height, quality = 0.82) {
  // Rough estimation based on WebP compression
  const pixels = width * height;
  const bytesPerPixel = quality * 0.5; // WebP compression factor
  return Math.round(pixels * bytesPerPixel);
}

// ============================================================================
// UPLOAD HELPERS
// ============================================================================

/**
 * Calculate chunk count for file upload
 * Adapted from frontend chunking logic
 */
export function calculateChunkCount(fileSize, chunkSize = 1.5 * 1024 * 1024) {
  return Math.ceil(fileSize / chunkSize);
}

/**
 * Create file chunks for upload
 * Adapted from frontend chunking implementation
 */
export function createFileChunks(fileBuffer, chunkSize = 1.5 * 1024 * 1024) {
  const chunks = [];
  for (let i = 0; i < fileBuffer.length; i += chunkSize) {
    const chunk = fileBuffer.slice(i, i + chunkSize);
    chunks.push(Array.from(chunk));
  }
  return chunks;
}

/**
 * Generate upload progress callback
 * Adapted from frontend progress tracking
 */
export function createProgressCallback(totalChunks, onProgress) {
  let completedChunks = 0;

  return (chunkIndex) => {
    completedChunks++;
    const progress = Math.round((completedChunks / totalChunks) * 100);
    onProgress?.(progress, completedChunks, totalChunks);
  };
}

// ============================================================================
// ASSET METADATA HELPERS
// ============================================================================

/**
 * Create asset metadata for ICP backend
 * Adapted from frontend asset metadata structure
 */
export function createAssetMetadata(fileName, fileSize, mimeType, assetType = "Original") {
  const now = BigInt(Date.now() * 1000000);

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
        name: fileName,
        storage_key: [],
        tags: ["test", "2lane-4asset"],
        processing_error: [],
        mime_type: mimeType,
        description: [],
        created_at: now,
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
 * Create blob reference for ICP backend
 * Adapted from frontend blob reference structure
 */
export function createBlobReference(blobId, fileSize) {
  return {
    locator: blobId,
    len: BigInt(fileSize),
    hash: [],
  };
}

// ============================================================================
// ERROR HANDLING HELPERS
// ============================================================================

/**
 * Handle upload errors with detailed messages
 * Adapted from frontend error handling
 */
export function handleUploadError(error, context = "") {
  let message = "Upload failed";

  if (error instanceof Error) {
    if (error.message.includes("File too large")) {
      message = `File too large: ${error.message}`;
    } else if (error.message.includes("ResourceExhausted")) {
      message = `Resource exhausted: ${error.message}`;
    } else if (error.message.includes("canister_not_found")) {
      message = `Canister not found: ${error.message}`;
    } else if (error.message.includes("Invalid blob ID")) {
      message = `Invalid blob ID: ${error.message}`;
    } else {
      message = error.message;
    }
  }

  if (context) {
    message = `${context}: ${message}`;
  }

  return new Error(message);
}

/**
 * Validate upload response
 * Adapted from frontend response validation
 */
export function validateUploadResponse(response, expectedFields = []) {
  console.log(`🔍 validateUploadResponse called with:`, typeof response, response);

  if (!response) {
    throw new Error("Empty response received");
  }

  if ("Err" in response) {
    throw new Error(`Upload failed: ${JSON.stringify(response.Err)}`);
  }

  if (!("Ok" in response)) {
    throw new Error("Invalid response format");
  }

  // Validate expected fields (these should be in the response object, not response.Ok)
  for (const field of expectedFields) {
    if (!(field in response)) {
      throw new Error(`Missing field in response: ${field}`);
    }
  }

  return response.Ok;
}

// ============================================================================
// LOGGING HELPERS
// ============================================================================

/**
 * Format file size for logging
 * Adapted from frontend size formatting
 */
export function formatFileSize(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

/**
 * Format upload speed for logging
 * Adapted from frontend speed calculations
 */
export function formatUploadSpeed(bytes, durationMs) {
  const mbps = bytes / (1024 * 1024) / (durationMs / 1000);
  return `${mbps.toFixed(2)} MB/s`;
}

/**
 * Format duration for logging
 * Adapted from frontend timing calculations
 */
export function formatDuration(ms) {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${(ms / 60000).toFixed(1)}m`;
}

// ============================================================================
// UTILITY HELPERS
// ============================================================================

/**
 * Sleep function for testing delays
 * Adapted from frontend async utilities
 */
export function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Retry function with exponential backoff
 * Adapted from frontend retry logic
 */
export async function retryWithBackoff(fn, maxRetries = 3, baseDelay = 1000) {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      if (attempt === maxRetries) {
        throw error;
      }

      const delay = baseDelay * Math.pow(2, attempt - 1);
      await sleep(delay);
    }
  }
}

/**
 * Create timeout promise
 * Adapted from frontend timeout handling
 */
export function createTimeout(ms, message = "Operation timed out") {
  return new Promise((_, reject) => {
    setTimeout(() => reject(new Error(message)), ms);
  });
}

/**
 * Race between operation and timeout
 * Adapted from frontend timeout racing
 */
export async function withTimeout(promise, ms, message = "Operation timed out") {
  return Promise.race([promise, createTimeout(ms, message)]);
}
