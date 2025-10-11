/**
 * Image Processing Helpers for ICP Backend Tests
 *
 * This module provides image processing utilities for testing the 2-lane + 4-asset system.
 * It simulates the frontend image processing logic for creating derivatives (display, thumb, placeholder).
 *
 * In a real implementation, these would use Sharp or Jimp for actual image processing.
 * For testing purposes, we create simulated derivative buffers.
 */

import { uploadBufferAsBlob } from "./upload-download.js";

import { formatFileSize } from "./logging.js";

/**
 * Validates that the file type is supported for image processing
 * @param {string} mimeType - The MIME type to validate
 * @throws {Error} If the MIME type is not supported
 */
export function validateImageType(mimeType) {
  const supportedTypes = ["image/jpeg", "image/jpg", "image/png", "image/webp"];
  if (!supportedTypes.includes(mimeType.toLowerCase())) {
    throw new Error(`Unsupported image type: ${mimeType}. Supported types: ${supportedTypes.join(", ")}`);
  }
}

/**
 * Calculates size limits for derivative assets based on original file size
 * @param {number} originalSize - Size of the original file in bytes
 * @returns {Object} Size limits for each derivative type
 */
export function calculateDerivativeSizes(originalSize) {
  // Size limits based on frontend S3 system
  const sizeLimits = {
    display: {
      maxSize: Math.min(2 * 1024 * 1024, Math.floor(originalSize * 0.3)), // Max 2MB or 30% of original
      maxWidth: 1920,
      maxHeight: 1080,
    },
    thumb: {
      maxSize: Math.min(200 * 1024, Math.floor(originalSize * 0.1)), // Max 200KB or 10% of original
      maxWidth: 400,
      maxHeight: 300,
    },
    placeholder: {
      maxSize: 10 * 1024, // Max 10KB
      maxWidth: 32,
      maxHeight: 18,
    },
  };

  return sizeLimits;
}

/**
 * Calculates derivative dimensions maintaining aspect ratio
 * @param {number} originalWidth - Original image width
 * @param {number} originalHeight - Original image height
 * @param {number} maxWidth - Maximum allowed width
 * @param {number} maxHeight - Maximum allowed height
 * @returns {Object} Calculated width and height
 */
export function calculateDerivativeDimensions(originalWidth, originalHeight, maxWidth, maxHeight) {
  const aspectRatio = originalWidth / originalHeight;

  let width = originalWidth;
  let height = originalHeight;

  // Scale down if too wide
  if (width > maxWidth) {
    width = maxWidth;
    height = Math.floor(width / aspectRatio);
  }

  // Scale down if too tall
  if (height > maxHeight) {
    height = maxHeight;
    width = Math.floor(height * aspectRatio);
  }

  return { width, height };
}

/**
 * Processes image derivatives (display, thumb, placeholder) from original file
 * This simulates the frontend image processing logic for testing purposes.
 *
 * @param {Buffer} fileBuffer - Original image file buffer
 * @param {string} mimeType - MIME type of the original image
 * @returns {Object} Processed derivatives with buffers and metadata
 */
