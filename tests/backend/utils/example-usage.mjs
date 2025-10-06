#!/usr/bin/env node

/**
 * Test Framework Usage Example
 *
 * Demonstrates how to use the comprehensive test framework
 * for creating meaningful E2E tests with real data
 */

import {
  // Core utilities
  createTestActor,
  getEnvironmentInfo,

  // Data creation utilities
  getOrCreateTestCapsule,
  createTestMemory,
  createTestMemoriesBatch,
  createTestAssetData,

  // Validation utilities
  validateBulkDeleteResult,
  verifyMemoriesDeleted,
  verifyMemoriesExist,

  // Helper utilities
  logHeader,
  logSuccess,
  logError,
  logInfo,
  measureExecutionTime,
  createTestCleanup,

  // Test data fixtures
  getTestData,
  generateTestData,
} from "./index.js";

/**
 * Example: Comprehensive Bulk Memory API Test
 *
 * This example shows how to create a meaningful test that:
 * 1. Sets up real test data (capsule + memories)
 * 2. Tests bulk operations with actual data
 * 3. Verifies results and system state
 * 4. Cleans up test data
 */
async function exampleBulkMemoryTest() {
  logHeader("🧪 Comprehensive Bulk Memory API Test");

  // Initialize test environment
  const { actor, canisterId } = await createTestActor();
  logInfo(`Using canister: ${canisterId}`);

  // Setup test data
  logInfo("Setting up test data...");
  const capsuleId = await getOrCreateTestCapsule(actor);
  logInfo(`Using capsule: ${capsuleId}`);

  // Create test memories with real data
  const memoryIds = await createTestMemoriesBatch(actor, capsuleId, 5, {
    prefix: "bulk_test",
    baseContent: "Test Memory Content",
  });
  logInfo(`Created ${memoryIds.length} test memories`);

  // Create cleanup function
  const cleanup = createTestCleanup(actor, memoryIds, [capsuleId]);

  try {
    // Test 1: Bulk delete with real data
    logInfo("Testing memories_delete_bulk with real data...");
    const bulkDeleteResult = await measureExecutionTime(() => actor.memories_delete_bulk(capsuleId, memoryIds));

    // Validate result
    const validation = validateBulkDeleteResult(bulkDeleteResult.result, memoryIds.length, 0);

    if (validation.valid) {
      logSuccess(`Bulk delete succeeded: ${bulkDeleteResult.duration}ms`);

      // Verify memories are actually deleted
      const allDeleted = await verifyMemoriesDeleted(actor, memoryIds);
      if (allDeleted) {
        logSuccess("All memories successfully deleted");
      } else {
        logError("Some memories still exist after bulk delete");
      }
    } else {
      logError(`Bulk delete validation failed: ${validation.error}`);
    }

    // Test 2: Create new memories for delete_all test
    logInfo("Creating new memories for delete_all test...");
    const newMemoryIds = await createTestMemoriesBatch(actor, capsuleId, 3, {
      prefix: "delete_all_test",
      baseContent: "Delete All Test Content",
    });

    // Test delete_all
    logInfo("Testing memories_delete_all...");
    const deleteAllResult = await measureExecutionTime(() => actor.memories_delete_all(capsuleId));

    // Validate result
    const deleteAllValidation = validateBulkDeleteResult(deleteAllResult.result, newMemoryIds.length, 0);

    if (deleteAllValidation.valid) {
      logSuccess(`Delete all succeeded: ${deleteAllResult.duration}ms`);

      // Verify capsule is empty
      const capsuleEmpty = await verifyCapsuleEmpty(actor, capsuleId);
      if (capsuleEmpty) {
        logSuccess("Capsule is empty after delete_all");
      } else {
        logError("Capsule still contains memories after delete_all");
      }
    } else {
      logError(`Delete all validation failed: ${deleteAllValidation.error}`);
    }

    // Test 3: Asset cleanup with real data
    logInfo("Creating memory with assets for cleanup test...");
    const assetMemoryId = await createTestMemory(actor, capsuleId, {
      name: "asset_cleanup_test",
      description: "Memory for asset cleanup testing",
      content: "Asset cleanup test content",
      tags: ["test", "asset", "cleanup"],
    });

    // Test asset cleanup
    logInfo("Testing memories_cleanup_assets_all...");
    const assetCleanupResult = await measureExecutionTime(() => actor.memories_cleanup_assets_all(assetMemoryId));

    // Validate result
    const assetValidation = validateAssetCleanupResult(
      assetCleanupResult.result,
      1 // Expected 1 asset cleaned
    );

    if (assetValidation.valid) {
      logSuccess(`Asset cleanup succeeded: ${assetCleanupResult.duration}ms`);
    } else {
      logError(`Asset cleanup validation failed: ${assetValidation.error}`);
    }

    // Test 4: Performance testing
    logInfo("Testing performance with larger dataset...");
    const performanceMemoryIds = await createTestMemoriesBatch(actor, capsuleId, 20, {
      prefix: "performance_test",
      baseContent: "Performance Test Content",
    });

    const performanceResult = await measureExecutionTime(() =>
      actor.memories_delete_bulk(capsuleId, performanceMemoryIds)
    );

    const performanceValidation = validateBulkDeleteResult(performanceResult.result, performanceMemoryIds.length, 0);

    if (performanceValidation.valid) {
      logSuccess(
        `Performance test succeeded: ${performanceResult.duration}ms for ${performanceMemoryIds.length} memories`
      );

      // Calculate performance metrics
      const metrics = calculatePerformanceMetrics(performanceMemoryIds.length, performanceResult.duration);
      logInfo(`Performance: ${formatPerformanceMetrics(metrics)}`);
    } else {
      logError(`Performance test validation failed: ${performanceValidation.error}`);
    }

    logSuccess("🎉 All bulk memory API tests completed successfully!");
  } catch (error) {
    logError(`Test failed: ${error.message}`);
    throw error;
  } finally {
    // Cleanup test data
    logInfo("Cleaning up test data...");
    const cleanupResult = await cleanup();
    logInfo(`Cleaned up ${cleanupResult.deletedMemories} memories and ${cleanupResult.deletedCapsules} capsules`);
  }
}

