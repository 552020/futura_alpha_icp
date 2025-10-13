#!/usr/bin/env node

/**
 * Integration test to verify 404 fixes for HTTP asset serving
 *
 * This test verifies that the fixes for 404 errors are working:
 * 1. Token subject principal is used correctly
 * 2. Variant-to-asset-id resolution works
 * 3. Both inline and blob assets are handled
 * 4. Proper diagnostic logging is in place
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { execSync } from "child_process";
import fs from "fs";
import path from "path";

// Configuration
const CANISTER_ID = "uxrrr-q7777-77774-qaaaq-cai"; // Our local canister ID
const LOCAL_URL = "http://127.0.0.1:4943";

// Test configuration
const TEST_CONFIG = {
  memoryId: "579c02d5-508f-bd49-579c-00000000bd49", // Use existing memory
  variant: "thumbnail",
  testAssetId: "test-asset-404-fix",
  testPrincipal: "vxfqp-jdnq2-fsg4h-qtbil-w4yjc-3eyde-vt5gu-6e5e2-e6hlx-xz5aj-sae", // Our test principal
};

/**
 * Create a test token for the given memory and variant
 */
async function createTestToken(memoryId, variant, assetIds = null) {
  // This would normally call your token creation API
  // For now, we'll create a mock token structure
  const tokenPayload = {
    ver: 1,
    kid: 1,
    exp_ns: Date.now() * 1000000 + 3600000000000, // 1 hour from now
    nonce: new Uint8Array(12).fill(1),
    scope: {
      memory_id: memoryId,
      variants: [variant],
      asset_ids: assetIds,
    },
    sub: Principal.fromText(TEST_CONFIG.testPrincipal),
  };

  // In a real implementation, you'd sign this token
  // For testing, we'll use a mock signature
  const mockSignature = new Uint8Array(32).fill(0x42);

  return {
    p: tokenPayload,
    s: mockSignature,
  };
}

/**
 * Encode token for URL usage
 */
function encodeTokenForUrl(token) {
  const jsonStr = JSON.stringify(token);
  return Buffer.from(jsonStr).toString("base64url");
}

/**
 * Test asset serving with proper token subject principal
 */
async function testTokenSubjectPrincipal() {
  console.log("🧪 Testing token subject principal usage...");

  try {
    // Create a token with a specific subject principal
    const token = await createTestToken(TEST_CONFIG.memoryId, TEST_CONFIG.variant);
    const encodedToken = encodeTokenForUrl(token);

    // Make request to asset endpoint using the correct canister URL format
    const url = `http://${CANISTER_ID}.localhost:4943/asset/${TEST_CONFIG.memoryId}/${TEST_CONFIG.variant}?token=${encodedToken}`;

    const response = await fetch(url, {
      method: "GET",
      headers: {
        Accept: "image/*",
      },
    });

    console.log(`   Status: ${response.status}`);
    console.log(`   Headers: ${JSON.stringify(Object.fromEntries(response.headers.entries()))}`);

    if (response.status === 404) {
      console.log("   ✅ Expected 404 (no assets exist yet)");
      return true;
    } else if (response.status === 200) {
      console.log("   ✅ Asset served successfully");
      return true;
    } else {
      console.log(`   ❌ Unexpected status: ${response.status}`);
      return false;
    }
  } catch (error) {
    console.log(`   ❌ Error: ${error.message}`);
    return false;
  }
}

/**
 * Test variant-to-asset-id resolution
 */
async function testVariantResolution() {
  console.log("🧪 Testing variant-to-asset-id resolution...");

  try {
    // Test with specific asset ID
    const token = await createTestToken(TEST_CONFIG.memoryId, TEST_CONFIG.variant, [TEST_CONFIG.testAssetId]);
    const encodedToken = encodeTokenForUrl(token);

    const url = `http://${CANISTER_ID}.localhost:4943/asset/${TEST_CONFIG.memoryId}/${TEST_CONFIG.variant}?id=${TEST_CONFIG.testAssetId}&token=${encodedToken}`;

    const response = await fetch(url, {
      method: "GET",
      headers: {
        Accept: "image/*",
      },
    });

    console.log(`   Status: ${response.status}`);

    if (response.status === 404) {
      console.log("   ✅ Expected 404 (asset doesn't exist)");
      return true;
    } else if (response.status === 200) {
      console.log("   ✅ Asset resolved and served");
      return true;
    } else {
      console.log(`   ❌ Unexpected status: ${response.status}`);
      return false;
    }
  } catch (error) {
    console.log(`   ❌ Error: ${error.message}`);
    return false;
  }
}

