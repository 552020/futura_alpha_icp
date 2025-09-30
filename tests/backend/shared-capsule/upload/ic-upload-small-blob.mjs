#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { HttpAgent, Actor } from "@dfinity/agent";
import fetch from "node-fetch";
import { loadDfxIdentity, makeMainnetAgent } from "./ic-identity.js";

// Adjust to your local replica or mainnet gateway
const HOST = process.env.IC_HOST || "http://127.0.0.1:4943";
const CANISTER_ID = process.env.BACKEND_CANISTER_ID || process.env.BACKEND_ID || "backend";
const IS_MAINNET = process.env.IC_HOST === "https://ic0.app" || process.env.IC_HOST === "https://icp0.io";

// Import the backend interface
import { idlFactory } from "../../../../src/nextjs/src/ic/declarations/backend/backend.did.js";

// Function to create the appropriate agent based on network
async function createAgent() {
  try {
    // Load DFX identity for both local and mainnet
    console.log("Loading DFX identity...");
    const identity = loadDfxIdentity();
    console.log(`Using DFX identity: ${identity.getPrincipal().toString()}`);

    if (IS_MAINNET) {
      return makeMainnetAgent(identity);
    } else {
      // Use DFX identity for local replica too
      const agent = new HttpAgent({ host: HOST, identity, fetch });
      // Fetch root key for local replica
      await agent.fetchRootKey();
      return agent;
    }
  } catch (error) {
    console.error("Failed to load DFX identity:", error.message);
    throw error;
  }
}

async function createSmallBlob(capsuleId, data, filename = "test-blob.txt") {
  const startTime = Date.now();
  console.log(`Creating small blob: "${data}" (${data.length} bytes)`);
  console.log(`Capsule ID: ${capsuleId}`);

  // Create the appropriate agent for the network
  const agent = await createAgent();

  console.log(`Using ${IS_MAINNET ? "MAINNET" : "LOCAL"} mode`);
  console.log(`Host: ${HOST}`);
  console.log(`Canister ID: ${CANISTER_ID}`);

  const backend = Actor.createActor(idlFactory, {
    agent,
    canisterId: CANISTER_ID,
  });

  const dataBuffer = Buffer.from(data, "utf8");
  const dataSize = dataBuffer.length;
  const totalChunks = Math.ceil(dataSize / 65536); // 64KiB chunks

  console.log(`Data size: ${dataSize} bytes, will upload in ${totalChunks} chunks`);

  // 1) Get or create a test capsule
  console.log("🔍 Getting test capsule...");
  let capsuleResult = await backend.capsules_read_basic([]);
  let actualCapsuleId;

  if ("Ok" in capsuleResult && capsuleResult.Ok) {
    actualCapsuleId = capsuleResult.Ok.capsule_id;
    console.log(`✅ Using existing capsule: ${actualCapsuleId}`);
  } else {
    console.log("🆕 No capsule found, creating one...");
    const createResult = await backend.capsules_create([]);
    if (!("Ok" in createResult)) {
      console.error("❌ Failed to create capsule:", createResult);
      throw new Error("Failed to create capsule: " + JSON.stringify(createResult));
    }
    actualCapsuleId = createResult.Ok.id;
    console.log(`✅ Created new capsule: ${actualCapsuleId}`);
  }

  // 2) Begin upload session
  console.log("🚀 Starting upload session...");
  const assetMetadata = {
    Document: {
      document_type: [],
      base: {
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000), // Convert to nanoseconds
        asset_type: { Original: null },
        sha256: [],
        name: filename,
        storage_key: [],
        tags: ["test-blob", "small-upload"],
        processing_error: [],
        mime_type: "text/plain",
        description: [`Test blob: ${data}`],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(dataSize),
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      language: [],
      page_count: [],
      word_count: [],
    },
  };

  console.log("📋 Upload session configuration:", {
    capsuleId: actualCapsuleId,
    totalChunks,
    dataSize,
    data: data,
  });

  const begin = await backend.uploads_begin(actualCapsuleId, assetMetadata, totalChunks, `small-blob-${Date.now()}`);

  if (!("Ok" in begin)) {
    console.error("❌ uploads_begin failed:", begin);
    throw new Error("uploads_begin failed: " + JSON.stringify(begin));
  }

  const session = begin.Ok;
  console.log(`✅ Upload session started: ${session}`);

  // 2) Upload the single chunk (since it's small)
  console.log("📦 Uploading data chunk...");
  const uploadStartTime = Date.now();

  const put = await backend.uploads_put_chunk(session, 0, new Uint8Array(dataBuffer));

  if (!("Ok" in put)) {
    console.error(`❌ put_chunk failed:`, put);
    throw new Error(`put_chunk failed: ` + JSON.stringify(put));
  }

  const uploadEndTime = Date.now();
  const uploadDuration = uploadEndTime - uploadStartTime;

  console.log(`✅ Chunk uploaded (${dataSize} bytes)`);
  console.log(`⏱️ Upload time: ${uploadDuration}ms`);

  // 3) Compute data hash
  console.log("🔐 Computing data hash...");
  const hash = crypto.createHash("sha256").update(dataBuffer).digest();
  const hashHex = hash.toString("hex");
  console.log(`✅ Data hash: ${hashHex}`);

  // 4) Finish upload
  console.log("🏁 Finishing upload...");
  const fin = await backend.uploads_finish(session, hash, BigInt(dataSize));

  if (!("Ok" in fin)) {
    console.error("❌ uploads_finish failed:", fin);
    throw new Error("uploads_finish failed: " + JSON.stringify(fin));
  }

  const totalTime = Date.now() - startTime;

  console.log("🎉 Small blob creation completed successfully!");
  console.log("📋 Result:", fin.Ok);
  console.log(`⏱️ Total time: ${totalTime}ms`);
  console.log(`🔐 Final hash: ${hashHex}`);

  return {
    blobId: fin.Ok,
    hash: hashHex,
    size: dataSize,
    data: data,
  };
}

async function main() {
  const args = process.argv.slice(2);

  if (args.length < 2) {
    console.error("Usage: node ic-upload-small-blob.mjs <capsule_id> <data> [filename]");
    console.error("Examples:");
    console.error("  node ic-upload-small-blob.mjs capsule_1234567890 'A' 'test-a.txt'");
    console.error("  node ic-upload-small-blob.mjs capsule_1234567890 'Hello World' 'hello.txt'");
    console.error("Environment variables:");
    console.error("  IC_HOST - IC host (default: http://127.0.0.1:4943)");
    console.error("  BACKEND_ID - Backend canister ID (default: backend)");
    process.exit(1);
  }

  const [capsuleId, data, filename = "test-blob.txt"] = args;

  try {
    const result = await createSmallBlob(capsuleId, data, filename);
    console.log("\n🎯 Blob creation result:");
    console.log(`   Blob ID: ${result.blobId}`);
    console.log(`   Hash: ${result.hash}`);
    console.log(`   Size: ${result.size} bytes`);
    console.log(`   Data: "${result.data}"`);
  } catch (error) {
    console.error("❌ Small blob creation failed:", error.message);
    process.exit(1);
  }
}

main();
