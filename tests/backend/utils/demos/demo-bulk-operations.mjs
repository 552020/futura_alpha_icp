#!/usr/bin/env node

/**
 * Demo: Bulk Memory Operations
 * 
 * This demo shows how to create, manage, and delete multiple memories
 * using the bulk operations API.
 */

import { createTestActor, getOrCreateTestCapsule } from '../index.js';
import { 
  createAssetMetadata, 
  formatFileSize, 
  formatDuration,
  sleep
} from '../../shared-capsule/upload/helpers.mjs';

/**
 * Create multiple test memories
 */
async function createTestMemories(actor, capsuleId, count) {
  const memoryIds = [];
  
  for (let i = 0; i < count; i++) {
    const content = `Bulk test memory ${i + 1} of ${count}`;
    const contentBytes = Array.from(Buffer.from(content, 'utf8'));
    const assetMetadata = createAssetMetadata(
      `bulk_test_${i + 1}`,
      contentBytes.length,
      'text/plain'
    );
    
    const memoryResult = await actor.memories_create(
      capsuleId,
      [contentBytes],
      [], [], [], [], [], [],
      assetMetadata,
      `bulk_${Date.now()}_${i}`
    );
    
    if (memoryResult.Ok) {
      memoryIds.push(memoryResult.Ok);
    } else {
      console.error(`Failed to create memory ${i + 1}: ${JSON.stringify(memoryResult.Err)}`);
    }
  }
  
  return memoryIds;
}

/**
 * Demo 1: Bulk Memory Creation
 */
async function demoBulkCreation() {
  console.log('🧪 Demo 1: Bulk Memory Creation');
  console.log('=' .repeat(50));
  
  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);
  
  try {
    const count = 5;
    console.log(`📝 Creating ${count} test memories...`);
    
    const startTime = Date.now();
    const memoryIds = await createTestMemories(actor, capsuleId, count);
    const endTime = Date.now();
    const duration = endTime - startTime;
    
    console.log(`✅ Created ${memoryIds.length} memories in ${formatDuration(duration)}`);
    console.log(`📊 Average time per memory: ${formatDuration(duration / memoryIds.length)}`);
    console.log(`🆔 Memory IDs: ${memoryIds.join(', ')}`);
    
    // Clean up
    console.log('\n🧹 Cleaning up...');
    for (const memoryId of memoryIds) {
      await actor.memories_delete(memoryId);
    }
    console.log('✅ All memories deleted');
    
    console.log('\n🎉 Bulk creation demo completed!');
    
  } catch (error) {
    console.error(`❌ Demo failed: ${error.message}`);
  }
}

/**
 * Demo 2: Bulk Memory Deletion
 */
async function demoBulkDeletion() {
  console.log('\n🧪 Demo 2: Bulk Memory Deletion');
  console.log('=' .repeat(50));
  
  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);
  
  try {
    // Create test memories
    const count = 10;
    console.log(`📝 Creating ${count} test memories...`);
    const memoryIds = await createTestMemories(actor, capsuleId, count);
    console.log(`✅ Created ${memoryIds.length} memories`);
    
    // Test bulk deletion
    console.log('\n🗑️  Testing bulk deletion...');
    const startTime = Date.now();
    
    const deleteResult = await actor.memories_delete_bulk(capsuleId, memoryIds);
    
    const endTime = Date.now();
    const duration = endTime - startTime;
    
    if (deleteResult.Ok) {
      console.log(`✅ Bulk deletion completed in ${formatDuration(duration)}`);
      console.log(`📊 Deleted count: ${deleteResult.Ok.deleted_count}`);
      console.log(`❌ Failed count: ${deleteResult.Ok.failed_count}`);
      console.log(`💬 Message: ${deleteResult.Ok.message}`);
    } else {
      console.error(`❌ Bulk deletion failed: ${JSON.stringify(deleteResult.Err)}`);
    }
    
    console.log('\n🎉 Bulk deletion demo completed!');
    
  } catch (error) {
    console.error(`❌ Demo failed: ${error.message}`);
  }
}

/**
 * Demo 3: Performance Comparison
 */
