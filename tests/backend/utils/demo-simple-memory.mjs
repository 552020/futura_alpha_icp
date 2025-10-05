#!/usr/bin/env node

/**
 * Demo: Simple Memory Creation and Retrieval
 * 
 * Uses the working upload helpers to demonstrate:
 * 1. Creating a real memory with actual data
 * 2. Verifying the memory exists and contains expected data
 * 3. Showing the framework in action
 */

import { HttpAgent } from "@dfinity/agent";
import { Actor } from "@dfinity/agent";
import { loadDfxIdentity } from "../shared-capsule/upload/ic-identity.js";
import { 
  formatFileSize, 
  formatDuration, 
  formatUploadSpeed,
  createAssetMetadata,
  createBlobReference,
  generateFileId,
  calculateFileHash,
  handleUploadError,
  validateUploadResponse,
  sleep
} from "../shared-capsule/upload/helpers.mjs";

// Import the backend interface (use the working one)
import { idlFactory } from "../../../src/nextjs/src/ic/declarations/backend/backend.did.js";

/**
 * Create a test agent using the working identity system
 */
async function createTestAgent() {
  const identity = loadDfxIdentity();
  const agent = new HttpAgent({
    host: "http://127.0.0.1:4943",
    identity,
    fetch, // Use node-fetch import
  });
  
  // CRITICAL: Fetch root key for local replica
  await agent.fetchRootKey();
  
  return agent;
}

/**
 * Create a test actor
 */
async function createTestActor() {
  const agent = await createTestAgent();
  const canisterId = process.env.BACKEND_CANISTER_ID || "uxrrr-q7777-77774-qaaaq-cai";
  
  return Actor.createActor(idlFactory, {
    agent,
    canisterId,
  });
}

/**
 * Demo 1: Create and Verify Inline Memory
 */
async function demoInlineMemory() {
  console.log("🧪 Demo 1: Inline Memory Creation and Verification");
  
  const actor = await createTestActor();
  
  try {
    // Step 1: Create a test capsule
    console.log("Creating test capsule...");
    const capsuleResult = await actor.capsules_create([]);
    
    if (!capsuleResult.Ok) {
      throw new Error(`Failed to create capsule: ${JSON.stringify(capsuleResult.Err)}`);
    }
    
    const capsuleId = capsuleResult.Ok.id;
    console.log(`✅ Capsule created: ${capsuleId}`);
    
    // Step 2: Create test content
    const testContent = "Hello, this is real content stored inline in the memory!";
    const contentBytes = new TextEncoder().encode(testContent);
    const contentHash = calculateFileHash(contentBytes);
    
    console.log(`Content: "${testContent}"`);
    console.log(`Content size: ${formatFileSize(contentBytes.length)}`);
    console.log(`Content hash: ${contentHash.toString('hex')}`);
    
    // Step 3: Create asset metadata
    const assetMetadata = createAssetMetadata(
      "demo_inline_memory",
      contentBytes.length,
      "text/plain",
      "Original"
    );
    
    console.log("Asset metadata created:");
    console.log(JSON.stringify(assetMetadata, (key, value) => 
      typeof value === "bigint" ? value.toString() : value, 2
    ));
    
    // Step 4: Create the memory
    console.log("Creating inline memory...");
    const memoryResult = await actor.memories_create(
      capsuleId,
      [contentBytes], // Wrap in array for opt vec nat8
      [], // no blob ref for inline
      [], // no external location
      [], // no external storage key
      [], // no external URL
      [], // no external size
      [], // no external hash
      assetMetadata,
      `demo_inline_${Date.now()}`
    );
    
    if (!memoryResult.Ok) {
      throw new Error(`Failed to create memory: ${JSON.stringify(memoryResult.Err)}`);
    }
    
    const memoryId = memoryResult.Ok;
    console.log(`✅ Memory created: ${memoryId}`);
    
    // Step 5: Verify the memory exists
    console.log("Verifying memory exists...");
    const memoryInfo = await actor.memories_read(memoryId);
    
    if (!memoryInfo.Ok) {
      throw new Error(`Failed to read memory: ${JSON.stringify(memoryInfo.Err)}`);
    }
    
    const memory = memoryInfo.Ok;
    console.log("✅ Memory exists in the system");
    
    // Step 6: Verify memory details
    console.log("Memory Details:");
    console.log(`  ID: ${memory.id}`);
    console.log(`  Title: ${memory.metadata.title}`);
    console.log(`  Description: ${memory.metadata.description}`);
    console.log(`  Content Type: ${memory.metadata.content_type}`);
    console.log(`  Created At: ${new Date(Number(memory.metadata.created_at) / 1000000).toISOString()}`);
    console.log(`  Inline Assets: ${memory.inline_assets.length}`);
    console.log(`  Blob Internal Assets: ${memory.blob_internal_assets.length}`);
    console.log(`  Blob External Assets: ${memory.blob_external_assets.length}`);
    
    // Step 7: Verify the content
    if (memory.inline_assets.length > 0) {
      const inlineAsset = memory.inline_assets[0];
      const retrievedContent = new TextDecoder().decode(inlineAsset.bytes);
      console.log(`Retrieved content: "${retrievedContent}"`);
      
      if (retrievedContent === testContent) {
        console.log("✅ Content matches exactly!");
      } else {
        console.log("❌ Content does not match");
      }
    } else {
      console.log("❌ No inline assets found");
    }
    
    // Step 8: Clean up
    console.log("Cleaning up...");
    const deleteResult = await actor.memories_delete(memoryId);
    if (deleteResult.Ok) {
      console.log("✅ Memory deleted successfully");
    } else {
      console.log(`❌ Failed to delete memory: ${JSON.stringify(deleteResult.Err)}`);
    }
    
    const capsuleDeleteResult = await actor.capsules_delete(capsuleId);
    if (capsuleDeleteResult.Ok) {
      console.log("✅ Capsule deleted successfully");
    } else {
      console.log(`❌ Failed to delete capsule: ${JSON.stringify(capsuleDeleteResult.Err)}`);
    }
    
    console.log("🎉 Demo completed successfully!");
    
  } catch (error) {
    console.error("❌ Demo failed:", error.message);
    throw error;
  }
}

