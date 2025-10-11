#!/usr/bin/env node

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { loadDfxIdentity, makeMainnetAgent } from "./ic-identity.js";

const BACKEND_CANISTER_ID = process.argv[2];

if (!BACKEND_CANISTER_ID) {
  console.error("💥 Usage: node test_simple_memory_delete.mjs <BACKEND_CANISTER_ID>");
  process.exit(1);
}

async function testSimpleMemoryDelete() {
  console.log("ℹ️  Starting Simple Memory Delete Test");

  try {
    // Load identity and create agent
    const identity = await loadDfxIdentity();

    // Check if we're connecting to mainnet or local
    const isMainnet = process.env.IC_HOST === "https://ic0.app" || process.env.IC_HOST === "https://icp0.io";
    let agent;

    if (isMainnet) {
      console.log("🌐 Connecting to mainnet");
      agent = makeMainnetAgent(identity);
    } else {
      console.log("🏠 Connecting to local network (127.0.0.1:4943)");
      agent = new HttpAgent({
        host: "http://127.0.0.1:4943",
        identity,
      });
      // Fetch root key for local replica
      await agent.fetchRootKey();
    }

    // Create backend actor
    const backend = Actor.createActor(
      (await import("./declarations/backend/backend.did.js")).idlFactory,
      {
        agent,
        canisterId: Principal.fromText(BACKEND_CANISTER_ID),
      }
    );

    console.log("ℹ️  Testing memories_delete with new signature...");

    // Test 1: Try to delete a non-existent memory with delete_assets=true
    console.log("ℹ️  Test 1: Delete non-existent memory with delete_assets=true");
    const result1 = await backend.memories_delete("non-existent-memory", true);
    console.log(`ℹ️  Result: ${JSON.stringify(result1)}`);

    // Test 2: Try to delete a non-existent memory with delete_assets=false
    console.log("ℹ️  Test 2: Delete non-existent memory with delete_assets=false");
    const result2 = await backend.memories_delete("non-existent-memory", false);
    console.log(`ℹ️  Result: ${JSON.stringify(result2)}`);

    console.log("✅ Simple memory delete test completed!");
    console.log("ℹ️  ✅ New memories_delete signature is working");
    console.log("ℹ️  ✅ delete_assets parameter is accepted");
  } catch (error) {
    console.error("💥 Test failed:", error.message);
    process.exit(1);
  }
}

testSimpleMemoryDelete();
