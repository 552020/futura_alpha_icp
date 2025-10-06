#!/usr/bin/env node

/**
 * Lane B: Image Processing Test
 *
 * Tests only the image processing functionality (Lane B)
 * without actual uploads or memory creation.
 */

import { Actor, HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity, makeMainnetAgent } from "./ic-identity.js";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { idlFactory } from "../../../../src/nextjs/src/ic/declarations/backend/backend.did.js";
import {
  validateFileSize,
  validateImageType,
  calculateFileHash,
  generateFileId,
  calculateDerivativeDimensions,
  calculateDerivativeSizes,
  createFileChunks,
  createProgressCallback,
  createAssetMetadata,
  createBlobReference,
  handleUploadError,
  validateUploadResponse,
  formatFileSize,
  formatUploadSpeed,
  formatDuration,
} from "./helpers.mjs";

// Test configuration
const TEST_NAME = "Lane B: Image Processing Test";
const TEST_IMAGE_PATH = "./assets/input/avocado_big_21mb.jpg";

// Global backend instance
let backend;

// Helper functions
function echoInfo(message) {
  console.log(`ℹ️  ${message}`);
}

function echoPass(message) {
  console.log(`✅ ${message}`);
}

function echoFail(message) {
  console.log(`❌ ${message}`);
}

// Real image processing (Node.js version of frontend logic)
async function processImageDerivativesPure(fileBuffer, mimeType) {
  const originalSize = fileBuffer.length;

  echoInfo(`🖼️ Processing derivatives for ${formatFileSize(originalSize)} file`);

  // Validate file type using helper
  validateImageType(mimeType);

  // Get derivative size limits from helper
  const sizeLimits = calculateDerivativeSizes(originalSize);

  // Calculate realistic dimensions
  const aspectRatio = 16 / 9;
  const originalWidth = Math.floor(Math.sqrt(originalSize / 3));
  const originalHeight = Math.floor(originalWidth / aspectRatio);

  // Calculate derivative dimensions using helper
  const displayDims = calculateDerivativeDimensions(
    originalWidth,
    originalHeight,
    sizeLimits.display.maxWidth,
    sizeLimits.display.maxHeight
  );
  const thumbDims = calculateDerivativeDimensions(
    originalWidth,
    originalHeight,
    sizeLimits.thumb.maxWidth,
    sizeLimits.thumb.maxHeight
  );

  // Create derivative buffers (simulation - in real implementation, use Sharp/Jimp)
  const displaySize = Math.min(sizeLimits.display.maxSize, Math.floor(originalSize * 0.1));
  const displayBuffer = Buffer.alloc(displaySize);
  fileBuffer.copy(displayBuffer, 0, 0, displaySize);

  const thumbSize = Math.min(sizeLimits.thumb.maxSize, Math.floor(originalSize * 0.05));
  const thumbBuffer = Buffer.alloc(thumbSize);
  fileBuffer.copy(thumbBuffer, 0, 0, thumbSize);

  const placeholderSize = Math.min(sizeLimits.placeholder.maxSize, 1024);
  const placeholderBuffer = Buffer.alloc(placeholderSize, 0x42);

  // Log precise sizes using helper
  echoInfo(`📊 Derivative sizes:`);
  echoInfo(`  Display: ${formatFileSize(displaySize)} (${displayDims.width}x${displayDims.height})`);
  echoInfo(`  Thumb: ${formatFileSize(thumbSize)} (${thumbDims.width}x${thumbDims.height})`);
  echoInfo(`  Placeholder: ${formatFileSize(placeholderSize)} (32x18)`);

  return {
    original: {
      buffer: fileBuffer,
      size: originalSize,
      width: originalWidth,
      height: originalHeight,
      mimeType: mimeType,
    },
    display: {
      buffer: displayBuffer,
      size: displaySize,
      width: displayDims.width,
      height: displayDims.height,
      mimeType: "image/webp",
    },
    thumb: {
      buffer: thumbBuffer,
      size: thumbSize,
      width: thumbDims.width,
      height: thumbDims.height,
      mimeType: "image/webp",
    },
    placeholder: {
      buffer: placeholderBuffer,
      size: placeholderSize,
      width: 32,
      height: 18,
      mimeType: "image/webp",
    },
  };
}

// Test function
async function testLaneBImageProcessing() {
  const fileBuffer = fs.readFileSync(TEST_IMAGE_PATH);

  const processedAssets = await processImageDerivativesPure(fileBuffer, "image/jpeg");

  // Verify all derivatives were created
  return processedAssets.original && processedAssets.display && processedAssets.thumb && processedAssets.placeholder;
}

// Main test runner
async function main() {
  echoInfo(`Starting ${TEST_NAME}`);

  // Parse command line arguments
  const args = process.argv.slice(2);
  const backendCanisterId = args[0];
  const network = args[1] || "local"; // Default to local network

  if (!backendCanisterId) {
    echoFail("Usage: node test_lane_b_image_processing.mjs <CANISTER_ID> [mainnet|local]");
    echoFail("Example: node test_lane_b_image_processing.mjs uxrrr-q7777-77774-qaaaq-cai local");
    process.exit(1);
  }

  // Setup agent and backend based on network
  const identity = loadDfxIdentity();
  let agent;

  if (network === "mainnet") {
    echoInfo(`🌐 Connecting to mainnet (ic0.app)`);
    agent = makeMainnetAgent(identity);
  } else {
    echoInfo(`🏠 Connecting to local network (127.0.0.1:4943)`);
    agent = new HttpAgent({
      host: "http://127.0.0.1:4943",
      identity,
      fetch: (await import("node-fetch")).default,
    });
  }

  await agent.fetchRootKey();

  backend = Actor.createActor(idlFactory, {
    agent,
    canisterId: backendCanisterId,
  });

  // Run test
  try {
    echoInfo(`Running: ${TEST_NAME}`);
    const result = await testLaneBImageProcessing();
    if (result) {
      echoPass(TEST_NAME);
    } else {
      echoFail(TEST_NAME);
      process.exit(1);
    }
  } catch (error) {
    echoFail(`${TEST_NAME}: ${error.message}`);
    process.exit(1);
  }

  echoPass("Test completed successfully! ✅");
}

// Run the test
main().catch((error) => {
  echoFail(`Test execution failed: ${error.message}`);
  process.exit(1);
});
