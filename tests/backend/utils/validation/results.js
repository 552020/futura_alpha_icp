/**
 * Result Validation Utilities
 *
 * Provides utilities for validating API responses and results
 */

/**
 * Validate bulk delete result
 * @param {Object} result - API result
 * @param {number} expectedDeleted - Expected deleted count
 * @param {number} expectedFailed - Expected failed count
 * @returns {Object} Validation result
 */
export function validateBulkDeleteResult(result, expectedDeleted, expectedFailed = 0) {
  if (!result.Ok) {
    return {
      valid: false,
      error: `Expected Ok result, got: ${JSON.stringify(result)}`,
    };
  }

  const bulkResult = result.Ok;

  if (bulkResult.deleted_count !== expectedDeleted) {
    return {
      valid: false,
      error: `Expected ${expectedDeleted} deleted, got ${bulkResult.deleted_count}`,
    };
  }

  if (bulkResult.failed_count !== expectedFailed) {
    return {
      valid: false,
      error: `Expected ${expectedFailed} failed, got ${bulkResult.failed_count}`,
    };
  }

  return { valid: true };
}

/**
 * Validate asset cleanup result
 * @param {Object} result - API result
 * @param {number} expectedCleaned - Expected cleaned count
 * @returns {Object} Validation result
 */
export function validateAssetCleanupResult(result, expectedCleaned) {
  if (!result.Ok) {
    return {
      valid: false,
      error: `Expected Ok result, got: ${JSON.stringify(result)}`,
    };
  }

  const cleanupResult = result.Ok;

  if (cleanupResult.assets_cleaned !== expectedCleaned) {
    return {
      valid: false,
      error: `Expected ${expectedCleaned} assets cleaned, got ${cleanupResult.assets_cleaned}`,
    };
  }

  return { valid: true };
}

/**
 * Validate asset removal result
 * @param {Object} result - API result
 * @param {boolean} expectedRemoved - Expected removal status
 * @returns {Object} Validation result
 */
export function validateAssetRemovalResult(result, expectedRemoved) {
  if (!result.Ok) {
    return {
      valid: false,
      error: `Expected Ok result, got: ${JSON.stringify(result)}`,
    };
  }

  const removalResult = result.Ok;

  if (removalResult.asset_removed !== expectedRemoved) {
    return {
      valid: false,
      error: `Expected asset_removed=${expectedRemoved}, got ${removalResult.asset_removed}`,
    };
  }

  return { valid: true };
}

/**
 * Validate bulk asset cleanup result
 * @param {Object} result - API result
 * @param {number} expectedCleaned - Expected cleaned count
 * @returns {Object} Validation result
 */
export function validateBulkAssetCleanupResult(result, expectedCleaned) {
  if (!result.Ok) {
    return {
      valid: false,
      error: `Expected Ok result, got: ${JSON.stringify(result)}`,
    };
  }

  const cleanupResult = result.Ok;

  if (cleanupResult.total_assets_cleaned !== expectedCleaned) {
    return {
      valid: false,
      error: `Expected ${expectedCleaned} total assets cleaned, got ${cleanupResult.total_assets_cleaned}`,
    };
  }

  return { valid: true };
}

/**
 * Validate memory assets list result
 * @param {Object} result - API result
 * @param {number} expectedCount - Expected asset count
 * @returns {Object} Validation result
 */
export function validateMemoryAssetsListResult(result, expectedCount) {
  if (!result.Ok) {
    return {
      valid: false,
      error: `Expected Ok result, got: ${JSON.stringify(result)}`,
    };
  }

  const assetsResult = result.Ok;

  if (assetsResult.total_count !== expectedCount) {
    return {
      valid: false,
      error: `Expected ${expectedCount} total assets, got ${assetsResult.total_count}`,
    };
  }

  return { valid: true };
}

/**
 * Validate generic API result
 * @param {Object} result - API result
 * @param {string} expectedType - Expected result type (Ok, Err)
 * @returns {Object} Validation result
 */
export function validateApiResult(result, expectedType = "Ok") {
  if (!(expectedType in result)) {
    return {
      valid: false,
      error: `Expected ${expectedType} result, got: ${JSON.stringify(result)}`,
    };
  }

  return { valid: true };
}

/**
 * Validate error result
 * @param {Object} result - API result
 * @param {string} expectedError - Expected error type
 * @returns {Object} Validation result
 */
export function validateErrorResult(result, expectedError = null) {
  if (!result.Err) {
    return {
      valid: false,
      error: `Expected Err result, got: ${JSON.stringify(result)}`,
    };
  }

  if (expectedError && result.Err !== expectedError) {
    return {
      valid: false,
      error: `Expected error ${expectedError}, got ${result.Err}`,
    };
  }

  return { valid: true };
}

/**
 * Validate success result
 * @param {Object} result - API result
 * @param {string} expectedField - Expected field in result
 * @returns {Object} Validation result
 */
export function validateSuccessResult(result, expectedField = null) {
  if (!result.Ok) {
    return {
      valid: false,
      error: `Expected Ok result, got: ${JSON.stringify(result)}`,
    };
  }

  if (expectedField && !(expectedField in result.Ok)) {
    return {
      valid: false,
      error: `Expected field ${expectedField} in result, got: ${JSON.stringify(result.Ok)}`,
    };
  }

  return { valid: true };
}
