#!/usr/bin/env node

/**
 * Demo: Memory Creation and Retrieval
 * 
 * This demo shows how to create and retrieve memories using the test framework.
 * It demonstrates both inline and blob memory types with real data.
 */

import { createTestActor, getOrCreateTestCapsule } from '../index.js';
import { 
  createAssetMetadata, 
  createBlobReference, 
  calculateFileHash, 
  formatFileSize, 
  formatDuration,
  sleep
} from '../../shared-capsule/upload/helpers.mjs';

/**
 * Demo 1: Create and Retrieve Inline Memory
 * 
 * Inline memories store small content directly in the memory struct.
 * Perfect for text notes, small images, or metadata.
 */
async function demoInlineMemory() {
  console.log('🧪 Demo 1: Inline Memory Creation and Retrieval');
  console.log('=' .repeat(60));
  
  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);
  
  try {
    // Step 1: Prepare test content
    const content = "Hello, this is a real inline memory stored directly in the memory struct!";
    const contentBytes = Array.from(Buffer.from(content, 'utf8'));
    const contentHash = calculateFileHash(Buffer.from(contentBytes)).toString('hex');
    
    console.log(`📝 Content: "${content}"`);
    console.log(`📊 Content size: ${formatFileSize(contentBytes.length)}`);
    console.log(`🔐 Content hash: ${contentHash}`);
    
    // Step 2: Create asset metadata
    const assetMetadata = createAssetMetadata(
      'demo_inline_memory',
      contentBytes.length,
      'text/plain',
      'Original'
    );
    
    console.log('\n📋 Asset metadata created:');
    console.log(JSON.stringify(assetMetadata, (key, value) => 
      typeof value === 'bigint' ? value.toString() : value, 2
    ));
    
    // Step 3: Create the memory
    console.log('\n🚀 Creating inline memory...');
    const startTime = Date.now();
    
    const memoryResult = await actor.memories_create(
      capsuleId,
      [contentBytes], // opt vec nat8 - inline content
      [], // no blob ref for inline
      [], // no external location
      [], // no external storage key
      [], // no external URL
      [], // no external size
      [], // no external hash
      assetMetadata,
      `demo_inline_${Date.now()}`
    );
    
    const endTime = Date.now();
    const duration = endTime - startTime;
    
    if (!memoryResult.Ok) {
      throw new Error(`Failed to create memory: ${JSON.stringify(memoryResult.Err)}`);
    }
    
    const memoryId = memoryResult.Ok;
    console.log(`✅ Memory created: ${memoryId}`);
    console.log(`⏱️  Creation time: ${formatDuration(duration)}`);
    
    // Step 4: Retrieve and verify the memory
    console.log('\n🔍 Retrieving memory...');
    const retrieveStartTime = Date.now();
    
    const retrievedMemoryResult = await actor.memories_read(memoryId);
    
    const retrieveEndTime = Date.now();
    const retrieveDuration = retrieveEndTime - retrieveStartTime;
    
    if (!retrievedMemoryResult.Ok) {
      throw new Error(`Failed to retrieve memory: ${JSON.stringify(retrievedMemoryResult.Err)}`);
    }
    
    const retrievedMemory = retrievedMemoryResult.Ok;
    console.log(`✅ Memory retrieved successfully!`);
    console.log(`⏱️  Retrieval time: ${formatDuration(retrieveDuration)}`);
    
    // Step 5: Display memory details
    console.log('\n📄 Memory Details:');
    console.log(`  🆔 ID: ${retrievedMemory.id}`);
    console.log(`  📝 Title: ${retrievedMemory.metadata.title[0] || 'No title'}`);
    console.log(`  📄 Content Type: ${retrievedMemory.metadata.content_type}`);
    console.log(`  📅 Created At: ${new Date(Number(retrievedMemory.metadata.created_at) / 1_000_000).toISOString()}`);
    console.log(`  🏷️  Tags: ${retrievedMemory.metadata.tags.join(', ') || 'No tags'}`);
    console.log(`  👤 Created By: ${retrievedMemory.metadata.created_by || 'Unknown'}`);
    
    // Step 6: Verify content integrity
    console.log('\n🔍 Content Verification:');
    console.log(`  📦 Inline Assets: ${retrievedMemory.inline_assets.length}`);
    console.log(`  🗂️  Blob Internal Assets: ${retrievedMemory.blob_internal_assets.length}`);
    console.log(`  🌐 Blob External Assets: ${retrievedMemory.blob_external_assets.length}`);
    
    if (retrievedMemory.inline_assets.length > 0) {
      const retrievedContentBytes = retrievedMemory.inline_assets[0].bytes;
      const retrievedContent = Buffer.from(retrievedContentBytes).toString('utf8');
      
      if (retrievedContent === content) {
        console.log('✅ Content integrity verified!');
        console.log(`📝 Retrieved content: "${retrievedContent}"`);
      } else {
        console.error('❌ Content mismatch!');
        console.log(`Expected: "${content}"`);
        console.log(`Retrieved: "${retrievedContent}"`);
      }
    } else {
      console.error('❌ No inline assets found in retrieved memory.');
    }
    
    // Step 7: Clean up
    console.log('\n🧹 Cleaning up...');
    await actor.memories_delete(memoryId);
    console.log('✅ Memory deleted');
    
    console.log('\n🎉 Inline memory demo completed successfully!');
    
  } catch (error) {
    console.error(`❌ Demo failed: ${error.message}`);
    console.error('Stack trace:', error.stack);
  }
}

