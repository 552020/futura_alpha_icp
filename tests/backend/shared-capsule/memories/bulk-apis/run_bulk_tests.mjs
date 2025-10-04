#!/usr/bin/env node

/**
 * Bulk Memory APIs Test Runner
 * Orchestrates and runs all bulk memory API tests
 */

import { spawn } from "child_process";
import { fileURLToPath } from "url";
import { dirname, join } from "path";
import { readdir } from "fs/promises";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Colors for output
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

function log(message, color = "white") {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

function logHeader(message) {
  log("\n" + "=".repeat(60), "cyan");
  log(message, "cyan");
  log("=".repeat(60), "cyan");
}

function logSuccess(message) {
  log(`✅ ${message}`, "green");
}

function logError(message) {
  log(`❌ ${message}`, "red");
}

function logInfo(message) {
  log(`ℹ️  ${message}`, "blue");
}

function logWarning(message) {
  log(`⚠️  ${message}`, "yellow");
}

/**
 * Run a single test file
 */
async function runTestFile(testFile) {
  return new Promise((resolve) => {
    logInfo(`Running: ${testFile}`);

    const child = spawn("node", [testFile], {
      stdio: "inherit",
      cwd: __dirname,
    });

    child.on("close", (code) => {
      if (code === 0) {
        logSuccess(`${testFile} - PASSED`);
        resolve({ file: testFile, passed: true, code });
      } else {
        logError(`${testFile} - FAILED (exit code: ${code})`);
        resolve({ file: testFile, passed: false, code });
      }
    });

    child.on("error", (error) => {
      logError(`${testFile} - ERROR: ${error.message}`);
      resolve({ file: testFile, passed: false, error: error.message });
    });
  });
}

/**
 * Find all test files in the current directory
 */
async function findTestFiles() {
  try {
    const files = await readdir(__dirname);
    return files.filter((file) => file.endsWith(".mjs") && file.startsWith("test_")).sort();
  } catch (error) {
    logError(`Failed to read directory: ${error.message}`);
    return [];
  }
}

/**
 * Check if dfx is available
 */
async function checkDfxAvailability() {
  return new Promise((resolve) => {
    const child = spawn("dfx", ["--version"], { stdio: "pipe" });

    child.on("close", (code) => {
      if (code === 0) {
        logSuccess("dfx is available");
        resolve(true);
      } else {
        logError("dfx is not available or not working");
        resolve(false);
      }
    });

    child.on("error", () => {
      logError("dfx command not found");
      resolve(false);
    });
  });
}

/**
 * Check if the backend canister is running
 */
async function checkBackendCanister() {
  return new Promise((resolve) => {
    const child = spawn("dfx", ["canister", "status", "backend"], { stdio: "pipe" });

    child.on("close", (code) => {
      if (code === 0) {
        logSuccess("Backend canister is running");
        resolve(true);
      } else {
        logError("Backend canister is not running or not accessible");
        resolve(false);
      }
    });

    child.on("error", () => {
      logError("Failed to check backend canister status");
      resolve(false);
    });
  });
}

/**
 * Main test runner
 */
async function main() {
  logHeader("🧪 Bulk Memory APIs Test Runner");

  // Check prerequisites
  logInfo("Checking prerequisites...");

  const dfxAvailable = await checkDfxAvailability();
  if (!dfxAvailable) {
    logError("dfx is required but not available. Please install dfx and ensure it's in your PATH.");
    process.exit(1);
  }

  const backendRunning = await checkBackendCanister();
  if (!backendRunning) {
    logWarning("Backend canister may not be running. Tests may fail.");
    logInfo("To start the backend canister, run: dfx start && dfx deploy backend");
  }

  // Find and run test files
  logInfo("Finding test files...");
  const testFiles = await findTestFiles();

  if (testFiles.length === 0) {
    logError("No test files found");
    process.exit(1);
  }

  logInfo(`Found ${testFiles.length} test file(s): ${testFiles.join(", ")}`);

  // Run tests
  logHeader("Running Tests");

  const results = [];
  let totalTests = 0;
  let passedTests = 0;
  let failedTests = 0;

  for (const testFile of testFiles) {
    const result = await runTestFile(testFile);
    results.push(result);
    totalTests++;

    if (result.passed) {
      passedTests++;
    } else {
      failedTests++;
    }
  }

  // Print summary
  logHeader("Test Results Summary");

  logInfo(`Total tests: ${totalTests}`);
  logSuccess(`Passed: ${passedTests}`);

  if (failedTests > 0) {
    logError(`Failed: ${failedTests}`);

    logInfo("\nFailed tests:");
    results
      .filter((result) => !result.passed)
      .forEach((result) => {
        logError(`  - ${result.file} (exit code: ${result.code || "error"})`);
      });
  }

  // Final result
  if (failedTests === 0) {
    logSuccess("🎉 All bulk memory API tests passed!");
    process.exit(0);
  } else {
    logError(`❌ ${failedTests} test(s) failed`);
    process.exit(1);
  }
}

// Handle command line arguments
const args = process.argv.slice(2);

if (args.includes("--help") || args.includes("-h")) {
  logInfo("Bulk Memory APIs Test Runner");
  logInfo("Usage: node run_bulk_tests.mjs [options]");
  logInfo("");
  logInfo("Options:");
  logInfo("  --help, -h     Show this help message");
  logInfo("  --debug        Enable debug output");
  logInfo("");
  logInfo("Environment Variables:");
  logInfo("  DEBUG=true     Enable debug output");
  logInfo("  IC_HOST        ICP host (default: http://127.0.0.1:4943)");
  logInfo("  BACKEND_CANISTER_ID  Backend canister ID");
  process.exit(0);
}

if (args.includes("--debug")) {
  process.env.DEBUG = "true";
}

// Run main function
main().catch((error) => {
  logError(`Test runner failed: ${error.message}`);
  process.exit(1);
});

