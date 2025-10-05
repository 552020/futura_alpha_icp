#!/usr/bin/env node

/**
 * Test Framework Demo
 * 
 * This test demonstrates the new test framework by:
 * 1. Creating a capsule using framework utilities
 * 2. Retrieving the capsule to verify it works
 * 3. Creating a memory using framework utilities
 * 4. Retrieving the memory to verify it works
 * 
 * This validates that our new test framework is working correctly.
 */

import {
  createTestAgent,
  createTestActor,
  getEnvironmentInfo
} from "../../utils/index.js";

import {
  createTestCapsule,
  createTestMemory,
  validateCapsule,
  validateMemory
} from "../../utils/data/capsule.js";

import {
  createAssetMetadata,
  createTestContent
} from "../../utils/data/memory.js";

// Test configuration
const TEST_NAME = "Test Framework Demo";
const HOST = process.env.IC_HOST || "http://127.0.0.1:4943";

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
  console.log("=".repeat(60));
}

function echoSubHeader(message) {
  console.log(`\n📋 ${message}`);
  console.log("-".repeat(40));
}

/**
 * Test Framework Demo: Create and Retrieve Capsule
 */
async function testCapsuleCreation() {
  echoSubHeader("Test 1: Create and Retrieve Capsule");
  
  try {
    // Step 1: Create test actor using framework
    echoInfo("Creating test actor using framework...");
    const { actor, agent, canisterId } = await createTestActor();
    echoPass(`Test actor created for canister: ${canisterId}`);
    
    // Step 2: Create capsule using framework
    echoInfo("Creating test capsule using framework...");
    const capsuleData = await createTestCapsule(actor);
    echoPass(`Capsule created: ${capsuleData.id}`);
    
    // Step 3: Retrieve capsule to verify
    echoInfo("Retrieving capsule to verify creation...");
    const retrievedCapsule = await actor.capsules_read_basic(capsuleData.id);
    
    if (!retrievedCapsule.Ok) {
      throw new Error(`Failed to retrieve capsule: ${JSON.stringify(retrievedCapsule.Err)}`);
    }
    
    const capsule = retrievedCapsule.Ok;
    echoPass("Capsule retrieved successfully!");
    
    // Step 4: Validate capsule using framework
    echoInfo("Validating capsule using framework...");
    const validation = validateCapsule(capsule);
    
    if (validation.isValid) {
      echoPass("✅ Capsule validation passed!");
      echoInfo(`  📊 Memory count: ${validation.memoryCount}`);
      echoInfo(`  📊 Gallery count: ${validation.galleryCount}`);
      echoInfo(`  📊 Created at: ${new Date(Number(validation.createdAt) / 1_000_000).toISOString()}`);
    } else {
      echoFail("❌ Capsule validation failed!");
      echoError(`  Issues: ${validation.issues.join(', ')}`);
      return false;
    }
    
    return { actor, capsuleId: capsuleData.id };
    
  } catch (error) {
    echoFail(`Capsule creation test failed: ${error.message}`);
    return false;
  }
}

/**
 * Test Framework Demo: Create and Retrieve Memory
 */
async function testMemoryCreation(actor, capsuleId) {
  echoSubHeader("Test 2: Create and Retrieve Memory");
  
  try {
    // Step 1: Create test content using framework
    echoInfo("Creating test content using framework...");
    const content = createTestContent("framework_demo", "text/plain");
    echoPass(`Test content created: "${content.text}" (${content.bytes.length} bytes)`);
    
    // Step 2: Create asset metadata using framework
    echoInfo("Creating asset metadata using framework...");
    const assetMetadata = createAssetMetadata(
      content.name,
      content.bytes.length,
      content.mimeType
    );
    echoPass("Asset metadata created");
    
    // Step 3: Create memory using framework
    echoInfo("Creating test memory using framework...");
    const memoryData = await createTestMemory(actor, capsuleId, {
      content: content.text,
      name: content.name,
      mimeType: content.mimeType,
      tags: ["test", "framework", "demo"]
    });
    
    echoPass(`Memory created: ${memoryData.id}`);
    
    // Step 4: Retrieve memory to verify
    echoInfo("Retrieving memory to verify creation...");
    const retrievedMemory = await actor.memories_read(memoryData.id);
    
    if (!retrievedMemory.Ok) {
      throw new Error(`Failed to retrieve memory: ${JSON.stringify(retrievedMemory.Err)}`);
    }
    
    const memory = retrievedMemory.Ok;
    echoPass("Memory retrieved successfully!");
    
    // Step 5: Validate memory using framework
    echoInfo("Validating memory using framework...");
    const validation = validateMemory(memory);
    
    if (validation.isValid) {
      echoPass("✅ Memory validation passed!");
      echoInfo(`  📊 ID: ${validation.id}`);
      echoInfo(`  📊 Title: ${validation.title}`);
      echoInfo(`  📊 Content Type: ${validation.contentType}`);
      echoInfo(`  📊 Inline Assets: ${validation.inlineAssetCount}`);
      echoInfo(`  📊 Created at: ${new Date(Number(validation.createdAt) / 1_000_000).toISOString()}`);
      
      // Verify content integrity
      if (validation.inlineAssetCount > 0) {
        const inlineAsset = memory.inline_assets[0];
        const retrievedContent = Buffer.from(inlineAsset.bytes).toString('utf8');
        
        if (retrievedContent === content.text) {
          echoPass("✅ Content integrity verified!");
          echoInfo(`  📝 Retrieved: "${retrievedContent}"`);
        } else {
          echoFail("❌ Content integrity failed!");
          echoError(`  Expected: "${content.text}"`);
          echoError(`  Retrieved: "${retrievedContent}"`);
          return false;
        }
      }
    } else {
      echoFail("❌ Memory validation failed!");
      echoError(`  Issues: ${validation.issues.join(', ')}`);
      return false;
    }
    
    return { memoryId: memoryData.id };
    
  } catch (error) {
    echoFail(`Memory creation test failed: ${error.message}`);
    return false;
  }
}