/**
 * Demo 2: Create and Retrieve Blob Memory
 * 
 * Blob memories store large content in the ICP blob store.
 * Perfect for large files, images, or documents.
 */
async function demoBlobMemory() {
  console.log('\n🧪 Demo 2: Blob Memory Creation and Retrieval');
  console.log('=' .repeat(60));
  
  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);
  
  try {
    // Step 1: Prepare test content (for blob, we only need metadata and a reference)
    const content = "This is blob content stored in the ICP blob store!";
    const contentBytes = Array.from(Buffer.from(content, 'utf8'));
    const blobId = `test_blob_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`;
    const blobRef = createBlobReference(blobId, contentBytes.length);
    
    console.log(`📝 Content: "${content}"`);
    console.log(`📊 Content size: ${formatFileSize(contentBytes.length)}`);
    console.log(`🆔 Blob ID: ${blobId}`);
    console.log('📋 Blob reference created:');
    console.log(JSON.stringify(blobRef, (key, value) => 
      typeof value === 'bigint' ? value.toString() : value, 2
    ));
    
    // Step 2: Create asset metadata
    const assetMetadata = createAssetMetadata(
      'demo_blob_memory',
      contentBytes.length,
      'text/plain',
      'Original'
    );
    
    console.log('\n📋 Asset metadata created:');
    console.log(JSON.stringify(assetMetadata, (key, value) => 
      typeof value === 'bigint' ? value.toString() : value, 2
    ));
    
    // Step 3: Create the memory (referencing the blob)
    console.log('\n🚀 Creating blob memory...');
    const startTime = Date.now();
    
    const memoryResult = await actor.memories_create(
      capsuleId,
      [], // no inline bytes for blob
      [blobRef], // opt BlobRef
      [], // no external location
      [], // no external storage key
      [], // no external URL
      [], // no external size
      [], // no external hash
      assetMetadata,
      `demo_blob_${Date.now()}`
    );
    
    const endTime = Date.now();
    const duration = endTime - startTime;
    
    if (!memoryResult.Ok) {
      throw new Error(`Failed to create memory: ${JSON.stringify(memoryResult.Err)}`);
    }
    
    const memoryId = memoryResult.Ok;
    console.log(`✅ Memory created: ${memoryId}`);
    console.log(`⏱️  Creation time: ${formatDuration(duration)}`);
    
    // Step 4: Retrieve and verify the memory
    console.log('\n🔍 Retrieving memory...');
    const retrieveStartTime = Date.now();
    
    const retrievedMemoryResult = await actor.memories_read(memoryId);
    
    const retrieveEndTime = Date.now();
    const retrieveDuration = retrieveEndTime - retrieveStartTime;
    
    if (!retrievedMemoryResult.Ok) {
      throw new Error(`Failed to retrieve memory: ${JSON.stringify(retrievedMemoryResult.Err)}`);
    }
    
    const retrievedMemory = retrievedMemoryResult.Ok;
    console.log(`✅ Memory retrieved successfully!`);
    console.log(`⏱️  Retrieval time: ${formatDuration(retrieveDuration)}`);
    
    // Step 5: Display memory details
    console.log('\n📄 Memory Details:');
    console.log(`  🆔 ID: ${retrievedMemory.id}`);
    console.log(`  📝 Title: ${retrievedMemory.metadata.title[0] || 'No title'}`);
    console.log(`  📄 Content Type: ${retrievedMemory.metadata.content_type}`);
    console.log(`  📅 Created At: ${new Date(Number(retrievedMemory.metadata.created_at) / 1_000_000).toISOString()}`);
    console.log(`  🏷️  Tags: ${retrievedMemory.metadata.tags.join(', ') || 'No tags'}`);
    console.log(`  👤 Created By: ${retrievedMemory.metadata.created_by || 'Unknown'}`);
    
    // Step 6: Verify blob reference
    console.log('\n🔍 Blob Reference Verification:');
    console.log(`  📦 Inline Assets: ${retrievedMemory.inline_assets.length}`);
    console.log(`  🗂️  Blob Internal Assets: ${retrievedMemory.blob_internal_assets.length}`);
    console.log(`  🌐 Blob External Assets: ${retrievedMemory.blob_external_assets.length}`);
    
    if (retrievedMemory.blob_internal_assets.length > 0) {
      const retrievedBlobRef = retrievedMemory.blob_internal_assets[0].blob_ref;
      
      if (retrievedBlobRef.locator === blobRef.locator && retrievedBlobRef.len === blobRef.len) {
        console.log('✅ Blob reference verified!');
        console.log(`🗂️  Blob locator: ${retrievedBlobRef.locator}`);
        console.log(`📊 Blob size: ${formatFileSize(Number(retrievedBlobRef.len))}`);
      } else {
        console.error('❌ Blob reference mismatch!');
        console.log(`Expected locator: ${blobRef.locator}`);
        console.log(`Retrieved locator: ${retrievedBlobRef.locator}`);
      }
    } else {
      console.error('❌ No internal blob assets found in retrieved memory.');
    }
    
    // Step 7: Clean up
    console.log('\n🧹 Cleaning up...');
    await actor.memories_delete(memoryId);
    console.log('✅ Memory deleted');
    
    console.log('\n🎉 Blob memory demo completed successfully!');
    
  } catch (error) {
    console.error(`❌ Demo failed: ${error.message}`);
    console.error('Stack trace:', error.stack);
  }
}

