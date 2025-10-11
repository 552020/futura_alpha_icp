import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { loadDfxIdentity } from "./ic-identity.js";
import fs from "fs";
import crypto from "crypto";

// Configuration
const BACKEND_CANISTER_ID = process.argv[2];
const TEST_IMAGE_PATH = "tests/backend/shared-capsule/upload/assets/input/avocado_big_21mb.jpg";

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

async function uploadFile(backend, filePath, sessionId, startTime) {
  const fileBuffer = fs.readFileSync(filePath);
  const fileSize = fileBuffer.length;
  const chunkSize = 1_800_000; // 1.8MB backend chunk size
  const totalChunks = Math.ceil(fileSize / chunkSize);

  echoInfo(`📤 Uploading: ${filePath.split("/").pop()} (${(fileSize / 1024 / 1024).toFixed(1)} MB)`);

  for (let chunkIndex = 0; chunkIndex < totalChunks; chunkIndex++) {
    const start = chunkIndex * chunkSize;
    const end = Math.min(start + chunkSize, fileSize);
    const chunk = fileBuffer.slice(start, end);

    const putResult = await backend.uploads_put_chunk(sessionId, chunkIndex, Array.from(chunk));
    if ("Err" in putResult) {
      throw new Error(`Chunk upload failed: ${JSON.stringify(putResult.Err)}`);
    }

    const progress = Math.round(((chunkIndex + 1) / totalChunks) * 100);
    if (progress % 25 === 0 || chunkIndex === totalChunks - 1) {
      echoInfo(`  📈 ${progress}% (${chunkIndex + 1}/${totalChunks} chunks)`);
    }
  }

  // Finish upload
  const expectedHash = crypto.createHash("sha256").update(fileBuffer).digest();
  const finishResult = await backend.uploads_finish(sessionId, expectedHash, BigInt(fileSize));

  if ("Err" in finishResult) {
    throw new Error(`Upload finish failed: ${JSON.stringify(finishResult.Err)}`);
  }

  const result = finishResult.Ok;
  echoPass(
    `✅ Upload completed: ${filePath.split("/").pop()} (${(fileSize / 1024 / 1024).toFixed(1)} MB) in ${
      (Date.now() - startTime) / 1000
    }s`
  );

  return {
    blobId: result.blob_id,
    size: result.size,
    hash: result.hash,
  };
}

async function test2AssetQuick() {
  echoInfo("🚀 Starting Quick 2-Asset Test");

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

    // Upload original file
    echoInfo("📤 Uploading original file...");
    const startTime1 = Date.now();
    const beginResult = await backend.uploads_begin(capsuleId, 1, "test-2asset");
    if ("Err" in beginResult) {
      throw new Error(`Upload begin failed: ${JSON.stringify(beginResult.Err)}`);
    }
    const sessionId = beginResult.Ok;

    const originalUpload = await uploadFile(backend, TEST_IMAGE_PATH, sessionId, startTime1);
    echoPass(`Original uploaded: ${originalUpload.blobId}`);

    // Create a simple derivative (just use the same file for speed)
    echoInfo("📤 Uploading derivative...");
    const startTime2 = Date.now();
    const beginResult2 = await backend.uploads_begin(capsuleId, 2, "test-2asset-derivative");
    if ("Err" in beginResult2) {
      throw new Error(`Derivative upload begin failed: ${JSON.stringify(beginResult2.Err)}`);
    }
    const sessionId2 = beginResult2.Ok;

    const derivativeUpload = await uploadFile(backend, TEST_IMAGE_PATH, sessionId2, startTime2);
    echoPass(`Derivative uploaded: ${derivativeUpload.blobId}`);

    // Create memory with both assets
    echoInfo("🧠 Creating memory with 2 assets...");
    const memoryMetadata = {
      memory_type: { Image: null },
      title: ["Quick 2-Asset Test"],
      description: ["Test memory with 2 assets"],
      content_type: "image/jpeg",
      created_at: BigInt(Date.now() * 1000000),
      updated_at: BigInt(Date.now() * 1000000),
      uploaded_at: BigInt(Date.now() * 1000000),
      date_of_memory: [],
      file_created_at: [],
      parent_folder_id: [],
      tags: ["test", "2asset", "quick"],
      deleted_at: [],
      people_in_memory: [],
      location: [],
      memory_notes: [],
      created_by: [],
      database_storage_edges: [],
    };

    const assetMetadata = {
      Image: {
        dpi: [],
        color_space: [],
        base: {
          url: [],
          height: [],
          updated_at: BigInt(Date.now() * 1000000),
          asset_type: { Original: null },
          sha256: [],
          name: "original",
          storage_key: [],
          tags: ["test", "2asset", "original"],
          processing_error: [],
          mime_type: "image/jpeg",
          description: [],
          created_at: BigInt(Date.now() * 1000000),
          deleted_at: [],
          bytes: BigInt(0),
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

    const derivativeAssetMetadata = {
      Image: {
        dpi: [],
        color_space: [],
        base: {
          url: [],
          height: [],
          updated_at: BigInt(Date.now() * 1000000),
          asset_type: { Derivative: null },
          sha256: [],
          name: "derivative",
          storage_key: [],
          tags: ["test", "2asset", "derivative"],
          processing_error: [],
          mime_type: "image/jpeg",
          description: [],
          created_at: BigInt(Date.now() * 1000000),
          deleted_at: [],
          bytes: BigInt(0),
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

    const allAssets = [
      { blob_id: originalUpload.blobId, metadata: assetMetadata },
      { blob_id: derivativeUpload.blobId, metadata: derivativeAssetMetadata },
    ];

    const memoryResult = await backend.memories_create_with_internal_blobs(
      capsuleId,
      memoryMetadata,
      allAssets,
      `memory-2asset-${Date.now()}`
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
    const blob1Read = await backend.blob_read(originalUpload.blobId);
    if ("Ok" in blob1Read) {
      throw new Error("Original asset still exists after deletion");
    }

    const blob2Read = await backend.blob_read(derivativeUpload.blobId);
    if ("Ok" in blob2Read) {
      throw new Error("Derivative asset still exists after deletion");
    }
    echoPass("Both assets successfully deleted");

    echoPass("🎉 Quick 2-Asset Test PASSED!");
  } catch (error) {
    echoFail(`Test failed: ${error.message}`);
    process.exit(1);
  }
}

// Run the test
test2AssetQuick();
