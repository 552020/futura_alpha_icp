/**
 * Test bulk token minting functionality
 *
 * This test demonstrates the bulk token API and frontend caching optimization
 * for dashboard scenarios where multiple memories need tokens.
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { idlFactory } from "../declarations/backend/backend.did.js";
import { TokenManager } from "./token-manager.mjs";
import { createTestCapsule, createTestMemoryWithImage, cleanupTestResources } from "../utils/helpers/http-auth.js";

// Test configuration
const CANISTER_ID = "uxrrr-q7777-77774-qaaaq-cai"; // Local canister ID
const TEST_CONFIG = {
  variants: ["thumbnail"],
  ttlSecs: 300,
};

// Helper functions
function logInfo(message) {
  console.log(`ℹ️  ${message}`);
}

function logSuccess(message) {
  console.log(`✅ ${message}`);
}

function logError(message) {
  console.error(`❌ ${message}`);
}

function logPerformance(message) {
  console.log(`⚡ ${message}`);
}

/**
 * Test individual token minting
 */
async function testIndividualTokens(actor) {
  logInfo("Testing individual token minting...");

  const startTime = Date.now();
  const tokens = [];

  for (const memoryId of TEST_CONFIG.memoryIds) {
    try {
      const token = await actor.mint_http_token(
        memoryId,
        TEST_CONFIG.variants,
        [], // No specific asset IDs (empty array instead of null)
        TEST_CONFIG.ttlSecs
      );
      tokens.push({ memoryId, token });
      logSuccess(`Token minted for ${memoryId}: ${token.substring(0, 20)}...`);
    } catch (error) {
      logError(`Failed to mint token for ${memoryId}: ${error.message}`);
    }
  }

  const duration = Date.now() - startTime;
  logPerformance(`Individual tokens: ${tokens.length}/${TEST_CONFIG.memoryIds.length} in ${duration}ms`);

  return tokens;
}

/**
 * Test bulk token minting
 */
async function testBulkTokens(actor) {
  logInfo("Testing bulk token minting...");

  const startTime = Date.now();

  try {
    const tokenPairs = await actor.mint_http_tokens_bulk(
      TEST_CONFIG.memoryIds,
      TEST_CONFIG.variants,
      [], // No specific asset IDs (empty array instead of null)
      TEST_CONFIG.ttlSecs
    );

    const duration = Date.now() - startTime;
    logPerformance(`Bulk tokens: ${tokenPairs.length}/${TEST_CONFIG.memoryIds.length} in ${duration}ms`);

    // Log results
    for (const [memoryId, token] of tokenPairs) {
      logSuccess(`Bulk token for ${memoryId}: ${token.substring(0, 20)}...`);
    }

    return tokenPairs;
  } catch (error) {
    logError(`Bulk token minting failed: ${error.message}`);
    return [];
  }
}

/**
 * Test TokenManager caching
 */
async function testTokenManager(actor) {
  logInfo("Testing TokenManager with caching...");

  const tokenManager = new TokenManager(actor);

  // Test individual token with caching
  logInfo("Testing individual token caching...");
  const startTime1 = Date.now();
  const token1 = await tokenManager.getToken(TEST_CONFIG.memoryIds[0], TEST_CONFIG.variants);
  const duration1 = Date.now() - startTime1;
  logPerformance(`First token fetch: ${duration1}ms`);

  const startTime2 = Date.now();
  const token2 = await tokenManager.getToken(TEST_CONFIG.memoryIds[0], TEST_CONFIG.variants);
  const duration2 = Date.now() - startTime2;
  logPerformance(`Cached token fetch: ${duration2}ms`);

  if (token1 === token2) {
    logSuccess("Token caching working correctly");
  } else {
    logError("Token caching failed - different tokens returned");
  }

  // Test bulk tokens with caching
  logInfo("Testing bulk token caching...");
  const startTime3 = Date.now();
  const bulkTokens1 = await tokenManager.getBulkTokens(TEST_CONFIG.memoryIds, TEST_CONFIG.variants);
  const duration3 = Date.now() - startTime3;
  logPerformance(`First bulk fetch: ${duration3}ms`);

  const startTime4 = Date.now();
  const bulkTokens2 = await tokenManager.getBulkTokens(TEST_CONFIG.memoryIds, TEST_CONFIG.variants);
  const duration4 = Date.now() - startTime4;
  logPerformance(`Cached bulk fetch: ${duration4}ms`);

  if (bulkTokens1.size === bulkTokens2.size) {
    logSuccess(`Bulk token caching working: ${bulkTokens1.size} tokens cached`);
  } else {
    logError("Bulk token caching failed - different token counts");
  }

  // Test cache stats
  const stats = tokenManager.getCacheStats();
  logInfo(`Cache stats: ${JSON.stringify(stats, null, 2)}`);

  return tokenManager;
}

