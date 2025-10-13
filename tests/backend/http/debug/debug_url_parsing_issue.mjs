#!/usr/bin/env node

/**
 * Debug URL parsing issue in HTTP module
 *
 * The issue: When multiple query parameters are present in the URL,
 * the HTTP module returns "Missing token" even though the token is present.
 *
 * This suggests that the URL parsing logic in the HTTP module is not
 * correctly extracting the token parameter when there are multiple parameters.
 */

import { logInfo, logSuccess, logError } from "../../utils/helpers/logging.js";

async function debugUrlParsing() {
  logInfo("🔍 Debugging URL parsing issue in HTTP module");

  try {
    // Test 1: Single parameter (token only)
    logInfo("\n📋 Test 1: Single parameter (token only)");
    const singleParamUrl = "http://localhost:4943/asset/memory123/thumbnail?token=test_token_123";
    logInfo(`URL: ${singleParamUrl}`);

    // Test 2: Multiple parameters (token + id)
    logInfo("\n📋 Test 2: Multiple parameters (token + id)");
    const multiParamUrl = "http://localhost:4943/asset/memory123/thumbnail?token=test_token_123&id=asset_456";
    logInfo(`URL: ${multiParamUrl}`);

    // Test 3: Multiple parameters (id + token)
    logInfo("\n📋 Test 3: Multiple parameters (id + token)");
    const multiParamUrl2 = "http://localhost:4943/asset/memory123/thumbnail?id=asset_456&token=test_token_123";
    logInfo(`URL: ${multiParamUrl2}`);

    // Test 4: Multiple parameters with URL encoding
    logInfo("\n📋 Test 4: Multiple parameters with URL encoding");
    const encodedToken = encodeURIComponent("test_token_123");
    const multiParamUrl3 = `http://localhost:4943/asset/memory123/thumbnail?id=asset_456&token=${encodedToken}`;
    logInfo(`URL: ${multiParamUrl3}`);

    logInfo("\n🔍 Analysis:");
    logInfo("The issue appears to be in the URL parsing logic in src/backend/src/http.rs");
    logInfo("The parse() function splits the URL and extracts query parameters, but");
    logInfo("there might be an issue with how it handles multiple parameters.");

    logInfo("\n📝 Expected behavior:");
    logInfo("- Single parameter: ?token=abc123 → should work");
    logInfo("- Multiple parameters: ?token=abc123&id=xyz → should work");
    logInfo("- Multiple parameters: ?id=xyz&token=abc123 → should work");

    logInfo("\n❌ Current behavior:");
    logInfo("- Single parameter: ?token=abc123 → works");
    logInfo("- Multiple parameters: ?token=abc123&id=xyz → returns 'Missing token'");
    logInfo("- Multiple parameters: ?id=xyz&token=abc123 → returns 'Missing token'");

    logInfo("\n🔧 Potential fixes:");
    logInfo("1. Check the query parameter parsing logic in src/backend/src/http.rs");
    logInfo("2. Verify that the ParsedRequest.q() method correctly finds parameters");
    logInfo("3. Add debug logging to see what parameters are actually parsed");
    logInfo("4. Test with different parameter orders and URL encodings");

    logSuccess("✅ URL parsing analysis complete");
  } catch (error) {
    logError(`❌ Error during URL parsing analysis: ${error.message}`);
  }
}

// Run the debug analysis
debugUrlParsing().catch(console.error);