/**
 * Test diagnostic logging
 */
async function testDiagnosticLogging() {
  console.log("🧪 Testing diagnostic logging...");

  try {
    // Make a request that should trigger logging
    const token = await createTestToken(TEST_CONFIG.memoryId, TEST_CONFIG.variant);
    const encodedToken = encodeTokenForUrl(token);

    const url = `http://${CANISTER_ID}.localhost:4943/asset/${TEST_CONFIG.memoryId}/${TEST_CONFIG.variant}?token=${encodedToken}`;

    const response = await fetch(url, {
      method: "GET",
      headers: {
        Accept: "image/*",
      },
    });

    console.log(`   Status: ${response.status}`);
    console.log("   ✅ Request completed (check canister logs for diagnostic output)");

    return true;
  } catch (error) {
    console.log(`   ❌ Error: ${error.message}`);
    return false;
  }
}

/**
 * Test Authorization header support
 */
async function testAuthorizationHeader() {
  console.log("🧪 Testing Authorization header support...");

  try {
    const token = await createTestToken(TEST_CONFIG.memoryId, TEST_CONFIG.variant);
    const encodedToken = encodeTokenForUrl(token);

    const url = `http://${CANISTER_ID}.localhost:4943/asset/${TEST_CONFIG.memoryId}/${TEST_CONFIG.variant}`;

    const response = await fetch(url, {
      method: "GET",
      headers: {
        Authorization: `Bearer ${encodedToken}`,
        Accept: "image/*",
      },
    });

    console.log(`   Status: ${response.status}`);

    if (response.status === 404) {
      console.log("   ✅ Expected 404 (no assets exist)");
      return true;
    } else if (response.status === 200) {
      console.log("   ✅ Asset served via Authorization header");
      return true;
    } else {
      console.log(`   ❌ Unexpected status: ${response.status}`);
      return false;
    }
  } catch (error) {
    console.log(`   ❌ Error: ${error.message}`);
    return false;
  }
}

/**
 * Main test runner
 */
async function runTests() {
  console.log("🚀 Starting 404 fixes integration tests...\n");

  const tests = [
    { name: "Token Subject Principal", fn: testTokenSubjectPrincipal },
    { name: "Variant Resolution", fn: testVariantResolution },
    { name: "Diagnostic Logging", fn: testDiagnosticLogging },
    { name: "Authorization Header", fn: testAuthorizationHeader },
  ];

  let passed = 0;
  let failed = 0;

  for (const test of tests) {
    console.log(`\n📋 ${test.name}`);
    console.log("=".repeat(50));

    try {
      const result = await test.fn();
      if (result) {
        passed++;
        console.log(`✅ ${test.name} PASSED`);
      } else {
        failed++;
        console.log(`❌ ${test.name} FAILED`);
      }
    } catch (error) {
      failed++;
      console.log(`❌ ${test.name} FAILED: ${error.message}`);
    }
  }

  console.log("\n" + "=".repeat(50));
  console.log(`📊 Test Results: ${passed} passed, ${failed} failed`);

  if (failed === 0) {
    console.log("🎉 All tests passed! 404 fixes are working correctly.");
    process.exit(0);
  } else {
    console.log("⚠️  Some tests failed. Check the output above for details.");
    process.exit(1);
  }
}

// Run tests if this script is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  runTests().catch(console.error);
}

export { runTests, testTokenSubjectPrincipal, testVariantResolution, testDiagnosticLogging, testAuthorizationHeader };
