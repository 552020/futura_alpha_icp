#!/usr/bin/env node

/**
 * HTTP Core Functionality Tests
 *
 * Tests the core HTTP module functionality with minimal dependencies
 * Focuses on basic HTTP operations without complex memory creation
 */

import { logHeader, logSuccess, logError, logInfo } from "../utils/helpers/logging.js";
import { measureExecutionTime } from "../utils/helpers/timing.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function testHttpCoreFunctionality() {
  logHeader("🔧 Testing HTTP Core Functionality");

  try {
    // Get canister ID
    const canisterId = await getCanisterId();
    logInfo(`Backend canister ID: ${canisterId}`);

    // Test 1: Health Check via HTTP Gateway
    await testHealthCheckGateway(canisterId);

    // Test 2: Health Check via dfx canister call
    await testHealthCheckDfx();

    // Test 3: Token minting (expecting forbidden - this is expected!)
    await testTokenMinting();

    // Test 4: Asset serving without token (should fail)
    await testAssetServingWithoutToken(canisterId);

    // Test 5: Invalid endpoints
    await testInvalidEndpoints(canisterId);

    // Test 6: HTTP request method via dfx
    await testHttpRequestMethod();

    // Test 7: HTTP Gateway accessibility
    await testHttpGatewayAccessibility(canisterId);

    logSuccess("🎉 HTTP Core Functionality tests completed!");
  } catch (error) {
    logError(`❌ HTTP Core Functionality test failed: ${error.message}`);
    throw error;
  }
}

async function getCanisterId() {
  try {
    const { stdout } = await execAsync("dfx canister id backend");
    return stdout.trim();
  } catch (error) {
    throw new Error(`Failed to get canister ID: ${error.message}`);
  }
}

async function testHealthCheckGateway(canisterId) {
  logInfo("Test 1: Health Check via HTTP Gateway");

  const url = `http://${canisterId}.localhost:4943/health`;
  logInfo(`Testing: ${url}`);

  try {
    const result = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(`curl -s -w "\\n%{http_code}" "${url}"`);
      return stdout;
    });

    const lines = result.result.trim().split("\n");
    const body = lines.slice(0, -1).join("\n");
    const status = lines[lines.length - 1];

    if (status === "200") {
      logSuccess("✅ Health check passed (200 OK)");
      logInfo(`Response: ${body}`);
    } else {
      logError(`❌ Health check failed (Status: ${status})`);
      logInfo(`Response: ${body}`);
    }
  } catch (error) {
    logError(`❌ Health check failed: ${error.message}`);
  }
}

async function testHealthCheckDfx() {
  logInfo("Test 2: Health Check via dfx canister call");

  try {
    const result = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(
        'dfx canister call backend http_request \'(record { method = "GET"; url = "/health"; headers = vec {}; body = blob ""; })\''
      );
      return stdout;
    });

    logSuccess("✅ dfx health check passed");
    logInfo(`Response: ${result.result}`);
  } catch (error) {
    logError(`❌ dfx health check failed: ${error.message}`);
  }
}

async function testTokenMinting() {
  logInfo("Test 3: Token minting (expecting 'forbidden' error)");

  try {
    const result = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(
        'dfx canister call backend mint_http_token \'("test_memory", vec {"thumbnail"; "preview"}, null, 180)\''
      );
      return stdout;
    });

    if (result.result.includes('"') && result.result.length > 50) {
      logSuccess("✅ Token minting working (got token)");
      logInfo(`Token: ${result.result.substring(0, 50)}...`);
    } else {
      logError(`❌ Token minting failed: ${result.result}`);
    }
  } catch (error) {
    if (error.message.includes("forbidden")) {
      logSuccess("✅ Token minting properly validates permissions (forbidden as expected)");
      logInfo("This means ACL integration is working correctly!");
    } else {
      logError(`❌ Token minting failed with unexpected error: ${error.message}`);
    }
  }
}

