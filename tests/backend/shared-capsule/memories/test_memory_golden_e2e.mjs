#!/usr/bin/env node

/**
 * Golden E2E Test: Memory Creation and Retrieval Workflow
 * 
 * This is the "golden path" test that validates the complete memory workflow:
 * 1. Create memory with real content
 * 2. Retrieve memory using the returned ID
 * 3. Verify content integrity
 * 4. Clean up properly
 * 
 * This test serves as a guardrail against interface regressions and ensures
 * the core memory API works end-to-end.
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { loadDfxIdentity } from "../upload/ic-identity.js";
import fetch from "node-fetch";
import crypto from "crypto";
import { createAssetMetadata } from "../upload/helpers.mjs";

// Import the backend interface
import { idlFactory } from "../../../../.dfx/local/canisters/backend/service.did.js";

// Test configuration
const TEST_NAME = "Golden E2E Memory Workflow Test";
const HOST = process.env.IC_HOST || "http://127.0.0.1:4943";
const IS_MAINNET = process.env.IC_HOST === "https://ic0.app" || process.env.IC_HOST === "https://icp0.io";

// Global backend instance
let backend;

// Helper functions
function echoInfo(message) {
  console.log(`ℹ️  ${message}`);
}

function echoPass(message) {
  console.log(`✅ ${message}`);
}

function echoFail(message) {
  console.log(`❌ ${message}`);
}

function echoError(message) {
  console.error(`💥 ${message}`);
}

function echoHeader(message) {
  console.log(`\n🎯 ${message}`);
  console.log("=" .repeat(60));
}

// Test configuration
const TEST_CONFIG = {
  content: "Hello, this is a golden E2E test memory with real content!",
  name: "golden_e2e_test",
  description: "Test memory for golden E2E workflow validation",
  tags: ["test", "e2e", "golden"],
  mimeType: "text/plain"
};

/**
 * Create test agent with proper certificate handling
 */
async function createTestAgent() {
  const identity = loadDfxIdentity();
  const agent = new HttpAgent({
    host: HOST,
    identity,
    fetch,
  });
  
  // CRITICAL for local replica: trust local root key
  if (!IS_MAINNET) {
    await agent.fetchRootKey();
  }
  
  return agent;
}

/**
 * Create test actor
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
 * Create test capsule
 */
async function createTestCapsule() {
  echoInfo("Creating test capsule...");
  
  const capsuleResult = await backend.capsules_create([]);
  
  if (!capsuleResult.Ok) {
    throw new Error(`Failed to create capsule: ${JSON.stringify(capsuleResult.Err)}`);
  }
  
  const capsuleId = capsuleResult.Ok.id;
  echoPass(`Capsule created: ${capsuleId}`);
  
  return capsuleId;
}

/**
 * Create asset metadata for the test using the working helper
 */
function createTestAssetMetadata() {
  const contentBytes = Array.from(Buffer.from(TEST_CONFIG.content, 'utf8'));
  
  // Use the working helper function
  return createAssetMetadata(
    TEST_CONFIG.name,
    contentBytes.length,
    TEST_CONFIG.mimeType
  );
}

/**
 * Golden E2E Test: Complete Memory Workflow
 */
