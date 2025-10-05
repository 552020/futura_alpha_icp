/**
 * Logging Utilities
 *
 * Provides standardized logging for tests
 */

// Colors for console output
const colors = {
  reset: "\x1b[0m",
  red: "\x1b[31m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  magenta: "\x1b[35m",
  cyan: "\x1b[36m",
  white: "\x1b[37m",
  bold: "\x1b[1m",
};

/**
 * Log a message with color
 * @param {string} message - Message to log
 * @param {string} color - Color name
 */
export function log(message, color = "white") {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

/**
 * Log a header message
 * @param {string} message - Header message
 */
export function logHeader(message) {
  log("\n" + "=".repeat(60), "cyan");
  log(message, "cyan");
  log("=".repeat(60), "cyan");
}

/**
 * Log a success message
 * @param {string} message - Success message
 */
export function logSuccess(message) {
  log(`✅ ${message}`, "green");
}

/**
 * Log an error message
 * @param {string} message - Error message
 */
export function logError(message) {
  log(`❌ ${message}`, "red");
}

/**
 * Log an info message
 * @param {string} message - Info message
 */
export function logInfo(message) {
  log(`ℹ️  ${message}`, "blue");
}

/**
 * Log a warning message
 * @param {string} message - Warning message
 */
export function logWarning(message) {
  log(`⚠️  ${message}`, "yellow");
}

/**
 * Log a debug message
 * @param {string} message - Debug message
 */
export function logDebug(message) {
  if (process.env.DEBUG === "true") {
    log(`[DEBUG] ${message}`, "yellow");
  }
}

/**
 * Log test results
 * @param {string} testName - Test name
 * @param {boolean} passed - Test result
 * @param {string} message - Additional message
 */
export function logTestResult(testName, passed, message = "") {
  if (passed) {
    logSuccess(`${testName} - PASSED${message ? `: ${message}` : ""}`);
  } else {
    logError(`${testName} - FAILED${message ? `: ${message}` : ""}`);
  }
}

/**
 * Log test summary
 * @param {number} totalTests - Total number of tests
 * @param {number} passedTests - Number of passed tests
 * @param {number} failedTests - Number of failed tests
 */
export function logTestSummary(totalTests, passedTests, failedTests) {
  logHeader("Test Results Summary");
  logInfo(`Total tests: ${totalTests}`);
  logSuccess(`Passed: ${passedTests}`);

  if (failedTests > 0) {
    logError(`Failed: ${failedTests}`);
  }

  if (failedTests === 0) {
    logSuccess("🎉 All tests passed!");
  } else {
    logError(`❌ ${failedTests} test(s) failed`);
  }
}

/**
 * Log performance metrics
 * @param {string} operation - Operation name
 * @param {number} duration - Duration in milliseconds
 * @param {number} itemCount - Number of items processed
 */
export function logPerformance(operation, duration, itemCount = 1) {
  const durationStr = formatDuration(duration);
  const itemsPerSecond = Math.round((itemCount / duration) * 1000);

  logInfo(`${operation}: ${durationStr} (${itemsPerSecond} items/sec)`);
}

/**
 * Format file size for logging
 * @param {number} bytes - File size in bytes
 * @returns {string} Formatted file size
 */
export function formatFileSize(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

/**
 * Format duration for logging
 * @param {number} ms - Duration in milliseconds
 * @returns {string} Formatted duration
 */
export function formatDuration(ms) {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${(ms / 60000).toFixed(1)}m`;
}

/**
 * Format upload speed for logging
 * @param {number} bytes - Bytes transferred
 * @param {number} durationMs - Duration in milliseconds
 * @returns {string} Formatted speed
 */
export function formatUploadSpeed(bytes, durationMs) {
  const mbps = bytes / (1024 * 1024) / (durationMs / 1000);
  return `${mbps.toFixed(2)} MB/s`;
}

/**
 * Log JSON data with proper formatting
 * @param {Object} data - Data to log
 * @param {string} label - Label for the data
 */
export function logJson(data, label = "Data") {
  const jsonStr = JSON.stringify(data, (key, value) => (typeof value === "bigint" ? value.toString() : value), 2);
  logDebug(`${label}: ${jsonStr}`);
}
