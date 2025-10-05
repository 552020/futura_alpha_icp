/**
 * Timing Utilities
 *
 * Provides utilities for performance measurement and timing
 */

/**
 * Create a timer for measuring execution time
 * @returns {Object} Timer object with start, stop, and duration methods
 */
export function createTimer() {
  let startTime = null;
  let endTime = null;

  return {
    start() {
      startTime = Date.now();
      return this;
    },

    stop() {
      endTime = Date.now();
      return this;
    },

    getDuration() {
      if (startTime === null) {
        throw new Error("Timer not started");
      }
      const end = endTime || Date.now();
      return end - startTime;
    },

    getDurationMs() {
      return this.getDuration();
    },

    getDurationSeconds() {
      return this.getDuration() / 1000;
    },

    reset() {
      startTime = null;
      endTime = null;
      return this;
    },
  };
}

/**
 * Measure execution time of a function
 * @param {Function} fn - Function to measure
 * @param {Array} args - Arguments to pass to function
 * @returns {Promise<Object>} Result with timing information
 */
export async function measureExecutionTime(fn, args = []) {
  const timer = createTimer();

  timer.start();
  const result = await fn(...args);
  timer.stop();

  return {
    result,
    duration: timer.getDuration(),
    durationMs: timer.getDurationMs(),
    durationSeconds: timer.getDurationSeconds(),
  };
}

/**
 * Measure execution time of a synchronous function
 * @param {Function} fn - Function to measure
 * @param {Array} args - Arguments to pass to function
 * @returns {Object} Result with timing information
 */
export function measureExecutionTimeSync(fn, args = []) {
  const timer = createTimer();

  timer.start();
  const result = fn(...args);
  timer.stop();

  return {
    result,
    duration: timer.getDuration(),
    durationMs: timer.getDurationMs(),
    durationSeconds: timer.getDurationSeconds(),
  };
}

/**
 * Create a timeout promise
 * @param {number} ms - Timeout in milliseconds
 * @param {string} message - Timeout message
 * @returns {Promise} Timeout promise
 */
export function createTimeout(ms, message = "Operation timed out") {
  return new Promise((_, reject) => {
    setTimeout(() => reject(new Error(message)), ms);
  });
}

/**
 * Race between operation and timeout
 * @param {Promise} promise - Operation promise
 * @param {number} ms - Timeout in milliseconds
 * @param {string} message - Timeout message
 * @returns {Promise} Result of operation or timeout error
 */
export async function withTimeout(promise, ms, message = "Operation timed out") {
  return Promise.race([promise, createTimeout(ms, message)]);
}

/**
 * Sleep function for testing delays
 * @param {number} ms - Sleep duration in milliseconds
 * @returns {Promise} Sleep promise
 */
export function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
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
      await sleep(delay);
    }
  }
}

/**
 * Benchmark multiple operations
 * @param {Array<Function>} operations - Array of functions to benchmark
 * @param {Array<Array>} args - Array of arguments for each operation
 * @returns {Promise<Array>} Array of benchmark results
 */
export async function benchmarkOperations(operations, args = []) {
  const results = [];

  for (let i = 0; i < operations.length; i++) {
    const operation = operations[i];
    const operationArgs = args[i] || [];

    const result = await measureExecutionTime(operation, operationArgs);
    results.push({
      operation: operation.name || `Operation ${i + 1}`,
      ...result,
    });
  }

  return results;
}

/**
 * Calculate performance metrics
 * @param {number} itemCount - Number of items processed
 * @param {number} durationMs - Duration in milliseconds
 * @returns {Object} Performance metrics
 */
export function calculatePerformanceMetrics(itemCount, durationMs) {
  const itemsPerSecond = Math.round((itemCount / durationMs) * 1000);
  const itemsPerMinute = itemsPerSecond * 60;
  const averageTimePerItem = durationMs / itemCount;

  return {
    itemCount,
    durationMs,
    itemsPerSecond,
    itemsPerMinute,
    averageTimePerItem,
  };
}

/**
 * Format performance metrics for logging
 * @param {Object} metrics - Performance metrics
 * @returns {string} Formatted metrics string
 */
export function formatPerformanceMetrics(metrics) {
  const { itemCount, durationMs, itemsPerSecond, averageTimePerItem } = metrics;

  return `${itemCount} items in ${durationMs}ms (${itemsPerSecond} items/sec, ${averageTimePerItem.toFixed(2)}ms/item)`;
}
