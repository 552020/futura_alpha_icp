#!/usr/bin/env node

/**
 * Simple Demo: Memory Creation and Retrieval
 * 
 * A quick demo showing the basics of creating and retrieving memories.
 * Perfect for testing the framework and understanding the flow.
 */

import { createTestActor, getOrCreateTestCapsule } from '../index.js';
import { 
  createAssetMetadata, 
  formatFileSize, 
  formatDuration
} from '../../shared-capsule/upload/helpers.mjs';

/**
 * Simple memory creation and retrieval demo
 */
async function simpleMemoryDemo() {
  console.log('🚀 Simple Memory Demo');
  console.log('=' .repeat(40));
  
  try {
    // Step 1: Create actor and capsule
    console.log('1️⃣ Creating test actor and capsule...');
    const { actor } = await createTestActor();
    const capsuleId = await getOrCreateTestCapsule(actor);
    console.log(`✅ Capsule ready: ${capsuleId}`);
    
    // Step 2: Prepare test content
    console.log('\n2️⃣ Preparing test content...');
    const content = "Hello, this is a simple test memory!";
    const contentBytes = Array.from(Buffer.from(content, 'utf8'));
    console.log(`📝 Content: "${content}"`);
    console.log(`📊 Size: ${formatFileSize(contentBytes.length)}`);
    
    // Step 3: Create asset metadata
    console.log('\n3️⃣ Creating asset metadata...');
    const assetMetadata = createAssetMetadata(
      'simple_test_memory',
      contentBytes.length,
      'text/plain'
    );
    console.log('✅ Asset metadata created');
    
    // Step 4: Create the memory
    console.log('\n4️⃣ Creating memory...');
    const startTime = Date.now();
    
    const memoryResult = await actor.memories_create(
      capsuleId,
      [contentBytes], // inline content
      [], // no blob ref
      [], // no external location
      [], // no external storage key
      [], // no external URL
      [], // no external size
      [], // no external hash
      assetMetadata,
      `simple_${Date.now()}`
    );
    
    const endTime = Date.now();
    const duration = endTime - startTime;
    
    if (!memoryResult.Ok) {
      throw new Error(`Failed to create memory: ${JSON.stringify(memoryResult.Err)}`);
    }
    
    const memoryId = memoryResult.Ok;
    console.log(`✅ Memory created: ${memoryId}`);
    console.log(`⏱️  Time: ${formatDuration(duration)}`);
    
    // Step 5: Retrieve the memory
    console.log('\n5️⃣ Retrieving memory...');
    const retrieveStartTime = Date.now();
    
    const retrievedMemoryResult = await actor.memories_read(memoryId);
    
    const retrieveEndTime = Date.now();
    const retrieveDuration = retrieveEndTime - retrieveStartTime;
    
    if (!retrievedMemoryResult.Ok) {
      throw new Error(`Failed to retrieve memory: ${JSON.stringify(retrievedMemoryResult.Err)}`);
    }
    
    const retrievedMemory = retrievedMemoryResult.Ok;
    console.log(`✅ Memory retrieved successfully!`);
    console.log(`⏱️  Time: ${formatDuration(retrieveDuration)}`);
    
    // Step 6: Display results
    console.log('\n6️⃣ Memory Details:');
    console.log(`  🆔 ID: ${retrievedMemory.id}`);
    console.log(`  📝 Title: ${retrievedMemory.metadata.title[0] || 'No title'}`);
    console.log(`  📄 Type: ${retrievedMemory.metadata.memory_type}`);
    console.log(`  📅 Created: ${new Date(Number(retrievedMemory.metadata.created_at) / 1_000_000).toISOString()}`);
    console.log(`  📦 Inline Assets: ${retrievedMemory.inline_assets.length}`);
    console.log(`  🗂️  Blob Assets: ${retrievedMemory.blob_internal_assets.length}`);
    
    // Step 7: Verify content
    if (retrievedMemory.inline_assets.length > 0) {
      const retrievedContent = Buffer.from(retrievedMemory.inline_assets[0].bytes).toString('utf8');
      if (retrievedContent === content) {
        console.log('✅ Content verified successfully!');
      } else {
        console.error('❌ Content mismatch!');
      }
    }
    
    // Step 8: Clean up
    console.log('\n7️⃣ Cleaning up...');
    await actor.memories_delete(memoryId);
    console.log('✅ Memory deleted');
    
    console.log('\n🎉 Demo completed successfully!');
    console.log('\n📚 What we learned:');
    console.log('  • How to create a test actor with proper certificate handling');
    console.log('  • How to create a test capsule for memory storage');
    console.log('  • How to create inline memories with real content');
    console.log('  • How to retrieve and verify memory content');
    console.log('  • How to clean up test data');
    
  } catch (error) {
    console.error(`❌ Demo failed: ${error.message}`);
    console.error('Stack trace:', error.stack);
    process.exit(1);
  }
}

// Run the demo
simpleMemoryDemo().catch(console.error);
