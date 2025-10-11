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
import { idlFactory } from "./declarations/backend/backend.did.js";

const CHUNK_SIZE = parseInt(process.env.CHUNK_SIZE || "65536", 10); // 64KiB default

// Function to create the appropriate agent based on network
async function createAgent() {
  if (IS_MAINNET) {
    try {
      // Load DFX identity for mainnet
      console.log("Loading DFX identity for mainnet...");
      const identity = loadDfxIdentity();
      console.log(`Using DFX identity: ${identity.getPrincipal().toString()}`);
      return makeMainnetAgent(identity);
    } catch (error) {
      console.error("Failed to load DFX identity:", error.message);
      throw error;
    }
  } else {
    // Use anonymous identity for local replica
    console.log("Using anonymous identity for local replica");
    const agent = new HttpAgent({ host: HOST, fetch });
    // Fetch root key for local replica
    await agent.fetchRootKey();
    return agent;
  }
}

async function main(filePath) {
  const startTime = Date.now();
  console.log(`Starting upload of ${filePath} to canister ${CANISTER_ID}`);
  console.log(`Using chunk size: ${CHUNK_SIZE} bytes`);

  // Create the appropriate agent for the network
  const agent = await createAgent();

  console.log(`Using ${IS_MAINNET ? "MAINNET" : "LOCAL"} mode`);
  console.log(`Host: ${HOST}`);
  console.log(`Canister ID: ${CANISTER_ID}`);

  const backend = Actor.createActor(idlFactory, {
    agent,
    canisterId: CANISTER_ID,
  });

  const fileStats = fs.statSync(filePath);
  const fileSize = fileStats.size;
  const totalChunks = Math.ceil(fileSize / CHUNK_SIZE);

  console.log(`File size: ${fileSize} bytes, will upload in ${totalChunks} chunks`);

  // 1) Get or create a test capsule
  console.log("🔍 Getting test capsule...");
  let capsuleResult = await backend.capsules_read_basic([]);
  let capsuleId;

  if ("Ok" in capsuleResult && capsuleResult.Ok) {
    capsuleId = capsuleResult.Ok.capsule_id;
    console.log(`✅ Using existing capsule: ${capsuleId}`);
  } else {
    console.log("🆕 No capsule found, creating one...");
    const createResult = await backend.capsules_create([]);
    if (!("Ok" in createResult)) {
      console.error("❌ Failed to create capsule:", createResult);
      throw new Error("Failed to create capsule: " + JSON.stringify(createResult));
    }
    capsuleId = createResult.Ok.id;
    console.log(`✅ Created new capsule: ${capsuleId}`);
  }

  // 2) Begin upload session
  console.log("🚀 Starting upload session...");
  const assetMetadata = {
    Image: {
      base: {
        url: [],
        height: [],
        updated_at: BigInt(Date.now() * 1000000), // Convert to nanoseconds
        asset_type: { Original: null },
        sha256: [],
        name: path.basename(filePath),
        storage_key: [],
        tags: ["upload-test", "node-uploader"],
        processing_error: [],
        mime_type: "application/octet-stream", // Default mime type
        description: [`Uploaded file: ${filePath}`],
        created_at: BigInt(Date.now() * 1000000),
        deleted_at: [],
        bytes: BigInt(fileSize),
        asset_location: [],
        width: [],
        processing_status: [],
        bucket: [],
      },
      dpi: [],
      color_space: [],
      exif_data: [],
      compression_ratio: [],
      orientation: [],
    },
  };

  console.log("📋 Upload session configuration:", {
    capsuleId,
    totalChunks,
    fileSize,
    chunkSize: CHUNK_SIZE,
  });

  const begin = await backend.uploads_begin(capsuleId, assetMetadata, totalChunks, `upload-${Date.now()}`);

  if (!("Ok" in begin)) {
    console.error("❌ uploads_begin failed:", begin);
    throw new Error("uploads_begin failed: " + JSON.stringify(begin));
  }

  const session = begin.Ok;
  console.log(`✅ Upload session started: ${session}`);

  // 3) Stream chunks
  console.log("📦 Starting chunk upload process...");
  const uploadStartTime = Date.now();
  const fd = fs.openSync(filePath, "r");
  const buf = Buffer.alloc(CHUNK_SIZE);
  let index = 0;
  let uploadedBytes = 0;

  for (;;) {
    const read = fs.readSync(fd, buf, 0, CHUNK_SIZE, null);
    if (read <= 0) break;

    const chunk = new Uint8Array(buf.subarray(0, read));
    console.log(`📤 Uploading chunk ${index + 1}/${totalChunks} (${read} bytes)`);

    const put = await backend.uploads_put_chunk(session, index, chunk);

    if (!("Ok" in put)) {
      console.error(`❌ put_chunk ${index} failed:`, put);
      throw new Error(`put_chunk ${index} failed: ` + JSON.stringify(put));
    }

    uploadedBytes += read;
    const percentage = (uploadedBytes / fileSize) * 100;
    console.log(`✅ Chunk ${index + 1}/${totalChunks} uploaded (${percentage.toFixed(1)}%)`);
    index++;
  }
  fs.closeSync(fd);

  const uploadEndTime = Date.now();
  const uploadDuration = uploadEndTime - uploadStartTime;
  const uploadSpeed = uploadedBytes / 1024 / 1024 / (uploadDuration / 1000); // MB/s

  console.log(`✅ All chunks uploaded (${uploadedBytes} bytes total)`);
  console.log(`⏱️ Upload time: ${uploadDuration}ms (${(uploadDuration / 1000).toFixed(2)}s)`);
  console.log(`🚀 Upload speed: ${uploadSpeed.toFixed(2)} MB/s`);

  // 4) Compute file hash
  console.log("🔐 Computing file hash...");
  const fileBuffer = fs.readFileSync(filePath);
  const hash = crypto.createHash("sha256").update(fileBuffer).digest();
  const hashHex = hash.toString("hex");
  console.log(`✅ File hash: ${hashHex}`);

  // 5) Finish upload
  console.log("🏁 Finishing upload...");
  const fin = await backend.uploads_finish(session, hash, BigInt(fileSize));

  if (!("Ok" in fin)) {
    console.error("❌ uploads_finish failed:", fin);
    throw new Error("uploads_finish failed: " + JSON.stringify(fin));
  }

  const totalTime = Date.now() - startTime;
  const totalSpeed = fileSize / 1024 / 1024 / (totalTime / 1000); // MB/s

  console.log("🎉 Upload completed successfully!");
  console.log("📋 Result:", fin.Ok);
  console.log(`⏱️ Total time: ${totalTime}ms (${(totalTime / 1000).toFixed(2)}s)`);
  console.log(`🚀 Total speed: ${totalSpeed.toFixed(2)} MB/s`);
  console.log(`🔐 Final hash: ${hashHex}`);
  return fin.Ok;
}

if (process.argv.length < 3) {
  console.error("Usage: node tools/ic-upload.mjs <file>");
  console.error("Environment variables:");
  console.error("  IC_HOST - IC host (default: http://127.0.0.1:4943)");
  console.error("  BACKEND_ID - Backend canister ID (default: backend)");
  console.error("  CHUNK_SIZE - Chunk size in bytes (default: 65536)");
  process.exit(1);
}

main(process.argv[2]).catch((e) => {
  console.error("Upload failed:", e.message);
  process.exit(1);
});