/**
 * Demo 2: Create and Verify Blob Memory
 */
async function demoBlobMemory() {
  console.log("🧪 Demo 2: Blob Memory Creation and Verification");
  
  const actor = await createTestActor();
  
  try {
    // Step 1: Create a test capsule
    console.log("Creating test capsule...");
    const capsuleResult = await actor.capsules_create([]);
    
    if (!capsuleResult.Ok) {
      throw new Error(`Failed to create capsule: ${JSON.stringify(capsuleResult.Err)}`);
    }
    
    const capsuleId = capsuleResult.Ok.id;
    console.log(`✅ Capsule created: ${capsuleId}`);
    
    // Step 2: Create test content
    const testContent = "This is blob content stored in ICP blob store!";
    const contentBytes = new TextEncoder().encode(testContent);
    const contentHash = calculateFileHash(contentBytes);
    
    console.log(`Content: "${testContent}"`);
    console.log(`Content size: ${formatFileSize(contentBytes.length)}`);
    console.log(`Content hash: ${contentHash.toString('hex')}`);
    
    // Step 3: Create blob reference
    const blobId = generateFileId("blob");
    const blobRef = createBlobReference(blobId, contentBytes.length);
    
    console.log("Blob reference created:");
    console.log(JSON.stringify(blobRef, (key, value) => 
      typeof value === "bigint" ? value.toString() : value, 2
    ));
    
    // Step 4: Create asset metadata
    const assetMetadata = createAssetMetadata(
      "demo_blob_memory",
      contentBytes.length,
      "text/plain",
      "Original"
    );
    
    // Step 5: Create the memory
    console.log("Creating blob memory...");
    const memoryResult = await actor.memories_create(
      capsuleId,
      null, // no inline bytes for blob
      blobRef,
      null, // no external location
      null, // no external storage key
      null, // no external URL
      null, // no external size
      null, // no external hash
      assetMetadata,
      `demo_blob_${Date.now()}`
    );
    
    if (!memoryResult.Ok) {
      throw new Error(`Failed to create memory: ${JSON.stringify(memoryResult.Err)}`);
    }
    
    const memoryId = memoryResult.Ok;
    console.log(`✅ Blob memory created: ${memoryId}`);
    
    // Step 6: Verify the memory exists
    console.log("Verifying blob memory exists...");
    const memoryInfo = await actor.memories_read(memoryId);
    
    if (!memoryInfo.Ok) {
      throw new Error(`Failed to read memory: ${JSON.stringify(memoryInfo.Err)}`);
    }
    
    const memory = memoryInfo.Ok;
    console.log("✅ Blob memory exists in the system");
    
    // Step 7: Verify memory details
    console.log("Blob Memory Details:");
    console.log(`  ID: ${memory.id}`);
    console.log(`  Title: ${memory.metadata.title}`);
    console.log(`  Inline Assets: ${memory.inline_assets.length}`);
    console.log(`  Blob Internal Assets: ${memory.blob_internal_assets.length}`);
    console.log(`  Blob External Assets: ${memory.blob_external_assets.length}`);
    
    // Step 8: Verify blob assets
    if (memory.blob_internal_assets.length > 0) {
      const blobAsset = memory.blob_internal_assets[0];
      console.log(`Blob reference: ${blobAsset.blob_ref.locator}`);
      console.log(`Blob length: ${blobAsset.blob_ref.len}`);
      console.log("✅ Blob assets found");
    } else {
      console.log("❌ No blob assets found");
    }
    
    // Step 9: Clean up
    console.log("Cleaning up...");
    const deleteResult = await actor.memories_delete(memoryId);
    if (deleteResult.Ok) {
      console.log("✅ Blob memory deleted successfully");
    } else {
      console.log(`❌ Failed to delete blob memory: ${JSON.stringify(deleteResult.Err)}`);
    }
    
    const capsuleDeleteResult = await actor.capsules_delete(capsuleId);
    if (capsuleDeleteResult.Ok) {
      console.log("✅ Capsule deleted successfully");
    } else {
      console.log(`❌ Failed to delete capsule: ${JSON.stringify(capsuleDeleteResult.Err)}`);
    }
    
    console.log("🎉 Blob memory demo completed successfully!");
    
  } catch (error) {
    console.error("❌ Blob memory demo failed:", error.message);
    throw error;
  }
}