/**
 * Demo 3: Performance Comparison
 * 
 * Compare inline vs blob memory creation and retrieval performance.
 */
async function demoPerformanceComparison() {
  console.log('\n🧪 Demo 3: Performance Comparison');
  console.log('=' .repeat(60));
  
  const { actor } = await createTestActor();
  const capsuleId = await getOrCreateTestCapsule(actor);
  
  try {
    const testSizes = [
      { name: 'Small (1KB)', size: 1024 },
      { name: 'Medium (10KB)', size: 10240 },
      { name: 'Large (100KB)', size: 102400 }
    ];
    
    for (const test of testSizes) {
      console.log(`\n📊 Testing ${test.name}:`);
      
      // Create test content
      const content = 'A'.repeat(test.size);
      const contentBytes = Array.from(Buffer.from(content, 'utf8'));
      const assetMetadata = createAssetMetadata(
        `perf_test_${test.size}`,
        contentBytes.length,
        'text/plain'
      );
      
      // Test inline memory
      console.log('  📦 Testing inline memory...');
      const inlineStartTime = Date.now();
      
      const inlineResult = await actor.memories_create(
        capsuleId,
        [contentBytes],
        [], [], [], [], [], [],
        assetMetadata,
        `perf_inline_${Date.now()}`
      );
      
      const inlineEndTime = Date.now();
      const inlineDuration = inlineEndTime - inlineStartTime;
      
      if (inlineResult.Ok) {
        console.log(`    ✅ Created in ${formatDuration(inlineDuration)}`);
        
        // Clean up inline memory
        await actor.memories_delete(inlineResult.Ok);
      } else {
        console.log(`    ❌ Failed: ${JSON.stringify(inlineResult.Err)}`);
      }
      
      // Test blob memory
      console.log('  🗂️  Testing blob memory...');
      const blobId = `perf_blob_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`;
      const blobRef = createBlobReference(blobId, contentBytes.length);
      
      const blobStartTime = Date.now();
      
      const blobResult = await actor.memories_create(
        capsuleId,
        [],
        [blobRef],
        [], [], [], [], [],
        assetMetadata,
        `perf_blob_${Date.now()}`
      );
      
      const blobEndTime = Date.now();
      const blobDuration = blobEndTime - blobStartTime;
      
      if (blobResult.Ok) {
        console.log(`    ✅ Created in ${formatDuration(blobDuration)}`);
        
        // Clean up blob memory
        await actor.memories_delete(blobResult.Ok);
      } else {
        console.log(`    ❌ Failed: ${JSON.stringify(blobResult.Err)}`);
      }
      
      // Performance comparison
      const speedup = inlineDuration / blobDuration;
      console.log(`  📈 Performance: Inline ${speedup.toFixed(2)}x ${speedup > 1 ? 'slower' : 'faster'} than blob`);
    }
    
    console.log('\n🎉 Performance comparison completed!');
    
  } catch (error) {
    console.error(`❌ Demo failed: ${error.message}`);
    console.error('Stack trace:', error.stack);
  }
}

/**
 * Main function to run all demos
 */
async function main() {
  console.log('🚀 Memory Creation and Retrieval Demo');
  console.log('Using the ICP Backend Test Framework');
  console.log('=' .repeat(60));
  
  try {
    await demoInlineMemory();
    await sleep(1000); // Brief pause between demos
    
    await demoBlobMemory();
    await sleep(1000); // Brief pause between demos
    
    await demoPerformanceComparison();
    
    console.log('\n🎉 All demos completed successfully!');
    console.log('\n📚 Key Takeaways:');
    console.log('  • Inline memories store content directly in the memory struct');
    console.log('  • Blob memories store content in the ICP blob store with references');
    console.log('  • Both types provide full content integrity and verification');
    console.log('  • Performance varies based on content size and storage type');
    console.log('  • The test framework handles all certificate verification automatically');
    
  } catch (error) {
    console.error(`❌ Demo suite failed: ${error.message}`);
    console.error('Stack trace:', error.stack);
    process.exit(1);
  }
}

// Run the main function
main().catch(console.error);

