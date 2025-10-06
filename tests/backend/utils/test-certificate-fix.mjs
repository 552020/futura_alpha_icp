#!/usr/bin/env node

/**
 * Certificate Fix Test
 * 
 * Tests the exact configuration that works vs what fails
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity } from "../shared-capsule/upload/ic-identity.js";
import fetch from "node-fetch";

// Use the SAME interface file as the working test
import { idlFactory } from "../../../src/nextjs/src/ic/declarations/backend/backend.did.js";

const CANISTER_ID = process.env.BACKEND_CANISTER_ID || "uxrrr-q7777-77774-qaaaq-cai";
const HOST = "http://127.0.0.1:4943";

async function testWorkingConfiguration() {
  console.log("🧪 Testing WORKING configuration (like test_capsules_create.mjs)");
  
  try {
    // Use EXACT same configuration as working test
    const identity = loadDfxIdentity();
    console.log(`Using identity: ${identity.getPrincipal().toString()}`);
    
    const agent = new HttpAgent({
      host: HOST,
      identity,
      fetch, // Same as working test
    });
    
    // Fetch root key for local replica (same as working test)
    await agent.fetchRootKey();
    console.log("✅ Fetched root key for local replica");
    
    const actor = Actor.createActor(idlFactory, {
      agent,
      canisterId: CANISTER_ID,
    });
    
    console.log("✅ Actor created successfully");
    
    // Test the call that was failing
    console.log("Testing capsules_create...");
    const result = await actor.capsules_create([]);
    
    if (result.Ok) {
      console.log("✅ capsules_create SUCCESS!");
      console.log(`Capsule ID: ${result.Ok.id}`);
      
      // Clean up
      await actor.capsules_delete(result.Ok.id);
      console.log("✅ Cleanup completed");
      
      return true;
    } else {
      console.log("❌ capsules_create failed:", JSON.stringify(result));
      return false;
    }
    
  } catch (error) {
    console.log("❌ Test failed:", error.message);
    return false;
  }
}

async function testFailingConfiguration() {
  console.log("\n🧪 Testing FAILING configuration (like our bulk tests)");
  
  try {
    const identity = loadDfxIdentity();
    
    const agent = new HttpAgent({
      host: HOST,
      identity,
      verifyQuerySignatures: false, // This might be the issue
      fetch: null, // Different from working test
    });
    
    // Don't fetch root key (different from working test)
    console.log("❌ Not fetching root key (like failing tests)");
    
    const actor = Actor.createActor(idlFactory, {
      agent,
      canisterId: CANISTER_ID,
    });
    
    console.log("✅ Actor created");
    
    // Test the call
    console.log("Testing capsules_create...");
    const result = await actor.capsules_create([]);
    
    if (result.Ok) {
      console.log("✅ capsules_create SUCCESS!");
      return true;
    } else {
      console.log("❌ capsules_create failed:", JSON.stringify(result));
      return false;
    }
    
  } catch (error) {
    console.log("❌ Test failed with certificate error:", error.message);
    return false;
  }
}

async function main() {
  console.log("🔍 Certificate Verification Error Analysis");
  console.log("=" * 50);
  
  const workingResult = await testWorkingConfiguration();
  const failingResult = await testFailingConfiguration();
  
  console.log("\n📊 Results:");
  console.log(`Working configuration: ${workingResult ? "✅ SUCCESS" : "❌ FAILED"}`);
  console.log(`Failing configuration: ${failingResult ? "✅ SUCCESS" : "❌ FAILED"}`);
  
  if (workingResult && !failingResult) {
    console.log("\n🎯 ROOT CAUSE IDENTIFIED:");
    console.log("The issue is in the agent configuration!");
    console.log("Working tests use:");
    console.log("  - fetch: node-fetch import");
    console.log("  - await agent.fetchRootKey()");
    console.log("  - No verifyQuerySignatures: false");
    console.log("");
    console.log("Failing tests use:");
    console.log("  - fetch: null");
    console.log("  - No fetchRootKey() call");
    console.log("  - verifyQuerySignatures: false");
  }
}

main().catch(console.error);