/**
 * Demo 3: Performance Measurement
 */
async function demoPerformanceMeasurement() {
  console.log("🧪 Demo 3: Performance Measurement");
  
  const actor = await createTestActor();
  
  try {
    // Create test capsule
    const capsuleResult = await actor.capsules_create([]);
    if (!capsuleResult.Ok) {
      throw new Error(`Failed to create capsule: ${JSON.stringify(capsuleResult.Err)}`);
    }
    const capsuleId = capsuleResult.Ok.id;
    
    // Measure memory creation performance
    const startTime = Date.now();
    
    const testContent = "Performance test content";
    const contentBytes = new TextEncoder().encode(testContent);
    const assetMetadata = createAssetMetadata(
      "performance_test_memory",
      contentBytes.length,
      "text/plain",
      "Original"
    );
    
    const memoryResult = await actor.memories_create(
      capsuleId,
      contentBytes,
      null, null, null, null, null, null,
      assetMetadata,
      `performance_test_${Date.now()}`
    );
    
    const endTime = Date.now();
    const duration = endTime - startTime;
    
    if (!memoryResult.Ok) {
      throw new Error(`Failed to create memory: ${JSON.stringify(memoryResult.Err)}`);
    }
    
    const memoryId = memoryResult.Ok;
    
    console.log(`✅ Memory created in ${formatDuration(duration)}`);
    console.log(`Performance: ${formatUploadSpeed(contentBytes.length, duration)}`);
    
    // Clean up
    await actor.memories_delete(memoryId);
    await actor.capsules_delete(capsuleId);
    
    console.log("🎉 Performance measurement completed!");
    
  } catch (error) {
    console.error("❌ Performance demo failed:", error.message);
    throw error;
  }
}

/**
 * Main demo function
 */
async function main() {
  console.log("🚀 Simple Memory Creation and Retrieval Demo");
  console.log("Using working upload helpers from the upload directory");
  console.log("=" * 60);
  
  try {
    await demoInlineMemory();
    console.log("\n" + "=" * 60 + "\n");
    
    await demoBlobMemory();
    console.log("\n" + "=" * 60 + "\n");
    
    await demoPerformanceMeasurement();
    
    console.log("\n🎉 All demos completed successfully!");
    console.log("\nThis demonstrates how the test framework provides:");
    console.log("✅ Real data creation (actual capsules and memories)");
    console.log("✅ Meaningful operations (real business logic)");
    console.log("✅ State verification (confirming operations worked)");
    console.log("✅ Performance measurement (tracking real performance)");
    console.log("✅ Automatic cleanup (removing test data)");
    
  } catch (error) {
    console.error("❌ Demo failed:", error.message);
    process.exit(1);
  }
}

// Run demo if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}
