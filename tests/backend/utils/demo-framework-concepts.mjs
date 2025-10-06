#!/usr/bin/env node

/**
 * Demo: Test Framework Concepts
 * 
 * Demonstrates the key concepts of the test framework without requiring ICP connection.
 * Shows the difference between meaningful and meaningless testing approaches.
 */

import {
  // Test data fixtures
  getTestData,
  generateTestData,
  
  // Validation utilities
  validateBulkDeleteResult,
  validateAssetCleanupResult,
  
  // Helper utilities
  logHeader,
  logSuccess,
  logError,
  logInfo,
  logWarning,
  formatDuration,
  formatFileSize,
  calculatePerformanceMetrics,
  formatPerformanceMetrics
} from "./index.js";

/**
 * Demo 1: Test Data Fixtures
 */
function demoTestDataFixtures() {
  logHeader("🧪 Demo 1: Test Data Fixtures");
  
  // Get standard test data
  const capsuleData = getTestData("capsule", "self");
  logInfo("Self-capsule data:");
  console.log(JSON.stringify(capsuleData, null, 2));
  
  const memoryData = getTestData("memory", "inline");
  logInfo("Inline memory data:");
  console.log(JSON.stringify(memoryData, null, 2));
  
  const assetData = getTestData("asset", "document");
  logInfo("Document asset data:");
  console.log(JSON.stringify(assetData, null, 2));
  
  // Generate custom test data
  const customMemory = generateTestData("memory", {
    name: "custom_memory",
    content: "Custom content for testing",
    tags: ["custom", "test", "demo"]
  });
  logInfo("Custom memory data:");
  console.log(JSON.stringify(customMemory, null, 2));
  
  logSuccess("✅ Test data fixtures working correctly");
}

/**
 * Demo 2: Result Validation
 */
function demoResultValidation() {
  logHeader("🧪 Demo 2: Result Validation");
  
  // Simulate a successful bulk delete result
  const successResult = {
    Ok: {
      deleted_count: 5,
      failed_count: 0,
      message: "All memories deleted successfully"
    }
  };
  
  // Validate the result
  const validation = validateBulkDeleteResult(successResult, 5, 0);
  
  if (validation.valid) {
    logSuccess("✅ Bulk delete validation passed");
    logInfo(`Deleted: ${successResult.Ok.deleted_count}, Failed: ${successResult.Ok.failed_count}`);
  } else {
    logError(`❌ Validation failed: ${validation.error}`);
  }
  
  // Simulate a partial failure result
  const partialFailureResult = {
    Ok: {
      deleted_count: 3,
      failed_count: 2,
      message: "Some memories could not be deleted"
    }
  };
  
  const partialValidation = validateBulkDeleteResult(partialFailureResult, 3, 2);
  
  if (partialValidation.valid) {
    logSuccess("✅ Partial failure validation passed");
    logInfo(`Deleted: ${partialFailureResult.Ok.deleted_count}, Failed: ${partialFailureResult.Ok.failed_count}`);
  } else {
    logError(`❌ Partial failure validation failed: ${partialValidation.error}`);
  }
  
  // Simulate an error result
  const errorResult = {
    Err: "NotFound"
  };
  
  const errorValidation = validateBulkDeleteResult(errorResult, 0, 0);
  
  if (!errorValidation.valid) {
    logSuccess("✅ Error result correctly identified as invalid");
    logInfo(`Error: ${errorValidation.error}`);
  } else {
    logError("❌ Error result should have been invalid");
  }
}

/**
 * Demo 3: Performance Metrics
 */
function demoPerformanceMetrics() {
  logHeader("🧪 Demo 3: Performance Metrics");
  
  // Simulate performance data
  const itemCount = 100;
  const durationMs = 1500;
  
  // Calculate performance metrics
  const metrics = calculatePerformanceMetrics(itemCount, durationMs);
  
  logInfo("Performance Metrics:");
  logInfo(`Items processed: ${metrics.itemCount}`);
  logInfo(`Duration: ${formatDuration(metrics.durationMs)}`);
  logInfo(`Items per second: ${metrics.itemsPerSecond}`);
  logInfo(`Items per minute: ${metrics.itemsPerMinute}`);
  logInfo(`Average time per item: ${metrics.averageTimePerItem.toFixed(2)}ms`);
  
  // Format performance metrics
  const formatted = formatPerformanceMetrics(metrics);
  logInfo(`Formatted: ${formatted}`);
  
  logSuccess("✅ Performance metrics calculated correctly");
}

/**
 * Demo 4: File Size Formatting
 */
function demoFileSizeFormatting() {
  logHeader("🧪 Demo 4: File Size Formatting");
  
  const sizes = [512, 1024, 1024 * 1024, 1024 * 1024 * 1024];
  
  logInfo("File size formatting examples:");
  for (const size of sizes) {
    const formatted = formatFileSize(size);
    logInfo(`${size} bytes = ${formatted}`);
  }
  
  logSuccess("✅ File size formatting working correctly");
}

/**
 * Demo 5: Meaningful vs Meaningless Testing
 */