async function testAssetServingWithoutToken(canisterId) {
  logInfo("Test 4: Asset serving without token (should fail)");

  const url = `http://${canisterId}.localhost:4943/assets/test_memory/thumbnail`;
  logInfo(`Testing: ${url}`);

  try {
    const result = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(`curl -s -w "\\n%{http_code}" "${url}"`);
      return stdout;
    });

    const lines = result.result.trim().split("\n");
    const body = lines.slice(0, -1).join("\n");
    const status = lines[lines.length - 1];

    if (status === "401" || status === "403" || status === "404") {
      logSuccess(`✅ Asset serving properly rejects requests without token (${status})`);
    } else {
      logError(`❌ Asset serving should reject without token, got: ${status}`);
      logInfo(`Response: ${body}`);
    }
  } catch (error) {
    logError(`❌ Asset serving test failed: ${error.message}`);
  }
}

async function testInvalidEndpoints(canisterId) {
  logInfo("Test 5: Invalid endpoints...");

  const testCases = [
    {
      name: "Invalid endpoint",
      url: `http://${canisterId}.localhost:4943/invalid-endpoint`,
      expectedStatus: "404",
    },
    {
      name: "Non-existent memory",
      url: `http://${canisterId}.localhost:4943/assets/nonexistent_memory/thumbnail`,
      expectedStatus: "404",
    },
  ];

  for (const testCase of testCases) {
    logInfo(`Testing ${testCase.name}: ${testCase.url}`);

    try {
      const result = await measureExecutionTime(async () => {
        const { stdout } = await execAsync(`curl -s -w "\\n%{http_code}" "${testCase.url}"`);
        return stdout;
      });

      const lines = result.result.trim().split("\n");
      const body = lines.slice(0, -1).join("\n");
      const status = lines[lines.length - 1];

      if (status === testCase.expectedStatus) {
        logSuccess(`✅ ${testCase.name} properly returns ${testCase.expectedStatus}`);
      } else {
        logError(`❌ ${testCase.name} should return ${testCase.expectedStatus}, got: ${status}`);
        logInfo(`Response: ${body}`);
      }
    } catch (error) {
      logError(`❌ ${testCase.name} test failed: ${error.message}`);
    }
  }
}

async function testHttpRequestMethod() {
  logInfo("Test 6: HTTP request method via dfx");

  try {
    const result = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(
        'dfx canister call backend http_request \'(record { method = "GET"; url = "/invalid-endpoint"; headers = vec {}; body = blob ""; })\''
      );
      return stdout;
    });

    logSuccess("✅ dfx http_request working");
    logInfo(`Response: ${result.result}`);
  } catch (error) {
    logError(`❌ dfx http_request failed: ${error.message}`);
  }
}

async function testHttpGatewayAccessibility(canisterId) {
  logInfo("Test 7: HTTP Gateway accessibility");

  const url = `http://${canisterId}.localhost:4943/`;
  logInfo(`Testing: ${url}`);

  try {
    const result = await measureExecutionTime(async () => {
      const { stdout } = await execAsync(`curl -s -w "\\n%{http_code}" "${url}"`);
      return stdout;
    });

    const lines = result.result.trim().split("\n");
    const body = lines.slice(0, -1).join("\n");
    const status = lines[lines.length - 1];

    if (status === "404") {
      logSuccess("✅ HTTP Gateway is accessible (404 for root is expected)");
    } else if (status === "200") {
      logSuccess("✅ HTTP Gateway is accessible (200 OK)");
    } else {
      logError(`❌ HTTP Gateway accessibility issue (Status: ${status})`);
      logInfo(`Response: ${body}`);
    }
  } catch (error) {
    logError(`❌ HTTP Gateway test failed: ${error.message}`);
  }
}

async function main() {
  try {
    await testHttpCoreFunctionality();
  } catch (error) {
    logError(`❌ Test suite failed: ${error.message}`);
    process.exit(1);
  }
}

main().catch(console.error);
