#!/usr/bin/env node

/**
 * Simple Capsule Demo
 *
 * This is a basic demo that:
 * 1. Creates a capsule
 * 2. Reads the capsule back
 * 3. Shows the capsule data
 *
 * No complex framework - just the basics!
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { loadDfxIdentity } from "../upload/ic-identity.js";
import fetch from "node-fetch";

// Import the backend interface
import { idlFactory } from "../../../../.dfx/local/canisters/backend/service.did.js";

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
 * Simple Capsule Demo
 */
async function simpleCapsuleDemo() {
  echoHeader("Simple Capsule Demo");

  let actor = null;
  let capsuleId = null;

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
    
    // Print the full create result
    echoInfo("📋 CAPSULE CREATE FUNCTION RESULT:");
    echoInfo(JSON.stringify(capsuleResult, (key, value) => 
      typeof value === 'bigint' ? value.toString() : value, 2));

    // Step 3: Read the capsule back
    echoInfo("Reading capsule back...");
    const readResult = await actor.capsules_read_basic([capsuleId]); // Wrap in array for opt text

    if (!readResult.Ok) {
      throw new Error(`Failed to read capsule: ${JSON.stringify(readResult.Err)}`);
    }

    const capsule = readResult.Ok;
    echoPass("Capsule read successfully!");
    
    // Print the full read result
    echoInfo("📋 CAPSULE READ FUNCTION RESULT:");
    echoInfo(JSON.stringify(readResult, (key, value) => 
      typeof value === 'bigint' ? value.toString() : value, 2));
    
    // Step 4: Show capsule data
    echoInfo("Capsule data:");
    echoInfo(`  🆔 ID: ${capsule.capsule_id}`);
    echoInfo(`  📊 Memory count: ${capsule.memory_count}`);
    echoInfo(`  📊 Gallery count: ${capsule.gallery_count}`);
    echoInfo(`  📅 Created at: ${new Date(Number(capsule.created_at) / 1_000_000).toISOString()}`);
    echoInfo(`  👤 Is owner: ${capsule.is_owner}`);
    echoInfo(`  👤 Is controller: ${capsule.is_controller}`);
    echoInfo(`  👤 Is self capsule: ${capsule.is_self_capsule}`);
    echoInfo(`  🔗 Bound to Neon: ${capsule.bound_to_neon}`);

    // Step 5: Show permissions
    if (capsule.permissions) {
      echoInfo("Permissions:");
      echoInfo(`  📖 Read: ${capsule.permissions.read ? "Yes" : "No"}`);
      echoInfo(`  ✏️  Write: ${capsule.permissions.write ? "Yes" : "No"}`);
      echoInfo(`  🗑️  Delete: ${capsule.permissions.delete ? "Yes" : "No"}`);
    }

    echoPass("🎉 Simple capsule demo completed successfully!");

    return { actor, capsuleId, capsule };
  } catch (error) {
    echoError(`Demo failed: ${error.message}`);
    throw error;
  }
}

/**
 * Cleanup
 */
async function cleanup(actor, capsuleId) {
  if (actor && capsuleId) {
    try {
      echoInfo("Cleaning up...");
      const deleteResult = await actor.capsules_delete(capsuleId);
      if (deleteResult.Ok) {
        echoPass("Capsule deleted successfully");
      } else {
        echoError(`Failed to delete capsule: ${JSON.stringify(deleteResult.Err)}`);
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

  try {
    const result = await simpleCapsuleDemo();
    actor = result.actor;
    capsuleId = result.capsule.capsule_id; // Use the actual capsule ID from the response
  } catch (error) {
    echoError(`Simple demo failed: ${error.message}`);
    console.error("Stack trace:", error.stack);
    process.exit(1);
  } finally {
    await cleanup(actor, capsuleId);
  }
}

// Run the demo
main().catch(console.error);
