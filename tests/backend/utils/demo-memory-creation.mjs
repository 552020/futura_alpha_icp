#!/usr/bin/env node

/**
 * Demo: Memory Creation and Retrieval
 * 
 * Demonstrates how to use the test framework to:
 * 1. Create a real memory with actual data
 * 2. Verify the memory exists and contains the expected data
 * 3. Show the difference between meaningful and meaningless testing
 */

import {
  // Core utilities
  createTestActor,
  getEnvironmentInfo,
  
  // Data creation
  getOrCreateTestCapsule,
  createTestMemory,
  createTestMemoryWithBlob,
  createTestMemoryWithExternal,
  
  // Validation
  verifyMemoriesExist,
  getMemoryInfo,
  listMemories,
  
  // Helpers
  logHeader,
  logSuccess,
  logError,
  logInfo,
  logWarning,
  measureExecutionTime,
  createTestCleanup,
  logJson
} from "./index.js";

/**
 * Demo 1: Create and Verify Inline Memory
 */
async function demoInlineMemory() {
  logHeader("🧪 Demo 1: Inline Memory Creation and Verification");
  
  // Initialize test environment
  const { actor, canisterId } = await createTestActor();
  logInfo(`Using canister: ${canisterId}`);
  
  // Get or create test capsule
  const capsuleId = await getOrCreateTestCapsule(actor);
  logInfo(`Using capsule: ${capsuleId}`);
  
  // Create cleanup function
  const cleanup = createTestCleanup(actor, [], [capsuleId]);
  
  try {
    // Create a real memory with actual content
    logInfo("Creating inline memory with real content...");
    const memoryId = await createTestMemory(actor, capsuleId, {
      name: "demo_inline_memory",
      description: "This is a demo memory with inline content",
      content: "Hello, this is real content stored inline in the memory!",
      tags: ["demo", "inline", "test"],
      mimeType: "text/plain"
    });
    
    logSuccess(`✅ Memory created with ID: ${memoryId}`);
    
    // Verify the memory exists
    logInfo("Verifying memory exists...");
    const memoryExists = await verifyMemoriesExist(actor, [memoryId]);
    
    if (memoryExists) {
      logSuccess("✅ Memory exists in the system");
      
      // Get detailed memory information
      logInfo("Retrieving memory details...");
      const memoryInfo = await getMemoryInfo(actor, memoryId);
      
      logInfo("Memory Details:");
      logJson(memoryInfo, "Memory Info");
      
      // Verify specific properties
      if (memoryInfo.metadata.title === "demo_inline_memory") {
        logSuccess("✅ Memory title is correct");
      } else {
        logError(`❌ Expected title 'demo_inline_memory', got '${memoryInfo.metadata.title}'`);
      }
      
      if (memoryInfo.inline_assets.length > 0) {
        logSuccess("✅ Memory has inline assets");
        logInfo(`Inline assets count: ${memoryInfo.inline_assets.length}`);
      } else {
        logError("❌ Memory has no inline assets");
      }
      
      // Verify the content
      const inlineAsset = memoryInfo.inline_assets[0];
      if (inlineAsset && inlineAsset.bytes) {
        const content = new TextDecoder().decode(inlineAsset.bytes);
        if (content.includes("Hello, this is real content")) {
          logSuccess("✅ Memory content is correct");
          logInfo(`Content: "${content}"`);
        } else {
          logError("❌ Memory content is incorrect");
        }
      }
      
    } else {
      logError("❌ Memory does not exist");
    }
    
  } catch (error) {
    logError(`Demo failed: ${error.message}`);
    throw error;
  } finally {
    // Cleanup
    logInfo("Cleaning up test data...");
    await cleanup();
  }
}

/**
 * Demo 2: Create and Verify Blob Memory
 */
async function demoBlobMemory() {
  logHeader("🧪 Demo 2: Blob Memory Creation and Verification");
  
  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);
  
  const cleanup = createTestCleanup(actor, [], [capsuleId]);
  
  try {
    // Create a blob memory
    logInfo("Creating blob memory...");
    const memoryId = await createTestMemoryWithBlob(actor, capsuleId, {
      name: "demo_blob_memory",
      description: "This is a demo memory with blob storage",
      blobRef: {
        locator: `blob-${Date.now()}`,
        len: 1024n,
        hash: []
      },
      fileSize: 1024,
      tags: ["demo", "blob", "test"]
    });
    
    logSuccess(`✅ Blob memory created with ID: ${memoryId}`);
    
    // Verify the memory exists
    const memoryExists = await verifyMemoriesExist(actor, [memoryId]);
    
    if (memoryExists) {
      logSuccess("✅ Blob memory exists in the system");
      
      const memoryInfo = await getMemoryInfo(actor, memoryId);
      
      if (memoryInfo.blob_internal_assets.length > 0) {
        logSuccess("✅ Memory has blob assets");
        logInfo(`Blob assets count: ${memoryInfo.blob_internal_assets.length}`);
      } else {
        logError("❌ Memory has no blob assets");
      }
      
    } else {
      logError("❌ Blob memory does not exist");
    }
    
  } catch (error) {
    logError(`Blob memory demo failed: ${error.message}`);
    throw error;
  } finally {
    await cleanup();
  }
}

/**
 * Demo 3: Create and Verify External Memory
 */