/**
 * Test asset URL generation
 */
async function testAssetUrls(tokenManager) {
  logInfo("Testing asset URL generation...");

  // Test single asset URL
  const singleUrl = await tokenManager.getAssetUrl(
    TEST_CONFIG.memoryIds[0],
    "thumbnail",
    null,
    "http://localhost:4943"
  );
  logSuccess(`Single asset URL: ${singleUrl}`);

  // Test bulk asset URLs
  const bulkUrls = await tokenManager.getBulkAssetUrls(
    TEST_CONFIG.memoryIds.slice(0, 3), // Test with first 3 memories
    "thumbnail",
    "http://localhost:4943"
  );

  logSuccess(`Bulk asset URLs generated: ${bulkUrls.size}`);
  for (const [memoryId, url] of bulkUrls) {
    logInfo(`  ${memoryId}: ${url.substring(0, 80)}...`);
  }
}

/**
 * Performance comparison test
 */
async function performanceComparison(actor) {
  logInfo("Running performance comparison...");

  const iterations = 3;
  const memoryIds = TEST_CONFIG.memoryIds.slice(0, 3); // Use subset for faster testing

  // Test individual token performance
  let individualTotal = 0;
  for (let i = 0; i < iterations; i++) {
    const startTime = Date.now();
    for (const memoryId of memoryIds) {
      try {
        await actor.mint_http_token(memoryId, ["thumbnail"], [], 180);
      } catch (error) {
        // Ignore errors for performance testing
      }
    }
    individualTotal += Date.now() - startTime;
  }
  const individualAvg = individualTotal / iterations;

  // Test bulk token performance
  let bulkTotal = 0;
  for (let i = 0; i < iterations; i++) {
    const startTime = Date.now();
    try {
      await actor.mint_http_tokens_bulk(memoryIds, ["thumbnail"], [], 180);
    } catch (error) {
      // Ignore errors for performance testing
    }
    bulkTotal += Date.now() - startTime;
  }
  const bulkAvg = bulkTotal / iterations;

  logPerformance(`Individual tokens (avg): ${individualAvg.toFixed(2)}ms`);
  logPerformance(`Bulk tokens (avg): ${bulkAvg.toFixed(2)}ms`);

  if (bulkAvg < individualAvg) {
    const improvement = (((individualAvg - bulkAvg) / individualAvg) * 100).toFixed(1);
    logSuccess(`Bulk tokens are ${improvement}% faster!`);
  } else {
    logInfo("Performance difference is minimal (expected for small datasets)");
  }
}

/**
 * Main test runner
 */
async function runTests() {
  logInfo("Starting bulk token tests...");

  let capsuleId = null;
  let memoryIds = [];
  let actor = null;

  try {
    // Create actor
    const agent = new HttpAgent({ host: "http://localhost:4943" });
    await agent.fetchRootKey();
    actor = Actor.createActor(idlFactory, { agent, canisterId: CANISTER_ID });

    // Step 1: Create test capsule
    logInfo("Creating test capsule...");
    capsuleId = await createTestCapsule();
    logSuccess(`✅ Test capsule created: ${capsuleId}`);

    // Step 2: Create test memories
    logInfo("Creating test memories...");
    for (let i = 1; i <= 3; i++) {
      const memoryId = await createTestMemoryWithImage(capsuleId, {
        name: `bulk_test_${i}.jpg`,
        mimeType: "image/jpeg",
      });
      memoryIds.push(memoryId);
      logSuccess(`✅ Test memory ${i} created: ${memoryId}`);
    }

    logInfo(`Testing with ${memoryIds.length} real memories`);

    // Update test config with real memory IDs
    TEST_CONFIG.memoryIds = memoryIds;

    // Run tests
    await testIndividualTokens(actor);
    console.log("");

    await testBulkTokens(actor);
    console.log("");

    const tokenManager = await testTokenManager(actor);
    console.log("");

    await testAssetUrls(tokenManager);
    console.log("");

    await performanceComparison(actor);
    console.log("");

    logSuccess("All bulk token tests completed!");
  } catch (error) {
    logError(`Test failed: ${error.message}`);
    console.error(error);
  } finally {
    // Clean up test resources
    if (memoryIds.length > 0) {
      logInfo("Cleaning up test memories...");
      for (const memoryId of memoryIds) {
        try {
          await cleanupTestResources(memoryId);
          logSuccess(`✅ Memory ${memoryId} cleaned up`);
        } catch (cleanupError) {
          logError(`❌ Cleanup failed for ${memoryId}: ${cleanupError.message}`);
        }
      }
    }
  }
}

// Run tests if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  runTests();
}

export { runTests, testIndividualTokens, testBulkTokens, testTokenManager };
