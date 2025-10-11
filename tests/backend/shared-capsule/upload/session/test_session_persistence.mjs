/**
 * Minimal E2E Test: Session Persistence
 *
 * Tests if sessions persist across multiple put_chunk calls
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity } from "./ic-identity.js";
import { idlFactory } from "./declarations/backend/backend.did.js";

const CHUNK_SIZE = 1_800_000; // 1.8MB

async function main() {
  console.log("🧪 Session Persistence Test\n");

  // 1. Connect to local network
  const identity = await loadDfxIdentity();
  const agent = new HttpAgent({
    host: "http://127.0.0.1:4943",
    identity,
  });
  await agent.fetchRootKey();

  const canisterId = process.env.BACKEND_CANISTER_ID || "uxrrr-q7777-77774-qaaaq-cai";
  const backend = Actor.createActor(idlFactory, {
    agent,
    canisterId,
  });

  console.log(`✅ Connected to backend: ${canisterId}\n`);

  // 2. Create a test capsule
  console.log("📦 Creating test capsule...");
  const createResult = await backend.capsules_create([]);

  let capsuleId;
  if (createResult.Ok) {
    capsuleId = createResult.Ok.id;
    console.log(`✅ Capsule created: ${capsuleId}\n`);
  } else {
    console.error("❌ Failed to create capsule:", createResult);
    process.exit(1);
  }

  // 3. Begin upload
  console.log("🚀 Step 1: uploads_begin");
  const now = BigInt(Date.now() * 1_000_000); // Nanoseconds
  const assetMetadata = {
    Image: {
      dpi: [],
      color_space: [],
      base: {
        url: [],
        height: [],
        updated_at: now,
        asset_type: { Original: null },
        sha256: [],
        name: "test-image.jpg",
        storage_key: [],
        tags: ["test"],
        processing_error: [],
        mime_type: "image/jpeg",
        description: [],
        created_at: now,
        deleted_at: [],
        bytes: BigInt(3_600_000), // 2 chunks worth
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      exif_data: [],
      compression_ratio: [],
      orientation: [],
    },
  };

  const beginResult = await backend.uploads_begin(
    capsuleId,
    assetMetadata,
    2, // 2 chunks
    "test-idem-123"
  );

  let sessionId;
  if (typeof beginResult === "number" || typeof beginResult === "bigint") {
    sessionId = Number(beginResult);
    console.log(`✅ Session created: ${sessionId}\n`);
  } else if (beginResult.Ok !== undefined) {
    sessionId = Number(beginResult.Ok);
    console.log(`✅ Session created: ${sessionId}\n`);
  } else {
    console.error("❌ uploads_begin failed:", beginResult);
    process.exit(1);
  }

  // 4. Put chunk 0
  console.log("📤 Step 2: uploads_put_chunk (chunk 0)");
  const chunk0 = new Uint8Array(CHUNK_SIZE).fill(0xaa);

  try {
    const put0Result = await backend.uploads_put_chunk(sessionId, 0, Array.from(chunk0));

    if (put0Result && put0Result.Err) {
      console.error("❌ Put chunk 0 failed:", put0Result.Err);
      process.exit(1);
    }
    console.log("✅ Chunk 0 uploaded successfully\n");
  } catch (error) {
    console.error("❌ Put chunk 0 error:", error);
    process.exit(1);
  }

  // 5. Put chunk 1 (this should also work if session persists)
  console.log("📤 Step 3: uploads_put_chunk (chunk 1)");
  const chunk1 = new Uint8Array(CHUNK_SIZE).fill(0xbb);

  try {
    const put1Result = await backend.uploads_put_chunk(sessionId, 1, Array.from(chunk1));

    if (put1Result && put1Result.Err) {
      console.error("❌ Put chunk 1 failed:", put1Result.Err);
      console.error("\n🔍 Diagnosis: Session was lost between chunk 0 and chunk 1!");
      console.error("   This means thread_local storage is not working correctly.");
      process.exit(1);
    }
    console.log("✅ Chunk 1 uploaded successfully\n");
  } catch (error) {
    console.error("❌ Put chunk 1 error:", error);
    console.error("\n🔍 Diagnosis: Session was lost between chunk 0 and chunk 1!");
    process.exit(1);
  }

  // 6. Finish upload
  console.log("🏁 Step 4: uploads_finish");
  // Recreate the same test data we uploaded
  const testData = new Uint8Array(3_600_000);
  testData.set(chunk0, 0);
  testData.set(chunk1.slice(0, 3_600_000 - CHUNK_SIZE), CHUNK_SIZE);

  const hash = await crypto.subtle.digest("SHA-256", testData);
  const hashArray = Array.from(new Uint8Array(hash));

  try {
    const finishResult = await backend.uploads_finish(sessionId, hashArray, BigInt(3_600_000));

    if (finishResult.Ok) {
      console.log("✅ Upload finished successfully:", finishResult.Ok);
      console.log("\n🎉 SUCCESS! Session persistence is working correctly!");
    } else {
      console.error("❌ uploads_finish failed:", finishResult.Err);
      process.exit(1);
    }
  } catch (error) {
    console.error("❌ uploads_finish error:", error);
    process.exit(1);
  }
}

main().catch((error) => {
  console.error("💥 Test failed with error:", error);
  process.exit(1);
});
