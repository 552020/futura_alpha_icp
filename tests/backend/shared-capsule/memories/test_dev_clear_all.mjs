#!/usr/bin/env node

import { 
  initializeTestEnvironment, 
  getOrCreateTestCapsule, 
  createTestMemory,
  createTestMemoriesBatch,
  cleanupTestMemories
} from "./bulk-apis/bulk_test_helpers.mjs";

async function main() {
  console.log("🧪 Testing dev_clear_all_memories_in_capsule function...");

  // Initialize test environment
  const { actor } = await initializeTestEnvironment();
  console.log("✅ Test environment initialized");

  try {
    // Test 1: Try to clear a non-existent capsule (should return NotFound)
    console.log("\n📋 Test 1: Non-existent capsule");
    try {
      const result1 = await actor.dev_clear_all_memories_in_capsule("non-existent-capsule", true);
      console.log("❌ Expected error but got result:", result1);
    } catch (error) {
      console.log("✅ Got expected error for non-existent capsule:", error.message);
    }

    // Test 2: Try to clear an empty capsule (should return 0 deleted count)
    console.log("\n📋 Test 2: Empty capsule");
    const capsuleId = await getOrCreateTestCapsule(actor);
    console.log("Using capsule ID:", capsuleId);
    
    const result2 = await actor.dev_clear_all_memories_in_capsule(capsuleId, false);
    console.log("✅ Empty capsule result:", result2);
    console.log("   Deleted count:", result2.deleted_count);
    console.log("   Message:", result2.message);

    // Test 3: Create some memories and then clear them
    console.log("\n📋 Test 3: Clear capsule with memories");
    const memoryIds = await createTestMemoriesBatch(actor, capsuleId, 3, "dev_clear_test");
    console.log("Created memories:", memoryIds);

    const result3 = await actor.dev_clear_all_memories_in_capsule(capsuleId, true);
    console.log("✅ Clear with memories result:", result3);
    console.log("   Deleted count:", result3.deleted_count);
    console.log("   Message:", result3.message);

    console.log("\n🎉 All tests completed!");

  } catch (error) {
    console.error("❌ Test failed:", error);
  }
}

// Run the test
main().catch(console.error);
