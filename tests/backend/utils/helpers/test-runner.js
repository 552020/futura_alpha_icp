/**
 * Test Runner Utilities
 *
 * Provides standardized test execution and reporting for all tests
 */

import { logHeader, logSuccess, logError, logInfo } from "./logging.js";

/**
 * Test runner class for managing test execution and reporting
 */
export class TestRunner {
  constructor(testSuiteName) {
    this.testSuiteName = testSuiteName;
    this.totalTests = 0;
    this.passedTests = 0;
    this.failedTests = 0;
  }

  /**
   * Run a single test
   * @param {string} testName - Name of the test
   * @param {Function} testFunction - Test function to run
   * @param {...any} args - Arguments to pass to the test function
   * @returns {Promise<boolean>} True if test passed, false otherwise
   */
  async runTest(testName, testFunction, ...args) {
    console.log(`\n[INFO] Running: ${testName}`);
    this.totalTests++;

    try {
      const result = await testFunction(...args);
      if (result && result.success) {
        console.log(`[PASS] ${testName}`);
        this.passedTests++;
        return true;
      } else {
        console.log(`[FAIL] ${testName}`);
        this.failedTests++;
        return false;
      }
    } catch (error) {
      console.log(`[FAIL] ${testName} - Error: ${error.message}`);
      this.failedTests++;
      return false;
    }
  }

  /**
   * Print test summary
   * @returns {boolean} True if all tests passed, false otherwise
   */
  printTestSummary() {
    console.log("\n=========================================");
    console.log(`Test Summary for ${this.testSuiteName}`);
    console.log("=========================================");
    console.log(`Total tests: ${this.totalTests}`);
    console.log(`Passed: ${this.passedTests}`);
    console.log(`Failed: ${this.failedTests}`);
    console.log("");

    if (this.failedTests === 0) {
      console.log(`✅ All ${this.testSuiteName} tests passed!`);
      return true;
    } else {
      console.log(`❌ ${this.failedTests} ${this.testSuiteName} test(s) failed`);
      return false;
    }
  }

  /**
   * Get test statistics
   * @returns {Object} Test statistics
   */
  getStats() {
    return {
      total: this.totalTests,
      passed: this.passedTests,
      failed: this.failedTests,
      successRate: this.totalTests > 0 ? (this.passedTests / this.totalTests) * 100 : 0,
    };
  }

  /**
   * Reset test counters
   */
  reset() {
    this.totalTests = 0;
    this.passedTests = 0;
    this.failedTests = 0;
  }
}

/**
 * Create a test runner instance
 * @param {string} testSuiteName - Name of the test suite
 * @returns {TestRunner} Test runner instance
 */
export function createTestRunner(testSuiteName) {
  return new TestRunner(testSuiteName);
}

/**
 * Run multiple tests with a test runner
 * @param {TestRunner} runner - Test runner instance
 * @param {Array} tests - Array of test objects with {name, fn, args}
 * @returns {Promise<boolean>} True if all tests passed, false otherwise
 */
export async function runTests(runner, tests) {
  for (const test of tests) {
    await runner.runTest(test.name, test.fn, ...(test.args || []));
  }
  return runner.printTestSummary();
}

/**
 * Simple test runner function (backward compatibility)
 * @param {string} testName - Name of the test
 * @param {Function} testFunction - Test function to run
 * @param {...any} args - Arguments to pass to the test function
 * @returns {Promise<Object>} Test result object
 */
export async function runTest(testName, testFunction, ...args) {
  console.log(`\n[INFO] Running: ${testName}`);

  try {
    const result = await testFunction(...args);
    if (result && result.success) {
      console.log(`[PASS] ${testName}`);
      return { success: true, result };
    } else {
      console.log(`[FAIL] ${testName}`);
      return { success: false, result };
    }
  } catch (error) {
    console.log(`[FAIL] ${testName} - Error: ${error.message}`);
    return { success: false, error: error.message };
  }
}

/**
 * Print test summary (backward compatibility)
 * @param {string} testSuiteName - Name of the test suite
 * @param {number} totalTests - Total number of tests
 * @param {number} passedTests - Number of passed tests
 * @param {number} failedTests - Number of failed tests
 * @returns {boolean} True if all tests passed, false otherwise
 */
export function printTestSummary(testSuiteName, totalTests, passedTests, failedTests) {
  console.log("\n=========================================");
  console.log(`Test Summary for ${testSuiteName}`);
  console.log("=========================================");
  console.log(`Total tests: ${totalTests}`);
  console.log(`Passed: ${passedTests}`);
  console.log(`Failed: ${failedTests}`);
  console.log("");

  if (failedTests === 0) {
    console.log(`✅ All ${testSuiteName} tests passed!`);
    return true;
  } else {
    console.log(`❌ ${failedTests} ${testSuiteName} test(s) failed`);
    return false;
  }
}