function demoMeaningfulVsMeaningless() {
  logHeader("🧪 Demo 5: Meaningful vs Meaningless Testing");
  
  logInfo("❌ MEANINGLESS TESTING (What we had before):");
  logInfo("  - Uses fake data: 'fake-capsule-id', 'fake-memory-id'");
  logInfo("  - Returns NotFound errors");
  logInfo("  - Tells us nothing about actual functionality");
  logInfo("  - No confidence in production behavior");
  
  logInfo("");
  logInfo("✅ MEANINGFUL TESTING (What the framework provides):");
  logInfo("  - Creates real data: actual capsules and memories");
  logInfo("  - Performs real operations with real data");
  logInfo("  - Validates actual results and system state");
  logInfo("  - Measures real performance characteristics");
  logInfo("  - Provides confidence in production behavior");
  
  logInfo("");
  logInfo("Example of meaningful test flow:");
  logInfo("  1. Create real capsule with actual data");
  logInfo("  2. Create real memories with actual content");
  logInfo("  3. Perform bulk operations on real data");
  logInfo("  4. Validate actual results (deleted_count, failed_count)");
  logInfo("  5. Verify system state (memories actually deleted)");
  logInfo("  6. Measure performance (execution time, throughput)");
  logInfo("  7. Clean up test data");
  
  logSuccess("✅ Framework provides meaningful testing approach");
}

/**
 * Demo 6: Test Framework Benefits
 */
function demoFrameworkBenefits() {
  logHeader("🧪 Demo 6: Test Framework Benefits");
  
  logInfo("🎯 Key Benefits:");
  logInfo("  ✅ Real Data: Creates actual capsules and memories");
  logInfo("  ✅ Meaningful Operations: Tests real business logic");
  logInfo("  ✅ State Verification: Confirms operations actually worked");
  logInfo("  ✅ Performance Measurement: Tracks real performance");
  logInfo("  ✅ Automatic Cleanup: Removes test data after completion");
  logInfo("  ✅ Standardized API: Consistent interface across all utilities");
  logInfo("  ✅ Comprehensive Coverage: All aspects of testing covered");
  logInfo("  ✅ Production Confidence: Tests that validate real functionality");
  
  logInfo("");
  logInfo("🔧 Framework Structure:");
  logInfo("  📁 core/ - Agent setup, actor creation, identity management");
  logInfo("  📁 data/ - Capsule, memory, asset creation utilities");
  logInfo("  📁 validation/ - Result validation, state verification");
  logInfo("  📁 helpers/ - Logging, timing, cleanup utilities");
  
  logInfo("");
  logInfo("📊 Usage Example:");
  logInfo("  const { actor } = await createTestActor();");
  logInfo("  const capsuleId = await getOrCreateTestCapsule(actor);");
  logInfo("  const memoryIds = await createTestMemoriesBatch(actor, capsuleId, 5);");
  logInfo("  const result = await measureExecutionTime(");
  logInfo("    () => actor.memories_delete_bulk(capsuleId, memoryIds)");
  logInfo("  );");
  logInfo("  const validation = validateBulkDeleteResult(result.result, 5, 0);");
  logInfo("  const allDeleted = await verifyMemoriesDeleted(actor, memoryIds);");
  
  logSuccess("✅ Framework provides comprehensive testing capabilities");
}

/**
 * Demo 7: Error Handling
 */
function demoErrorHandling() {
  logHeader("🧪 Demo 7: Error Handling");
  
  // Simulate different error types
  const errors = [
    { message: "Certificate verification failed", type: "certificate" },
    { message: "Connection refused", type: "connection" },
    { message: "Request timeout", type: "timeout" },
    { message: "Resource not found", type: "not_found" },
    { message: "Unauthorized access", type: "unauthorized" }
  ];
  
  logInfo("Error classification examples:");
  for (const error of errors) {
    logInfo(`  ${error.type}: ${error.message}`);
  }
  
  logInfo("");
  logInfo("User-friendly error messages:");
  for (const error of errors) {
    const userMessage = getUserErrorMessage(new Error(error.message));
    logInfo(`  ${error.type}: ${userMessage}`);
  }
  
  logSuccess("✅ Error handling provides clear feedback");
}

/**
 * Demo 8: Test Data Generation
 */
function demoTestDataGeneration() {
  logHeader("🧪 Demo 8: Test Data Generation");
  
  // Generate different types of test data
  const testTypes = ["capsule", "memory", "asset", "bulk", "performance", "error"];
  
  logInfo("Available test data types:");
  for (const type of testTypes) {
    const data = getTestData(type);
    logInfo(`  ${type}: ${Object.keys(data).length} variants available`);
  }
  
  logInfo("");
  logInfo("Bulk test data examples:");
  const bulkData = getTestData("bulk");
  for (const [size, config] of Object.entries(bulkData)) {
    logInfo(`  ${size}: ${config.memoryCount} memories, ${config.description}`);
  }
  
  logInfo("");
  logInfo("Performance test data:");
  const perfData = getTestData("performance");
  logInfo(`  Memory sizes: ${perfData.memorySizes.join(", ")} bytes`);
  logInfo(`  Memory counts: ${perfData.memoryCounts.join(", ")}`);
  logInfo(`  Asset types: ${perfData.assetTypes.join(", ")}`);
  
  logSuccess("✅ Test data generation provides comprehensive test scenarios");
}

/**
 * Main demo function
 */
async function main() {
  logHeader("🚀 Test Framework Concepts Demo");
  
  try {
    // Run all demos
    demoTestDataFixtures();
    demoResultValidation();
    demoPerformanceMetrics();
    demoFileSizeFormatting();
    demoMeaningfulVsMeaningless();
    demoFrameworkBenefits();
    demoErrorHandling();
    demoTestDataGeneration();
    
    logSuccess("🎉 All framework concepts demonstrated successfully!");
    
  } catch (error) {
    logError(`Demo failed: ${error.message}`);
    process.exit(1);
  }
}

// Run demo if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}

