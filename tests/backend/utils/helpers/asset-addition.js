//! Asset addition helper functions for testing
//!
//! This module provides helper functions for adding assets to existing memories
//! using the new memories_add_asset and memories_add_inline_asset backend functions.

import { createImageAssetMetadata } from './asset-metadata.js';

/**
 * Add a blob asset to an existing memory
 * 
 * @param {Object} backend - The backend actor
 * @param {string} memoryId - The memory ID to add the asset to
 * @param {string} blobId - The blob ID to add
 * @param {Object} options - Additional options
 * @param {string} options.assetType - Type of asset (e.g., 'display', 'thumb', 'placeholder')
 * @param {string} options.mimeType - MIME type of the asset
 * @param {string} options.idempotencyKey - Idempotency key for the operation
 * @returns {Promise<Object>} Result object with success status and asset ID
 */
export async function addAssetToMemory(backend, memoryId, blobId, options = {}) {
  const {
    assetType = 'display',
    mimeType = 'image/webp',
    idempotencyKey = `asset-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
  } = options;

  try {
    // Create asset metadata
    const assetMetadata = createImageAssetMetadata(
      `${assetType}.webp`,
      0, // Size will be updated from blob metadata
      mimeType
    );

    // Create the asset input
    const asset = {
      blob_id: blobId,
      metadata: assetMetadata
    };

    // Call the backend function
    const result = await backend.memories_add_asset(memoryId, asset, idempotencyKey);

    if ("Err" in result) {
      return {
        success: false,
        error: `Failed to add asset: ${JSON.stringify(result.Err)}`
      };
    }

    return {
      success: true,
      assetId: result.Ok
    };
  } catch (error) {
    return {
      success: false,
      error: `Exception adding asset: ${error.message}`
    };
  }
}

/**
 * Add an inline asset to an existing memory
 * 
 * @param {Object} backend - The backend actor
 * @param {string} memoryId - The memory ID to add the asset to
 * @param {Uint8Array} bytes - The asset data
 * @param {Object} options - Additional options
 * @param {string} options.assetType - Type of asset (e.g., 'display', 'thumb', 'placeholder')
 * @param {string} options.mimeType - MIME type of the asset
 * @param {string} options.idempotencyKey - Idempotency key for the operation
 * @returns {Promise<Object>} Result object with success status and asset ID
 */
export async function addInlineAssetToMemory(backend, memoryId, bytes, options = {}) {
  const {
    assetType = 'placeholder',
    mimeType = 'image/jpeg',
    idempotencyKey = `inline-asset-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
  } = options;

  try {
    // Create asset metadata
    const assetMetadata = createImageAssetMetadata(
      `${assetType}.jpg`,
      bytes.length,
      mimeType
    );

    // Create the asset input
    const asset = {
      bytes: Array.from(bytes), // Convert Uint8Array to regular array for Candid
      metadata: assetMetadata
    };

    // Call the backend function
    const result = await backend.memories_add_inline_asset(memoryId, asset, idempotencyKey);

    if ("Err" in result) {
      return {
        success: false,
        error: `Failed to add inline asset: ${JSON.stringify(result.Err)}`
      };
    }

    return {
      success: true,
      assetId: result.Ok
    };
  } catch (error) {
    return {
      success: false,
      error: `Exception adding inline asset: ${error.message}`
    };
  }
}

/**
 * Add multiple blob assets to an existing memory
 * 
 * @param {Object} backend - The backend actor
 * @param {string} memoryId - The memory ID to add assets to
 * @param {Array} assets - Array of asset objects with blobId and options
 * @returns {Promise<Object>} Result object with success status and asset IDs
 */
export async function addMultipleAssetsToMemory(backend, memoryId, assets) {
  const results = [];
  const errors = [];

  for (const asset of assets) {
    const result = await addAssetToMemory(backend, memoryId, asset.blobId, asset.options);
    results.push(result);
    
    if (!result.success) {
      errors.push(result.error);
    }
  }

  return {
    success: errors.length === 0,
    assetIds: results.filter(r => r.success).map(r => r.assetId),
    errors: errors
  };
}