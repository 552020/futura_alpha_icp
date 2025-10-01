#!/usr/bin/env node

import { readFile } from "fs/promises";
import { Actor, HttpAgent } from "@dfinity/agent";
import { idlFactory } from "../../../../src/nextjs/src/ic/declarations/backend/backend.did.js";
import crypto from "crypto";

const CANISTER_ID = "uxrrr-q7777-77774-qaaaq-cai";
const LOCAL_REPLICA = "http://127.0.0.1:4943";
const CHUNK_SIZE = 1024 * 1024 * 1.7; // 1.7 MB

async function main() {
  console.log("🔍 Single Upload Debug Test");
  console.log("============================\n");

  // Create agent
  const agent = new HttpAgent({ host: LOCAL_REPLICA });
  await agent.fetchRootKey();

  const actor = Actor.createActor(idlFactory, {
    agent,
    canisterId: CANISTER_ID,
  });

  // Create a small test file (100KB)
  const testData = Buffer.alloc(100 * 1024);
  crypto.randomFillSync(testData);

  const expectedHash = crypto.createHash("sha256").update(testData).digest();
  console.log(`📄 Test file: 100 KB`);
  console.log(`🔐 Expected hash: ${expectedHash.toString("hex").substring(0, 16)}...`);

  try {
    // Begin upload
    console.log("\n1️⃣ Starting upload session...");
    let result;
    try {
      result = await actor.uploads_begin(
        "test-capsule",
        { Base: { bytes: BigInt(testData.length), mime_type: "image/jpeg" } },
        1, // 1 chunk
        `test-${Date.now()}`
      );
    } catch (beginError) {
      console.error("❌ Begin call threw error:", beginError.message);
      console.log("\n📋 Backend logs:");
      console.log("Run: dfx canister logs backend");
      return;
    }

    if ("Err" in result) {
      console.error("❌ Begin failed:", JSON.stringify(result.Err, null, 2));
      return;
    }

    if (!result.Ok && result.Ok !== 0) {
      console.error("❌ Begin returned unexpected result:", JSON.stringify(result, null, 2));
      return;
    }

    const sessionId = result.Ok;
    console.log(`✅ Session ID: ${sessionId}`);

    // Upload chunk
    console.log("\n2️⃣ Uploading chunk...");
    await actor.uploads_put_chunk(sessionId, 0, Array.from(testData));
    console.log("✅ Chunk uploaded");

    // Finish upload
    console.log("\n3️⃣ Finishing upload...");
    const finishResult = await actor.uploads_finish(sessionId, Array.from(expectedHash), BigInt(testData.length));

    if ("Err" in finishResult) {
      console.error("❌ Finish failed:", finishResult.Err);
      console.log("\n📋 Now check logs with:");
      console.log('dfx canister logs backend | grep -E "(BLOB_|FINISH|COMMIT|UPLOAD_HASH)"');
      process.exit(1);
    }

    console.log("✅ Upload succeeded!");
    console.log(`   Memory ID: ${finishResult.Ok.memory_id}`);
    console.log(`   Blob ID: ${finishResult.Ok.blob_id}`);

    console.log("\n✅ TEST PASSED!");
  } catch (error) {
    console.error("❌ Error:", error.message);
    console.log("\n📋 Now check logs with:");
    console.log('dfx canister logs backend | grep -E "(BLOB_|FINISH|COMMIT|UPLOAD_HASH)"');
    process.exit(1);
  }
}

main();