export async function processImageDerivativesPure(fileBuffer, mimeType) {
  const originalSize = fileBuffer.length;

  console.log(`🖼️ Processing derivatives for ${formatFileSize(originalSize)} file`);

  // Validate file type
  validateImageType(mimeType);

  // Get derivative size limits
  const sizeLimits = calculateDerivativeSizes(originalSize);

  // Calculate realistic dimensions based on file size
  const aspectRatio = 16 / 9;
  const originalWidth = Math.floor(Math.sqrt(originalSize / 3));
  const originalHeight = Math.floor(originalWidth / aspectRatio);

  // Calculate derivative dimensions
  const displayDims = calculateDerivativeDimensions(
    originalWidth,
    originalHeight,
    sizeLimits.display.maxWidth,
    sizeLimits.display.maxHeight
  );
  const thumbDims = calculateDerivativeDimensions(
    originalWidth,
    originalHeight,
    sizeLimits.thumb.maxWidth,
    sizeLimits.thumb.maxHeight
  );

  // Create derivative buffers (simulation - in real implementation, use Sharp/Jimp)
  const displaySize = Math.min(sizeLimits.display.maxSize, Math.floor(originalSize * 0.1));
  const displayBuffer = Buffer.alloc(displaySize);
  fileBuffer.copy(displayBuffer, 0, 0, displaySize);

  const thumbSize = Math.min(sizeLimits.thumb.maxSize, Math.floor(originalSize * 0.05));
  const thumbBuffer = Buffer.alloc(thumbSize);
  fileBuffer.copy(thumbBuffer, 0, 0, thumbSize);

  const placeholderSize = Math.min(sizeLimits.placeholder.maxSize, 1024);
  const placeholderBuffer = Buffer.alloc(placeholderSize, 0x42);

  // Log precise sizes
  console.log(`📊 Derivative sizes:`);
  console.log(`  Display: ${formatFileSize(displaySize)} (${displayDims.width}x${displayDims.height})`);
  console.log(`  Thumb: ${formatFileSize(thumbSize)} (${thumbDims.width}x${thumbDims.height})`);
  console.log(`  Placeholder: ${formatFileSize(placeholderSize)} (32x18)`);

  return {
    original: {
      buffer: fileBuffer,
      size: originalSize,
      width: originalWidth,
      height: originalHeight,
      mimeType: mimeType,
    },
    display: {
      buffer: displayBuffer,
      size: displaySize,
      width: displayDims.width,
      height: displayDims.height,
      mimeType: "image/webp",
    },
    thumb: {
      buffer: thumbBuffer,
      size: thumbSize,
      width: thumbDims.width,
      height: thumbDims.height,
      mimeType: "image/webp",
    },
    placeholder: {
      buffer: placeholderBuffer,
      size: placeholderSize,
      width: 32,
      height: 18,
      mimeType: "image/webp",
    },
  };
}

/**
 * Process image derivatives and upload them to ICP
 * @param {Object} backend - Backend actor
 * @param {Object} derivatives - Derivatives object from processImageDerivativesPure
 * @param {string} capsuleId - Capsule ID
 * @param {Object} options - Upload options
 * @returns {Promise<{success: boolean, data?: {blobIds: Object}, error?: string}>}
 */
export async function processImageDerivativesToICP(backend, derivatives, capsuleId, options = {}) {
  console.log(`📤 Uploading derivatives to ICP...`);

  try {
    const results = {};
    const uploadPromises = [];

    // Upload display derivative
    if (derivatives.display) {
      console.log(`📤 Uploading display derivative...`);
      uploadPromises.push(
        uploadBufferAsBlob(backend, derivatives.display.buffer, capsuleId, {
          createMemory: false,
          idempotencyKey: options.idempotencyKey ? `${options.idempotencyKey}-display` : `display-${Date.now()}`,
        }).then((uploadResult) => {
          if (uploadResult.success) {
            results.display = uploadResult.blobId;
          } else {
            throw new Error(`Display upload failed: ${uploadResult.error}`);
          }
        })
      );
    }

    // Upload thumb derivative
    if (derivatives.thumb) {
      console.log(`📤 Uploading thumb derivative...`);
      uploadPromises.push(
        uploadBufferAsBlob(backend, derivatives.thumb.buffer, capsuleId, {
          createMemory: false,
          idempotencyKey: options.idempotencyKey ? `${options.idempotencyKey}-thumb` : `thumb-${Date.now()}`,
        }).then((uploadResult) => {
          if (uploadResult.success) {
            results.thumb = uploadResult.blobId;
          } else {
            throw new Error(`Thumb upload failed: ${uploadResult.error}`);
          }
        })
      );
    }

    // Upload placeholder derivative
    if (derivatives.placeholder) {
      console.log(`📤 Uploading placeholder derivative...`);
      uploadPromises.push(
        uploadBufferAsBlob(backend, derivatives.placeholder.buffer, capsuleId, {
          createMemory: false,
          idempotencyKey: options.idempotencyKey ? `${options.idempotencyKey}-placeholder` : `placeholder-${Date.now()}`,
        }).then((uploadResult) => {
          if (uploadResult.success) {
            results.placeholder = uploadResult.blobId;
          } else {
            throw new Error(`Placeholder upload failed: ${uploadResult.error}`);
          }
        })
      );
    }

    // Wait for all uploads to complete
    await Promise.all(uploadPromises);

    console.log(`✅ All derivatives uploaded successfully:`, Object.keys(results));
    return {
      success: true,
      data: { blobIds: results }
    };

  } catch (error) {
    console.error(`❌ Derivative upload failed: ${error.message}`);
    return {
      success: false,
      error: error.message
    };
  }
}