async function demoExternalMemory() {
  logHeader("🧪 Demo 3: External Memory Creation and Verification");
  
  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);
  
  const cleanup = createTestCleanup(actor, [], [capsuleId]);
  
  try {
    // Create an external memory
    logInfo("Creating external memory...");
    const memoryId = await createTestMemoryWithExternal(actor, capsuleId, {
      name: "demo_external_memory",
      description: "This is a demo memory with external storage",
      storageType: "S3",
      storageKey: "demo-bucket/demo-file.jpg",
      url: "https://s3.amazonaws.com/demo-bucket/demo-file.jpg",
      fileSize: 2048,
      tags: ["demo", "external", "test"]
    });
    
    logSuccess(`✅ External memory created with ID: ${memoryId}`);
    
    // Verify the memory exists
    const memoryExists = await verifyMemoriesExist(actor, [memoryId]);
    
    if (memoryExists) {
      logSuccess("✅ External memory exists in the system");
      
      const memoryInfo = await getMemoryInfo(actor, memoryId);
      
      if (memoryInfo.blob_external_assets.length > 0) {
        logSuccess("✅ Memory has external assets");
        logInfo(`External assets count: ${memoryInfo.blob_external_assets.length}`);
        
        const externalAsset = memoryInfo.blob_external_assets[0];
        logInfo(`Storage key: ${externalAsset.storage_key}`);
        logInfo(`URL: ${externalAsset.url}`);
      } else {
        logError("❌ Memory has no external assets");
      }
      
    } else {
      logError("❌ External memory does not exist");
    }
    
  } catch (error) {
    logError(`External memory demo failed: ${error.message}`);
    throw error;
  } finally {
    await cleanup();
  }
}

/**
 * Demo 4: Multiple Memories and List Verification
 */
async function demoMultipleMemories() {
  logHeader("🧪 Demo 4: Multiple Memories Creation and List Verification");
  
  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);
  
  const cleanup = createTestCleanup(actor, [], [capsuleId]);
  
  try {
    // Create multiple memories
    logInfo("Creating multiple memories...");
    const memoryIds = [];
    
    for (let i = 1; i <= 3; i++) {
      const memoryId = await createTestMemory(actor, capsuleId, {
        name: `demo_memory_${i}`,
        description: `Demo memory number ${i}`,
        content: `This is the content of demo memory ${i}`,
        tags: ["demo", "multiple", "test"]
      });
      memoryIds.push(memoryId);
      logInfo(`Created memory ${i}: ${memoryId}`);
    }
    
    logSuccess(`✅ Created ${memoryIds.length} memories`);
    
    // Verify all memories exist
    const allExist = await verifyMemoriesExist(actor, memoryIds);
    
    if (allExist) {
      logSuccess("✅ All memories exist in the system");
      
      // List all memories in the capsule
      logInfo("Listing all memories in capsule...");
      const memories = await listMemories(actor, capsuleId);
      
      logInfo(`Found ${memories.length} memories in capsule`);
      
      // Show details of each memory
      for (const memory of memories) {
        logInfo(`Memory: ${memory.id} - ${memory.metadata.title}`);
      }
      
    } else {
      logError("❌ Not all memories exist");
    }
    
  } catch (error) {
    logError(`Multiple memories demo failed: ${error.message}`);
    throw error;
  } finally {
    await cleanup();
  }
}

/**
 * Demo 5: Performance Measurement
 */
async function demoPerformanceMeasurement() {
  logHeader("🧪 Demo 5: Performance Measurement");
  
  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);
  
  const cleanup = createTestCleanup(actor, [], [capsuleId]);
  
  try {
    // Measure memory creation performance
    logInfo("Measuring memory creation performance...");
    
    const result = await measureExecutionTime(async () => {
      return await createTestMemory(actor, capsuleId, {
        name: "performance_test_memory",
        description: "Memory for performance testing",
        content: "Performance test content",
        tags: ["demo", "performance", "test"]
      });
    });
    
    logSuccess(`✅ Memory created in ${result.duration}ms`);
    logInfo(`Performance: ${result.durationMs}ms total, ${result.durationSeconds.toFixed(3)}s`);
    
    // Verify the memory exists
    const memoryExists = await verifyMemoriesExist(actor, [result.result]);
    
    if (memoryExists) {
      logSuccess("✅ Performance test memory exists");
    } else {
      logError("❌ Performance test memory does not exist");
    }
    
  } catch (error) {
    logError(`Performance demo failed: ${error.message}`);
    throw error;
  } finally {
    await cleanup();
  }
}

/**
 * Demo 6: Error Handling
 */
async function demoErrorHandling() {
  logHeader("🧪 Demo 6: Error Handling");
  
  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);
  
  try {
    // Try to get a non-existent memory
    logInfo("Testing error handling with non-existent memory...");
    
    try {
      const memoryInfo = await getMemoryInfo(actor, "non-existent-memory-id");
      logError("❌ Expected error for non-existent memory");
    } catch (error) {
      logSuccess(`✅ Correctly handled non-existent memory: ${error.message}`);
    }
    
    // Try to create memory with invalid capsule
    logInfo("Testing error handling with invalid capsule...");
    
    try {
      await createTestMemory(actor, "invalid-capsule-id", {
        name: "error_test_memory",
        content: "This should fail"
      });
      logError("❌ Expected error for invalid capsule");
    } catch (error) {
      logSuccess(`✅ Correctly handled invalid capsule: ${error.message}`);
    }
    
  } catch (error) {
    logError(`Error handling demo failed: ${error.message}`);
    throw error;
  }
}

/**
 * Main demo function
 */
async function main() {
  logHeader("🚀 Test Framework Demo - Memory Creation and Verification");
  
  try {
    // Run all demos
    await demoInlineMemory();
    await demoBlobMemory();
    await demoExternalMemory();
    await demoMultipleMemories();
    await demoPerformanceMeasurement();
    await demoErrorHandling();
    
    logSuccess("🎉 All demos completed successfully!");
    
  } catch (error) {
    logError(`Demo failed: ${error.message}`);
    process.exit(1);
  }
}

// Run demo if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}

