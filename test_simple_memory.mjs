#!/usr/bin/env node

import { Actor, HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity } from "./tests/backend/shared-capsule/upload/ic-identity.js";
import fetch from "node-fetch";

// Import the backend interface
import { idlFactory } from "./src/nextjs/src/ic/declarations/backend/backend.did.js";

async function testSimpleMemory() {
  console.log("🧪 Testing simple memory creation...");
  
  try {
    // Create agent
    const identity = loadDfxIdentity();
    const agent = new HttpAgent({
      host: "http://127.0.0.1:4943",
      identity,
      fetch,
    });
    await agent.fetchRootKey();
    
    // Create actor
    const actor = Actor.createActor(idlFactory, {
      agent,
      canisterId: "uxrrr-q7777-77774-qaaaq-cai",
    });
    
    // Create capsule
    console.log("Creating capsule...");
    const capsuleResult = await actor.capsules_create([]);
    if (!capsuleResult.Ok) {
      throw new Error(`Failed to create capsule: ${JSON.stringify(capsuleResult.Err)}`);
    }
    const capsuleId = capsuleResult.Ok.id;
    console.log(`✅ Capsule created: ${capsuleId}`);
    
    // Create simple memory
    console.log("Creating memory...");
    const content = "Hello, simple test!";
    const contentBytes = Array.from(Buffer.from(content, "utf8"));
    
    const memoryResult = await actor.memories_create(
      capsuleId,
      [contentBytes],
      [],
      [],
      [],
      [],
      [],
      [],
      {
        Note: {
          base: {
            name: "simple_test",
            bytes: BigInt(contentBytes.length),
            mime_type: "text/plain",
            tags: ["test"],
            created_at: BigInt(Date.now() * 1000000),
            updated_at: BigInt(Date.now() * 1000000),
            asset_type: { Original: null },
            url: [],
            height: [],
            sha256: [],
            storage_key: [],
            processing_error: [],
            description: [],
            deleted_at: [],
            asset_location: [],
            width: [],
            processing_status: [],
            bucket: []
          },
          language: [],
          word_count: [],
          format: []
        }
      },
      `simple_${Date.now()}`
    );
    
    if (!memoryResult.Ok) {
      throw new Error(`Failed to create memory: ${JSON.stringify(memoryResult.Err)}`);
    }
    
    const memoryId = memoryResult.Ok;
    console.log(`✅ Memory created: ${memoryId}`);
    console.log(`✅ Memory ID type: ${typeof memoryId}`);
    
    // Clean up
    await actor.memories_delete(memoryId);
    await actor.capsules_delete(capsuleId);
    console.log("✅ Cleanup completed");
    
    console.log("🎉 Simple memory test passed!");
    
  } catch (error) {
    console.error("❌ Test failed:", error.message);
    console.error("Stack trace:", error.stack);
    process.exit(1);
  }
}

testSimpleMemory().catch(console.error);
