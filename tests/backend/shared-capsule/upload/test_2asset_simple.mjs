import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { loadDfxIdentity } from "./ic-identity.js";
import crypto from "crypto";

// Configuration
const BACKEND_CANISTER_ID = process.argv[2];

// Helper functions
function echoInfo(msg) {
  console.log(`ℹ️  ${msg}`);
}

function echoPass(msg) {
  console.log(`✅ ${msg}`);
}

function echoFail(msg) {
  console.log(`❌ ${msg}`);
}

async function createTestCapsule(backend) {
  echoInfo("Creating test capsule...");
  const capsuleResult = await backend.capsules_create([]);
  if ("Err" in capsuleResult) {
    throw new Error(`Capsule creation failed: ${JSON.stringify(capsuleResult.Err)}`);
  }
  const capsuleId = capsuleResult.Ok.id;
  echoPass(`Test capsule created: ${capsuleId}`);
  return capsuleId;
}

async function uploadSmallFile(backend, capsuleId, content, name) {
  const fileBuffer = Buffer.from(content);
  const fileSize = fileBuffer.length;

  echoInfo(`📤 Uploading: ${name} (${fileSize} bytes)`);

  // Begin upload
  const beginResult = await backend.uploads_begin(capsuleId, 1, `test-${name}`);
  if ("Err" in beginResult) {
    throw new Error(`Upload begin failed: ${JSON.stringify(beginResult.Err)}`);
  }
  const sessionId = beginResult.Ok;

  // Upload single chunk (small file)
  const putResult = await backend.uploads_put_chunk(sessionId, 0, Array.from(fileBuffer));
  if ("Err" in putResult) {
    throw new Error(`Chunk upload failed: ${JSON.stringify(putResult.Err)}`);
  }

  // Finish upload
  const expectedHash = crypto.createHash("sha256").update(fileBuffer).digest();
  const finishResult = await backend.uploads_finish(sessionId, expectedHash, BigInt(fileSize));

  if ("Err" in finishResult) {
    throw new Error(`Upload finish failed: ${JSON.stringify(finishResult.Err)}`);
  }

  const result = finishResult.Ok;
  echoPass(`✅ Upload completed: ${name} (${fileSize} bytes)`);

  return {
    blobId: result.blob_id,
    size: result.size,
    hash: result.hash,
  };
}

async function test2AssetSimple() {
  echoInfo("🚀 Starting Simple 2-Asset Test");

  // Load identity and create agent
  const identity = await loadDfxIdentity();
  const agent = new HttpAgent({
    host: "http://127.0.0.1:4943",
    identity,
  });
  await agent.fetchRootKey();

  // Create backend actor
  const backend = Actor.createActor(
    (await import("./declarations/backend/backend.did.js")).idlFactory,
    {
      agent,
      canisterId: Principal.fromText(BACKEND_CANISTER_ID),
    }
  );

  try {
    // Create test capsule
    const capsuleId = await createTestCapsule(backend);

    // Upload two small files
    echoInfo("📤 Uploading first asset...");
    const asset1 = await uploadSmallFile(backend, capsuleId, "Hello, this is asset 1!", "asset1");

    echoInfo("📤 Uploading second asset...");
    const asset2 = await uploadSmallFile(backend, capsuleId, "Hello, this is asset 2!", "asset2");

    // Create memory with both assets
    echoInfo("🧠 Creating memory with 2 assets...");
    const memoryMetadata = {
      memory_type: { Note: null },
      title: ["Simple 2-Asset Test"],
      description: ["Test memory with 2 small assets"],
      content_type: "text/plain",
      created_at: BigInt(Date.now() * 1000000),
      updated_at: BigInt(Date.now() * 1000000),
      uploaded_at: BigInt(Date.now() * 1000000),
      date_of_memory: [],
      file_created_at: [],
      parent_folder_id: [],
      tags: ["test", "2asset", "simple"],
      deleted_at: [],
      people_in_memory: [],
      location: [],
      memory_notes: [],
      created_by: [],
      database_storage_edges: [],
    };

    const assetMetadata = {
      Note: {
        base: {
          url: [],
          height: [],
          updated_at: BigInt(Date.now() * 1000000),
          asset_type: { Original: null },
          sha256: [],
          name: "test-asset",
          storage_key: [],
          tags: ["test", "2asset"],
          processing_error: [],
          mime_type: "text/plain",
          description: [],
          created_at: BigInt(Date.now() * 1000000),
          deleted_at: [],
          bytes: BigInt(0),
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

    const allAssets = [
      { blob_id: asset1.blobId, metadata: assetMetadata },
      { blob_id: asset2.blobId, metadata: assetMetadata },
    ];

    const memoryResult = await backend.memories_create_with_internal_blobs(
      capsuleId,
      memoryMetadata,
      allAssets,
      `memory-2asset-simple-${Date.now()}`
    );

    if ("Err" in memoryResult) {
      throw new Error(`Memory creation failed: ${JSON.stringify(memoryResult.Err)}`);
    }

    const memoryId = memoryResult.Ok;
    echoPass(`Memory created: ${memoryId}`);

    // Verify memory exists and has 2 assets
    echoInfo("🔍 Verifying memory has 2 assets...");
    const readResult = await backend.memories_read(memoryId);
    if ("Err" in readResult) {
      throw new Error(`Memory read failed: ${JSON.stringify(readResult.Err)}`);
    }

    const memory = readResult.Ok;
    if (memory.blob_internal_assets.length !== 2) {
      throw new Error(`Expected 2 assets, got ${memory.blob_internal_assets.length}`);
    }
    echoPass(`Memory verified: ${memory.blob_internal_assets.length} assets`);

    // Test full deletion (memory + assets)
    echoInfo("🗑️ Testing full deletion...");
    const deleteResult = await backend.memories_delete(memoryId, true);
    if ("Err" in deleteResult) {
      throw new Error(`Memory deletion failed: ${JSON.stringify(deleteResult.Err)}`);
    }
    echoPass("Memory deleted successfully");

    // Verify memory is gone
    echoInfo("🔍 Verifying memory is deleted...");
    const readAfterDelete = await backend.memories_read(memoryId);
    if ("Ok" in readAfterDelete) {
      throw new Error("Memory still exists after deletion");
    }
    echoPass("Memory successfully deleted");

    // Verify assets are gone (should get NotFound errors)
    echoInfo("🔍 Verifying assets are deleted...");
    const blob1Read = await backend.blob_read(asset1.blobId);
    if ("Ok" in blob1Read) {
      throw new Error("Asset 1 still exists after deletion");
    }

    const blob2Read = await backend.blob_read(asset2.blobId);
    if ("Ok" in blob2Read) {
      throw new Error("Asset 2 still exists after deletion");
    }
    echoPass("Both assets successfully deleted");

    echoPass("🎉 Simple 2-Asset Test PASSED!");
  } catch (error) {
    echoFail(`Test failed: ${error.message}`);
    process.exit(1);
  }
}

// Run the test
test2AssetSimple();