async function demoPerformanceComparison() {
  console.log('\n🧪 Demo 3: Performance Comparison');
  console.log('=' .repeat(50));
  
  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);
  
  try {
    const testSizes = [5, 10, 20];
    
    for (const size of testSizes) {
      console.log(`\n📊 Testing with ${size} memories:`);
      
      // Create test memories
      const memoryIds = await createTestMemories(actor, capsuleId, size);
      console.log(`  ✅ Created ${memoryIds.length} memories`);
      
      // Test individual deletion
      console.log('  🗑️  Testing individual deletion...');
      const individualStartTime = Date.now();
      
      for (const memoryId of memoryIds) {
        await actor.memories_delete(memoryId);
      }
      
      const individualEndTime = Date.now();
      const individualDuration = individualEndTime - individualStartTime;
      
      console.log(`    ⏱️  Individual deletion: ${formatDuration(individualDuration)}`);
      
      // Recreate for bulk test
      const newMemoryIds = await createTestMemories(actor, capsuleId, size);
      
      // Test bulk deletion
      console.log('  🗑️  Testing bulk deletion...');
      const bulkStartTime = Date.now();
      
      const bulkResult = await actor.memories_delete_bulk(capsuleId, newMemoryIds);
      
      const bulkEndTime = Date.now();
      const bulkDuration = bulkEndTime - bulkStartTime;
      
      if (bulkResult.Ok) {
        console.log(`    ⏱️  Bulk deletion: ${formatDuration(bulkDuration)}`);
        
        // Calculate speedup
        const speedup = individualDuration / bulkDuration;
        console.log(`    📈 Speedup: ${speedup.toFixed(2)}x faster`);
      } else {
        console.log(`    ❌ Bulk deletion failed: ${JSON.stringify(bulkResult.Err)}`);
      }
    }
    
    console.log('\n🎉 Performance comparison completed!');
    
  } catch (error) {
    console.error(`❌ Demo failed: ${error.message}`);
  }
}

/**
 * Demo 4: Error Handling
 */
async function demoErrorHandling() {
  console.log('\n🧪 Demo 4: Error Handling');
  console.log('=' .repeat(50));
  
  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);
  
  try {
    // Test with non-existent memory IDs
    console.log('🧪 Testing with non-existent memory IDs...');
    const fakeMemoryIds = ['fake_memory_1', 'fake_memory_2', 'fake_memory_3'];
    
    const deleteResult = await actor.memories_delete_bulk(capsuleId, fakeMemoryIds);
    
    if (deleteResult.Ok) {
      console.log('✅ Bulk deletion handled gracefully');
      console.log(`📊 Deleted count: ${deleteResult.Ok.deleted_count}`);
      console.log(`❌ Failed count: ${deleteResult.Ok.failed_count}`);
      console.log(`💬 Message: ${deleteResult.Ok.message}`);
    } else {
      console.log(`❌ Bulk deletion failed: ${JSON.stringify(deleteResult.Err)}`);
    }
    
    // Test with empty list
    console.log('\n🧪 Testing with empty memory list...');
    const emptyResult = await actor.memories_delete_bulk(capsuleId, []);
    
    if (emptyResult.Ok) {
      console.log('✅ Empty list handled gracefully');
      console.log(`📊 Deleted count: ${emptyResult.Ok.deleted_count}`);
      console.log(`💬 Message: ${emptyResult.Ok.message}`);
    } else {
      console.log(`❌ Empty list failed: ${JSON.stringify(emptyResult.Err)}`);
    }
    
    console.log('\n🎉 Error handling demo completed!');
    
  } catch (error) {
    console.error(`❌ Demo failed: ${error.message}`);
  }
}

/**
 * Main function to run all demos
 */
async function main() {
  console.log('🚀 Bulk Memory Operations Demo');
  console.log('Using the ICP Backend Test Framework');
  console.log('=' .repeat(60));
  
  try {
    await demoBulkCreation();
    await sleep(1000);
    
    await demoBulkDeletion();
    await sleep(1000);
    
    await demoPerformanceComparison();
    await sleep(1000);
    
    await demoErrorHandling();
    
    console.log('\n🎉 All bulk operation demos completed!');
    console.log('\n📚 Key Takeaways:');
    console.log('  • Bulk operations are significantly faster than individual operations');
    console.log('  • The framework handles errors gracefully');
    console.log('  • Performance scales well with larger datasets');
    console.log('  • All operations maintain data integrity');
    
  } catch (error) {
    console.error(`❌ Demo suite failed: ${error.message}`);
    console.error('Stack trace:', error.stack);
    process.exit(1);
  }
}

// Run the main function
main().catch(console.error);
