/**
 * Error Handling Utilities
 *
 * Provides utilities for handling and validating errors
 */

/**
 * Handle upload errors with detailed messages
 * @param {Error} error - Error object
 * @param {string} context - Error context
 * @returns {Error} Enhanced error with context
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
 * Handle API errors with detailed messages
 * @param {Error} error - Error object
 * @param {string} context - Error context
 * @returns {Error} Enhanced error with context
 */
export function handleApiError(error, context = "") {
  let message = "API call failed";

  if (error instanceof Error) {
    if (error.message.includes("Certificate verification")) {
      message = `Certificate verification failed: ${error.message}`;
    } else if (error.message.includes("Connection refused")) {
      message = `Connection refused: ${error.message}`;
    } else if (error.message.includes("Timeout")) {
      message = `Request timeout: ${error.message}`;
    } else if (error.message.includes("NotFound")) {
      message = `Resource not found: ${error.message}`;
    } else if (error.message.includes("Unauthorized")) {
      message = `Unauthorized access: ${error.message}`;
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
 * Handle bulk operation errors
 * @param {Error} error - Error object
 * @param {string} operation - Operation name
 * @param {number} itemCount - Number of items
 * @returns {Error} Enhanced error with context
 */
export function handleBulkOperationError(error, operation, itemCount) {
  let message = `${operation} failed`;

  if (error instanceof Error) {
    if (error.message.includes("Partial failure")) {
      message = `${operation} partially failed for ${itemCount} items: ${error.message}`;
    } else if (error.message.includes("Complete failure")) {
      message = `${operation} completely failed for ${itemCount} items: ${error.message}`;
    } else {
      message = `${operation} failed for ${itemCount} items: ${error.message}`;
    }
  }

  return new Error(message);
}

/**
 * Validate upload response
 * @param {Object} response - API response
 * @param {string[]} expectedFields - Expected fields in response
 * @returns {Object} Validated response
 */
export function validateUploadResponse(response, expectedFields = []) {
  if (!response) {
    throw new Error("Empty response received");
  }

  if ("Err" in response) {
    throw new Error(`Upload failed: ${JSON.stringify(response.Err)}`);
  }

  if (!("Ok" in response)) {
    throw new Error("Invalid response format");
  }

  // Validate expected fields
  for (const field of expectedFields) {
    if (!(field in response)) {
      throw new Error(`Missing field in response: ${field}`);
    }
  }

  return response.Ok;
}

/**
 * Validate API response
 * @param {Object} response - API response
 * @param {string} expectedType - Expected response type (Ok, Err)
 * @param {string[]} expectedFields - Expected fields in response
 * @returns {Object} Validated response
 */
export function validateApiResponse(response, expectedType = "Ok", expectedFields = []) {
  if (!response) {
    throw new Error("Empty response received");
  }

  if (!(expectedType in response)) {
    throw new Error(`Expected ${expectedType} response, got: ${JSON.stringify(response)}`);
  }

  if (expectedType === "Ok" && expectedFields.length > 0) {
    for (const field of expectedFields) {
      if (!(field in response.Ok)) {
        throw new Error(`Missing field in response: ${field}`);
      }
    }
  }

  return response[expectedType];
}

/**
 * Classify error type
 * @param {Error} error - Error object
 * @returns {string} Error type
 */
export function classifyError(error) {
  if (error instanceof Error) {
    if (error.message.includes("Certificate verification")) {
      return "certificate";
    } else if (error.message.includes("Connection refused") || error.message.includes("ECONNREFUSED")) {
      return "connection";
    } else if (error.message.includes("Timeout") || error.message.includes("ETIMEDOUT")) {
      return "timeout";
    } else if (error.message.includes("NotFound")) {
      return "not_found";
    } else if (error.message.includes("Unauthorized")) {
      return "unauthorized";
    } else if (error.message.includes("Invalid argument")) {
      return "invalid_argument";
    } else if (error.message.includes("Resource exhausted")) {
      return "resource_exhausted";
    } else {
      return "unknown";
    }
  }
  return "unknown";
}

/**
 * Get error message for user display
 * @param {Error} error - Error object
 * @returns {string} User-friendly error message
 */
export function getUserErrorMessage(error) {
  const errorType = classifyError(error);

  switch (errorType) {
    case "certificate":
      return "Certificate verification failed. Please check your ICP connection.";
    case "connection":
      return "Connection failed. Please check your network connection.";
    case "timeout":
      return "Request timed out. Please try again.";
    case "not_found":
      return "Resource not found.";
    case "unauthorized":
      return "Unauthorized access. Please check your authentication.";
    case "invalid_argument":
      return "Invalid argument provided.";
    case "resource_exhausted":
      return "Resource exhausted. Please try again later.";
    default:
      return error.message || "An unknown error occurred.";
  }
}

/**
 * Retry function with exponential backoff
 * @param {Function} fn - Function to retry
 * @param {number} maxRetries - Maximum number of retries
 * @param {number} baseDelay - Base delay in milliseconds
 * @returns {Promise<any>} Function result
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
      await new Promise((resolve) => setTimeout(resolve, delay));
    }
  }
}