/**
 * Example: Error Handling Test
 *
 * Demonstrates how to test error scenarios with meaningful data
 */
async function exampleErrorHandlingTest() {
  logHeader("🧪 Error Handling Test");

  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);

  try {
    // Test with invalid memory IDs
    logInfo("Testing bulk delete with invalid memory IDs...");
    const invalidMemoryIds = ["invalid_memory_1", "invalid_memory_2"];

    const result = await actor.memories_delete_bulk(capsuleId, invalidMemoryIds);

    // Should return Ok with failed_count > 0
    if (result.Ok && result.Ok.failed_count > 0) {
      logSuccess(`Correctly handled invalid memory IDs: ${result.Ok.failed_count} failed`);
    } else {
      logError("Expected partial failure for invalid memory IDs");
    }

    // Test with non-existent capsule
    logInfo("Testing with non-existent capsule...");
    const nonExistentCapsuleId = "non-existent-capsule-12345";

    try {
      await actor.memories_delete_bulk(nonExistentCapsuleId, ["test_memory"]);
      logError("Expected error for non-existent capsule");
    } catch (error) {
      logSuccess(`Correctly handled non-existent capsule: ${error.message}`);
    }
  } catch (error) {
    logError(`Error handling test failed: ${error.message}`);
    throw error;
  }
}

/**
 * Example: Performance Benchmarking
 *
 * Demonstrates how to benchmark different operations
 */
async function examplePerformanceBenchmark() {
  logHeader("🧪 Performance Benchmark");

  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);

  try {
    // Create test data
    const memoryIds = await createTestMemoriesBatch(actor, capsuleId, 50, {
      prefix: "benchmark_test",
      baseContent: "Benchmark Test Content",
    });

    // Benchmark different operations
    const operations = [
      {
        name: "Bulk Delete",
        fn: () => actor.memories_delete_bulk(capsuleId, memoryIds),
      },
      {
        name: "Individual Delete",
        fn: async () => {
          for (const memoryId of memoryIds) {
            await actor.memories_delete(memoryId);
          }
        },
      },
    ];

    const results = await benchmarkOperations(operations);

    logInfo("Performance Benchmark Results:");
    for (const result of results) {
      const metrics = calculatePerformanceMetrics(memoryIds.length, result.duration);
      logInfo(`${result.operation}: ${formatPerformanceMetrics(metrics)}`);
    }
  } catch (error) {
    logError(`Performance benchmark failed: ${error.message}`);
    throw error;
  }
}

/**
 * Main function to run all examples
 */
async function main() {
  try {
    await exampleBulkMemoryTest();
    await exampleErrorHandlingTest();
    await examplePerformanceBenchmark();

    logSuccess("🎉 All examples completed successfully!");
  } catch (error) {
    logError(`Examples failed: ${error.message}`);
    process.exit(1);
  }
}

// Run examples if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}

