#!/usr/bin/env node

import { Actor, HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity } from "../upload/ic-identity.js";
import { idlFactory } from "../../declarations/backend/backend.did.js";

// Configuration
const CANISTER_ID = "uxrrr-q7777-77774-qaaaq-cai";
const HOST = "http://127.0.0.1:4943";

async function main() {
  console.log("🧪 Testing dev_clear_all_memories_in_capsule function...");

  // Create agent and actor
  const identity = loadDfxIdentity();
  const agent = new HttpAgent({
    host: HOST,
    identity,
    verifyQuerySignatures: false,
  });
  await agent.fetchRootKey();

  const actor = Actor.createActor(idlFactory, {
    agent,
    canisterId: CANISTER_ID,
  });

  console.log("✅ Test environment initialized");

  try {
    // Test 1: Try to clear a non-existent capsule (should return NotFound)
    console.log("\n📋 Test 1: Non-existent capsule");
    const result1 = await actor.dev_clear_all_memories_in_capsule("non-existent-capsule", true);
    if ("Err" in result1 && "NotFound" in result1.Err) {
      console.log("✅ Got expected NotFound error for non-existent capsule");
    } else {
      console.log("❌ Expected NotFound error but got:", result1);
    }

    // Test 2: Try to clear an empty capsule (should return 0 deleted count)
    console.log("\n📋 Test 2: Empty capsule");
    let capsuleId;
    try {
      // Get or create a capsule
      const capsulesResult = await actor.capsules_read_basic([]);
      
      if ("Ok" in capsulesResult && capsulesResult.Ok) {
        capsuleId = capsulesResult.Ok.capsule_id;
        console.log("Using existing capsule ID:", capsuleId);
      } else {
        // Create a new capsule
        const createResult = await actor.capsules_create([]);
        if ("Ok" in createResult) {
          capsuleId = createResult.Ok.id;
          console.log("Created new capsule ID:", capsuleId);
        } else {
          throw new Error("Failed to create capsule");
        }
      }
      
      const result2 = await actor.dev_clear_all_memories_in_capsule(capsuleId, false);
      console.log("✅ Empty capsule result:", result2);
      if ("Ok" in result2) {
        console.log("   Deleted count:", result2.Ok.deleted_count);
        console.log("   Message:", result2.Ok.message);
        console.log("   Failed count:", result2.Ok.failed_count);
      } else {
        console.log("   Error:", result2.Err);
      }

    } catch (error) {
      console.log("❌ Error testing empty capsule:", error.message);
    }

    // Test 3: Check if there are existing memories and clear them
    console.log("\n📋 Test 3: Clear capsule with existing memories");
    try {
      // First, check how many memories exist in the capsule
      const listResult = await actor.memories_list_by_capsule(capsuleId, [], []);
      let existingCount = 0;
      
      if ("Ok" in listResult) {
        existingCount = listResult.Ok.items.length;
        console.log(`   Found ${existingCount} existing memories in capsule`);
        
        if (existingCount > 0) {
          console.log("   Memory IDs:", listResult.Ok.items.map(item => item.id));
        }
      } else {
        console.log("   Failed to list memories:", listResult.Err);
      }

      // Now clear the capsule
      const result3 = await actor.dev_clear_all_memories_in_capsule(capsuleId, true);
      console.log("✅ Clear with memories result:", result3);
      if ("Ok" in result3) {
        console.log("   Deleted count:", result3.Ok.deleted_count);
        console.log("   Message:", result3.Ok.message);
        console.log("   Failed count:", result3.Ok.failed_count);
        
        // Verify the count matches
        if (result3.Ok.deleted_count === existingCount) {
          console.log("   ✅ Deleted count matches existing count");
        } else {
          console.log(`   ⚠️  Deleted count (${result3.Ok.deleted_count}) doesn't match existing count (${existingCount})`);
        }
      } else {
        console.log("   Error:", result3.Err);
      }

    } catch (error) {
      console.log("❌ Error testing clear with memories:", error.message);
    }

    console.log("\n🎉 All tests completed!");

  } catch (error) {
    console.error("❌ Test failed:", error);
  }
}

// Run the test
main().catch(console.error);