async function testGoldenMemoryWorkflow() {
  echoHeader("Golden E2E Memory Workflow Test");
  
  let capsuleId = null;
  let memoryId = null;
  
  try {
    // Step 1: Create test capsule
    echoInfo("Step 1: Creating test capsule...");
    capsuleId = await createTestCapsule();
    
    // Step 2: Prepare test content
    echoInfo("Step 2: Preparing test content...");
    const content = TEST_CONFIG.content;
    const contentBytes = Array.from(Buffer.from(content, 'utf8'));
    const assetMetadata = createTestAssetMetadata();
    
    echoInfo(`Content: "${content}"`);
    echoInfo(`Content size: ${contentBytes.length} bytes`);
    echoInfo(`Asset metadata: ${JSON.stringify(assetMetadata, (key, value) => 
      typeof value === 'bigint' ? value.toString() : value, 2)}`);
    
    // Step 3: Create memory
    echoInfo("Step 3: Creating memory...");
    const startTime = Date.now();
    
    const memoryResult = await backend.memories_create(
      capsuleId,
      [contentBytes], // opt vec nat8 - inline content
      [], // no blob ref for inline
      [], // no external location
      [], // no external storage key
      [], // no external URL
      [], // no external size
      [], // no external hash
      assetMetadata,
      `golden_e2e_${Date.now()}`
    );
    
    const endTime = Date.now();
    const duration = endTime - startTime;
    
    if (!memoryResult.Ok) {
      throw new Error(`Failed to create memory: ${JSON.stringify(memoryResult.Err)}`);
    }
    
    memoryId = memoryResult.Ok;
    echoPass(`Memory created: ${memoryId}`);
    echoInfo(`Creation time: ${duration}ms`);
    
    // Step 4: Retrieve memory
    echoInfo("Step 4: Retrieving memory...");
    const retrieveStartTime = Date.now();
    
    const retrievedMemoryResult = await backend.memories_read(memoryId);
    
    const retrieveEndTime = Date.now();
    const retrieveDuration = retrieveEndTime - retrieveStartTime;
    
    if (!retrievedMemoryResult.Ok) {
      throw new Error(`Failed to retrieve memory: ${JSON.stringify(retrievedMemoryResult.Err)}`);
    }
    
    const retrievedMemory = retrievedMemoryResult.Ok;
    echoPass(`Memory retrieved successfully!`);
    echoInfo(`Retrieval time: ${retrieveDuration}ms`);
    
    // Step 5: Verify memory details
    echoInfo("Step 5: Verifying memory details...");
    echoInfo(`  🆔 ID: ${retrievedMemory.id}`);
    echoInfo(`  📝 Title: ${retrievedMemory.metadata.title[0] || 'No title'}`);
    echoInfo(`  📄 Content Type: ${retrievedMemory.metadata.content_type}`);
    echoInfo(`  📅 Created At: ${new Date(Number(retrievedMemory.metadata.created_at) / 1_000_000).toISOString()}`);
    echoInfo(`  🏷️  Tags: ${retrievedMemory.metadata.tags.join(', ') || 'No tags'}`);
    echoInfo(`  👤 Created By: ${retrievedMemory.metadata.created_by || 'Unknown'}`);
    
    // Step 6: Verify content integrity
    echoInfo("Step 6: Verifying content integrity...");
    echoInfo(`  📦 Inline Assets: ${retrievedMemory.inline_assets.length}`);
    echoInfo(`  🗂️  Blob Internal Assets: ${retrievedMemory.blob_internal_assets.length}`);
    echoInfo(`  🌐 Blob External Assets: ${retrievedMemory.blob_external_assets.length}`);
    
    if (retrievedMemory.inline_assets.length > 0) {
      const inlineAsset = retrievedMemory.inline_assets[0];
      echoInfo(`Debug: Inline asset structure: ${JSON.stringify(inlineAsset, (key, value) => 
        typeof value === 'bigint' ? value.toString() : value, 2)}`);
      
      const retrievedContentBytes = inlineAsset.bytes;
      const retrievedContent = Buffer.from(retrievedContentBytes).toString('utf8');
      
      echoInfo(`Debug: Retrieved content bytes: ${JSON.stringify(retrievedContentBytes)}`);
      echoInfo(`Debug: Retrieved content: "${retrievedContent}"`);
      echoInfo(`Debug: Expected content: "${content}"`);
      
      if (retrievedContent === content) {
        echoPass("Content integrity verified successfully!");
        echoInfo(`Retrieved content: "${retrievedContent}"`);
      } else {
        echoFail("Content integrity verification failed!");
        echoError(`Expected: "${content}"`);
        echoError(`Retrieved: "${retrievedContent}"`);
        throw new Error("Content mismatch detected");
      }
    } else {
      throw new Error("No inline assets found in retrieved memory");
    }
    
    // Step 7: Verify memory ID consistency
    echoInfo("Step 7: Verifying memory ID consistency...");
    if (retrievedMemory.id === memoryId) {
      echoPass("Memory ID consistency verified!");
    } else {
      echoFail("Memory ID consistency failed!");
      echoError(`Expected ID: ${memoryId}`);
      echoError(`Retrieved ID: ${retrievedMemory.id}`);
      throw new Error("Memory ID mismatch detected");
    }
    
    echoPass("🎉 Golden E2E test completed successfully!");
    
    return { capsuleId, memoryId };
    
  } catch (error) {
    echoError(`Golden E2E test failed: ${error.message}`);
    throw error;
  }
}

/**
 * Cleanup test data
 */
async function cleanupTestData(capsuleId, memoryId) {
  echoInfo("Cleaning up test data...");
  
  try {
    if (memoryId) {
      const deleteResult = await backend.memories_delete(memoryId);
      if (deleteResult.Ok) {
        echoPass("Memory deleted successfully");
      } else {
        echoError(`Failed to delete memory: ${JSON.stringify(deleteResult.Err)}`);
      }
    }
    
    if (capsuleId) {
      const deleteCapsuleResult = await backend.capsules_delete(capsuleId);
      if (deleteCapsuleResult.Ok) {
        echoPass("Capsule deleted successfully");
      } else {
        echoError(`Failed to delete capsule: ${JSON.stringify(deleteCapsuleResult.Err)}`);
      }
    }
    
    echoPass("Cleanup completed");
    
  } catch (error) {
    echoError(`Cleanup failed: ${error.message}`);
  }
}

/**
 * Main test execution
 */
async function main() {
  echoHeader(TEST_NAME);
  
  let capsuleId = null;
  let memoryId = null;
  
  try {
    // Initialize backend
    echoInfo("Initializing backend connection...");
    backend = await createTestActor();
    echoPass("Backend connection established");
    
    // Run golden E2E test
    const result = await testGoldenMemoryWorkflow();
    capsuleId = result.capsuleId;
    memoryId = result.memoryId;
    
    echoPass("🎉 All tests passed successfully!");
    
  } catch (error) {
    echoError(`Test suite failed: ${error.message}`);
    console.error("Stack trace:", error.stack);
    process.exit(1);
  } finally {
    // Always cleanup
    await cleanupTestData(capsuleId, memoryId);
  }
}

// Run the test
main().catch(console.error);
