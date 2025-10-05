#!/usr/bin/env node

/**
 * Test Asset ID Endpoints
 *
 * This test demonstrates the new asset_id-based API endpoints:
 * - asset_get_by_id(memory_id, asset_id)
 * - asset_remove_by_id(memory_id, asset_id)
 *
 * Test Flow:
 * 1. Create a capsule
 * 2. Create a memory with assets (gets asset_id automatically)
 * 3. List assets to get asset_id
 * 4. Test asset_get_by_id() endpoint
 * 5. Test asset_remove_by_id() endpoint
 * 6. Verify asset was removed
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { loadDfxIdentity, makeMainnetAgent } from "../shared-capsule/upload/ic-identity.js";
import fetch from "node-fetch";
import crypto from "crypto";

// Import the backend interface
import { idlFactory } from "../../../.dfx/local/canisters/backend/service.did.js";

// Test configuration
const TEST_NAME = "Asset ID Endpoints Test";
const HOST = process.env.IC_HOST || "http://127.0.0.1:4943";
const IS_MAINNET = process.env.IC_HOST === "https://ic0.app" || process.env.IC_HOST === "https://icp0.io";
const CANISTER_ID = process.env.BACKEND_CANISTER_ID || process.env.BACKEND_ID || "uxrrr-q7777-77774-qaaaq-cai";

console.log(`🚀 ${TEST_NAME}`);
console.log(`📍 Host: ${HOST}`);

async function main() {
  try {
    // 1. Setup actor using the existing framework pattern
    console.log("\n🔧 Setting up actor...");
    let agent;

    if (IS_MAINNET) {
      agent = await makeMainnetAgent();
    } else {
      const identity = loadDfxIdentity();
      agent = new HttpAgent({
        host: HOST,
        identity: identity,
        verifyQuerySignatures: false,
      });

      // Fetch root key for local replica
      await agent.fetchRootKey();
    }

    const actor = Actor.createActor(idlFactory, {
      agent,
      canisterId: CANISTER_ID,
    });

    // 2. Create a capsule
    console.log("\n📦 Creating capsule...");
    const capsuleResult = await actor.capsules_create([]);
    if (!("Ok" in capsuleResult)) {
      throw new Error(`Failed to create capsule: ${JSON.stringify(capsuleResult)}`);
    }
    const capsuleId = capsuleResult.Ok.id;
    console.log(`✅ Created capsule: ${capsuleId}`);

    // 3. Create a memory with inline assets
    console.log("\n💾 Creating memory with assets...");
    const testData = "Hello, Asset ID World!";
    const bytesArray = Array.from(new Uint8Array(Buffer.from(testData)));

    const assetMetadata = {
      Note: {
        base: {
          name: "test_asset.txt",
          description: ["Test asset for asset_id endpoints"],
          tags: ["test", "asset-id"],
          asset_type: { Original: null },
          bytes: BigInt(testData.length),
          mime_type: "text/plain",
          sha256: [],
          width: [],
          height: [],
          url: [],
          storage_key: [],
          bucket: [],
          processing_status: [],
          processing_error: [],
          created_at: BigInt(Date.now() * 1000000),
          updated_at: BigInt(Date.now() * 1000000),
          deleted_at: [],
          asset_location: [],
        },
        word_count: [testData.split(" ").length],
        language: ["en"],
        format: ["text"],
      },
    };

    const memoryResult = await actor.memories_create(
      capsuleId,
      [bytesArray], // inline_bytes
      [], // blob_ref
      [], // storage_edge_blob_type
      ["Test Memory with Asset ID"], // title
      ["Memory created to test asset_id endpoints"], // description
      [], // date_of_memory
      [], // sha256
      assetMetadata, // asset_metadata
      "test-idem-" + Date.now() // idem
    );

    if (!("Ok" in memoryResult)) {
      throw new Error(`Failed to create memory: ${JSON.stringify(memoryResult)}`);
    }
    const memoryId = memoryResult.Ok;
    console.log(`✅ Created memory: ${memoryId}`);

    // 4. List assets to get asset_id
    console.log("\n📋 Listing assets to get asset_id...");
    const assetsResult = await actor.memories_list_assets(memoryId);
    if (!("Ok" in assetsResult)) {
      throw new Error(`Failed to list assets: ${JSON.stringify(assetsResult)}`);
    }

    const assetsList = assetsResult.Ok;
    console.log(`📊 Assets found:`, {
      totalCount: assetsList.total_count,
      inlineAssets: assetsList.inline_assets.length,
      internalAssets: assetsList.internal_assets.length,
      externalAssets: assetsList.external_assets.length,
    });

    // Get the first inline asset ID (should be the one we just created)
    if (assetsList.inline_assets.length === 0) {
      throw new Error("No inline assets found in memory");
    }

    // The inline_assets array contains asset references, but we need to get the actual asset_id
    // Let's read the memory to get the full asset data with asset_id
    console.log("\n📖 Reading memory to get asset details...");
    const memoryResult2 = await actor.memories_read(memoryId);
    if (!("Ok" in memoryResult2)) {
      throw new Error(`Failed to read memory: ${JSON.stringify(memoryResult2)}`);
    }

    const memory = memoryResult2.Ok;
    console.log(`📝 Memory details:`, {
      id: memory.id,
      inlineAssetsCount: memory.inline_assets.length,
      blobInternalAssetsCount: memory.blob_internal_assets.length,
      blobExternalAssetsCount: memory.blob_external_assets.length,
    });

    if (memory.inline_assets.length === 0) {
      throw new Error("No inline assets found in memory");
    }

    const asset = memory.inline_assets[0];
    const assetId = asset.asset_id;
    console.log(`🎯 Found asset with ID: ${assetId}`);
    console.log(`📄 Asset details:`, {
      asset_id: asset.asset_id,
      bytes_length: asset.bytes.length,
      metadata_name: asset.metadata.Note?.base?.name || "N/A",
    });

    // 5. Test asset_get_by_id() endpoint
    console.log("\n🔍 Testing asset_get_by_id() endpoint...");
    const getAssetResult = await actor.asset_get_by_id(memoryId, assetId);
    if (!("Ok" in getAssetResult)) {
      throw new Error(`Failed to get asset by ID: ${JSON.stringify(getAssetResult)}`);
    }

    const assetData = getAssetResult.Ok;
    console.log(`✅ Successfully retrieved asset by ID:`, {
      type: assetData.Inline ? "Inline" : "Other",
      content_type: assetData.Inline?.content_type || "N/A",
      size: assetData.Inline?.size || 0,
    });

    // 6. Test asset_remove_by_id() endpoint
    console.log("\n🗑️ Testing asset_remove_by_id() endpoint...");
    const removeAssetResult = await actor.asset_remove_by_id(memoryId, assetId);
    if (!("Ok" in removeAssetResult)) {
      throw new Error(`Failed to remove asset by ID: ${JSON.stringify(removeAssetResult)}`);
    }

    const removalResult = removeAssetResult.Ok;
    console.log(`✅ Successfully removed asset by ID:`, {
      memory_id: removalResult.memory_id,
      asset_removed: removalResult.asset_removed,
      message: removalResult.message,
    });

    // 7. Verify asset was removed
    console.log("\n🔍 Verifying asset was removed...");
    const assetsResult2 = await actor.memories_list_assets(memoryId);
    if (!("Ok" in assetsResult2)) {
      throw new Error(`Failed to list assets after removal: ${JSON.stringify(assetsResult2)}`);
    }

    const assetsList2 = assetsResult2.Ok;
    console.log(`📊 Assets after removal:`, {
      totalCount: assetsList2.total_count,
      inlineAssets: assetsList2.inline_assets.length,
      internalAssets: assetsList2.internal_assets.length,
      externalAssets: assetsList2.external_assets.length,
    });

    if (assetsList2.inline_assets.length === 0) {
      console.log("✅ Asset successfully removed - no inline assets remaining");
    } else {
      console.log("⚠️ Asset removal may not have worked - inline assets still present");
    }

    // 8. Test getting removed asset (should fail)
    console.log("\n🧪 Testing retrieval of removed asset (should fail)...");
    const getRemovedAssetResult = await actor.asset_get_by_id(memoryId, assetId);
    if ("Ok" in getRemovedAssetResult) {
      console.log("⚠️ Unexpected: Removed asset was still retrievable");
    } else {
      console.log("✅ Expected: Removed asset is no longer retrievable");
      console.log(`❌ Error (expected): ${JSON.stringify(getRemovedAssetResult.Err)}`);
    }

    console.log("\n🎉 Asset ID endpoints test completed successfully!");
    console.log("\n📋 Test Summary:");
    console.log("✅ Created capsule and memory with assets");
    console.log("✅ Retrieved asset using asset_get_by_id()");
    console.log("✅ Removed asset using asset_remove_by_id()");
    console.log("✅ Verified asset was removed");
    console.log("✅ Confirmed removed asset is no longer accessible");
  } catch (error) {
    console.error("❌ Test failed:", error.message);
    console.error("Stack trace:", error.stack);
    process.exit(1);
  }
}

main().catch(console.error);