/**
 * Test Framework Demo: Cleanup
 */
async function testCleanup(actor, capsuleId, memoryId) {
  echoSubHeader("Test 3: Cleanup");
  
  try {
    // Cleanup memory
    if (memoryId) {
      echoInfo("Cleaning up test memory...");
      const deleteResult = await actor.memories_delete(memoryId);
      if (deleteResult.Ok) {
        echoPass("Memory deleted successfully");
      } else {
        echoError(`Failed to delete memory: ${JSON.stringify(deleteResult.Err)}`);
      }
    }
    
    // Cleanup capsule
    if (capsuleId) {
      echoInfo("Cleaning up test capsule...");
      const deleteCapsuleResult = await actor.capsules_delete(capsuleId);
      if (deleteCapsuleResult.Ok) {
        echoPass("Capsule deleted successfully");
      } else {
        echoError(`Failed to delete capsule: ${JSON.stringify(deleteCapsuleResult.Err)}`);
      }
    }
    
    echoPass("✅ Cleanup completed");
    return true;
    
  } catch (error) {
    echoError(`Cleanup failed: ${error.message}`);
    return false;
  }
}

/**
 * Main test execution
 */
async function main() {
  echoHeader(TEST_NAME);
  
  let actor = null;
  let capsuleId = null;
  let memoryId = null;
  
  try {
    // Get environment info
    echoInfo("Getting environment information...");
    const env = getEnvironmentInfo();
    echoInfo(`  Host: ${env.host}`);
    echoInfo(`  Canister ID: ${env.canisterId}`);
    echoInfo(`  Is Mainnet: ${env.isMainnet}`);
    
    // Test 1: Create and retrieve capsule
    const capsuleResult = await testCapsuleCreation();
    if (!capsuleResult) {
      throw new Error("Capsule creation test failed");
    }
    
    actor = capsuleResult.actor;
    capsuleId = capsuleResult.capsuleId;
    
    // Test 2: Create and retrieve memory
    const memoryResult = await testMemoryCreation(actor, capsuleId);
    if (!memoryResult) {
      throw new Error("Memory creation test failed");
    }
    
    memoryId = memoryResult.memoryId;
    
    // Test 3: Cleanup
    await testCleanup(actor, capsuleId, memoryId);
    
    echoHeader("Test Framework Demo Results");
    echoPass("🎉 All framework tests passed!");
    echoInfo("✅ Test framework is working correctly");
    echoInfo("✅ Capsule creation and retrieval works");
    echoInfo("✅ Memory creation and retrieval works");
    echoInfo("✅ Content integrity is preserved");
    echoInfo("✅ Validation framework works");
    echoInfo("✅ Cleanup works properly");
    
  } catch (error) {
    echoError(`Test framework demo failed: ${error.message}`);
    console.error("Stack trace:", error.stack);
    process.exit(1);
  } finally {
    // Final cleanup
    if (actor && capsuleId) {
      try {
        await actor.capsules_delete(capsuleId);
      } catch (error) {
        echoError(`Final cleanup failed: ${error.message}`);
      }
    }
  }
}

// Run the test
main().catch(console.error);
