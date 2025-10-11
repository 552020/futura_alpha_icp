#!/usr/bin/env node

/**
 * Simple Memory Demo
 *
 * This is a basic demo that:
 * 1. Creates a capsule
 * 2. Creates a memory in that capsule
 * 3. Reads the memory back
 * 4. Shows the memory data
 *
 * No complex framework - just the basics!
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { loadDfxIdentity } from "../upload/ic-identity.js";
import fetch from "node-fetch";
import crypto from "crypto";

// Import the backend interface
import { idlFactory } from "../../declarations/backend/backend.did.js";

// Test configuration
const HOST = process.env.IC_HOST || "http://127.0.0.1:4943";
const IS_MAINNET = process.env.IC_HOST === "https://ic0.app" || process.env.IC_HOST === "https://icp0.io";

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
  console.log("=".repeat(50));
}

/**
 * Create test agent
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
 * Create asset metadata
 */
function createAssetMetadata(name, size, mimeType) {
  const contentHash = crypto.createHash("sha256").update(name).digest();

  return {
    Note: {
      base: {
        name: name,
        description: [`Test memory: ${name}`], // Wrap in array for opt text
        mime_type: mimeType,
        bytes: size,
        sha256: [Array.from(contentHash)],
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000), // Convert to nanoseconds
        asset_type: { Original: null },
        storage_key: [],
        tags: [],
        processing_error: [],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      language: [],
      word_count: [],
      format: [],
    },
  };
}

/**
 * Simple Memory Demo
 */
async function simpleMemoryDemo() {
  echoHeader("Simple Memory Demo");

  let actor = null;
  let capsuleId = null;
  let memoryId = null;

  try {
    // Step 1: Create test actor
    echoInfo("Creating test actor...");
    actor = await createTestActor();
    echoPass("Test actor created");

    // Step 2: Create a capsule
    echoInfo("Creating capsule...");
    const capsuleResult = await actor.capsules_create([]);

    if (!capsuleResult.Ok) {
      throw new Error(`Failed to create capsule: ${JSON.stringify(capsuleResult.Err)}`);
    }

    capsuleId = capsuleResult.Ok.id;
    echoPass(`Capsule created: ${capsuleId}`);

    // Step 3: Create test content
    echoInfo("Creating test content...");
    const content = "Hello, this is a simple test memory!";
    const contentBytes = Array.from(Buffer.from(content, "utf8"));
    echoPass(`Test content created: "${content}" (${contentBytes.length} bytes)`);

    // Step 4: Create asset metadata
    echoInfo("Creating asset metadata...");
    const assetMetadata = createAssetMetadata("simple_test_memory", contentBytes.length, "text/plain");
    echoPass("Asset metadata created");

    // Step 5: Create memory
    echoInfo("Creating memory...");
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

    if (!memoryResult.Ok) {
      throw new Error(`Failed to create memory: ${JSON.stringify(memoryResult.Err)}`);
    }

    memoryId = memoryResult.Ok;
    echoPass(`Memory created: ${memoryId}`);

    // Print the full create result
    echoInfo("📋 CREATE FUNCTION RESULT:");
    echoInfo(JSON.stringify(memoryResult, (key, value) => (typeof value === "bigint" ? value.toString() : value), 2));

    // Step 6: Read the memory back
    echoInfo("Reading memory back...");
    const readResult = await actor.memories_read(memoryId);

    if (!readResult.Ok) {
      throw new Error(`Failed to read memory: ${JSON.stringify(readResult.Err)}`);
    }

    const memory = readResult.Ok;
    echoPass("Memory read successfully!");

    // Print the full read result
    echoInfo("📋 READ FUNCTION RESULT:");
    echoInfo(JSON.stringify(readResult, (key, value) => (typeof value === "bigint" ? value.toString() : value), 2));

    // Step 7: Show memory data
    echoInfo("Memory data:");
    echoInfo(`  🆔 ID: ${memory.id}`);
    echoInfo(`  📝 Title: ${memory.metadata.title[0] || "No title"}`);
    echoInfo(`  📄 Content Type: ${memory.metadata.content_type}`);
    echoInfo(`  📅 Created at: ${new Date(Number(memory.metadata.created_at) / 1_000_000).toISOString()}`);
    echoInfo(`  🏷️  Tags: ${memory.metadata.tags.join(", ") || "No tags"}`);
    echoInfo(`  👤 Created by: ${memory.metadata.created_by || "Unknown"}`);

    // Step 8: Show asset data
    echoInfo("Asset data:");
    echoInfo(`  📦 Inline assets: ${memory.inline_assets.length}`);
    echoInfo(`  🗂️  Blob internal assets: ${memory.blob_internal_assets.length}`);
    echoInfo(`  🌐 Blob external assets: ${memory.blob_external_assets.length}`);

    // Step 9: Verify content integrity
    if (memory.inline_assets.length > 0) {
      const inlineAsset = memory.inline_assets[0];
      const retrievedContent = Buffer.from(inlineAsset.bytes).toString("utf8");

      echoInfo("Content verification:");
      echoInfo(`  📝 Original: "${content}"`);
      echoInfo(`  📝 Retrieved: "${retrievedContent}"`);

      if (retrievedContent === content) {
        echoPass("✅ Content integrity verified!");
      } else {
        echoFail("❌ Content integrity failed!");
        echoError(`  Expected: "${content}"`);
        echoError(`  Retrieved: "${retrievedContent}"`);
        return false;
      }
    }

    echoPass("🎉 Simple memory demo completed successfully!");

    return { actor, capsuleId, memoryId };
  } catch (error) {
    echoError(`Demo failed: ${error.message}`);
    throw error;
  }
}

/**
 * Cleanup
 */
async function cleanup(actor, capsuleId, memoryId) {
  if (actor) {
    try {
      echoInfo("Cleaning up...");

      // Delete memory first
      if (memoryId) {
        const deleteResult = await actor.memories_delete(memoryId, false);
        if (deleteResult && deleteResult.Ok !== undefined) {
          echoPass("Memory deleted successfully");
        } else {
          echoError(`Failed to delete memory: ${JSON.stringify(deleteResult)}`);
        }
      }

      // Delete capsule
      if (capsuleId) {
        const deleteCapsuleResult = await actor.capsules_delete(capsuleId);
        if (deleteCapsuleResult && deleteCapsuleResult.Ok !== undefined) {
          echoPass("Capsule deleted successfully");
        } else {
          echoError(`Failed to delete capsule: ${JSON.stringify(deleteCapsuleResult)}`);
        }
      }
    } catch (error) {
      echoError(`Cleanup failed: ${error.message}`);
    }
  }
}

/**
 * Main execution
 */
async function main() {
  let actor = null;
  let capsuleId = null;
  let memoryId = null;

  try {
    const result = await simpleMemoryDemo();
    actor = result.actor;
    capsuleId = result.capsuleId;
    memoryId = result.memoryId;
  } catch (error) {
    echoError(`Simple demo failed: ${error.message}`);
    console.error("Stack trace:", error.stack);
    process.exit(1);
  } finally {
    await cleanup(actor, capsuleId, memoryId);
  }
}

// Run the demo
main().catch(console.error);
