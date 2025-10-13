#!/usr/bin/env node

/**
 * Test Same Identity Flow
 *
 * This test ensures we use the same identity throughout the entire flow
 * to avoid ACL permission issues.
 */

import { logHeader, logInfo, logSuccess, logError } from "../utils/helpers/logging.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function testSameIdentityFlow() {
  logHeader("🔄 Testing Same Identity Flow");

  let capsuleId = null;
  let memoryId = null;
  let token = null;

  try {
    // Step 1: Get current identity
    logInfo("Step 1: Getting current identity...");
    const { stdout: identityOutput } = await execAsync(`dfx identity whoami`);
    const currentIdentity = identityOutput.trim();
    logInfo(`Current identity: ${currentIdentity}`);

    const { stdout: principalOutput } = await execAsync(`dfx identity get-principal`);
    const currentPrincipal = principalOutput.trim();
    logInfo(`Current principal: ${currentPrincipal}`);

    // Step 2: Create capsule with current identity
    logInfo("Step 2: Creating capsule...");
    const createCapsuleCmd = `dfx canister call backend capsules_create 'null' --output raw`;
    const { stdout: capsuleOutput } = await execAsync(createCapsuleCmd);
    capsuleId = capsuleOutput.trim();
    logSuccess(`✅ Capsule created: ${capsuleId}`);

    // Step 3: Create memory with current identity
    logInfo("Step 3: Creating memory...");

    // Create a simple 1x1 PNG image
    const imageData = Buffer.from([
      0x89,
      0x50,
      0x4e,
      0x47,
      0x0d,
      0x0a,
      0x1a,
      0x0a, // PNG signature
      0x00,
      0x00,
      0x00,
      0x0d, // IHDR chunk length
      0x49,
      0x48,
      0x44,
      0x52, // IHDR
      0x00,
      0x00,
      0x00,
      0x01, // width: 1
      0x00,
      0x00,
      0x00,
      0x01, // height: 1
      0x08,
      0x02,
      0x00,
      0x00,
      0x00, // bit depth, color type, compression, filter, interlace
      0x90,
      0x77,
      0x53,
      0xde, // CRC
      0x00,
      0x00,
      0x00,
      0x0c, // IDAT chunk length
      0x49,
      0x44,
      0x41,
      0x54, // IDAT
      0x08,
      0x99,
      0x01,
      0x01,
      0x00,
      0x00,
      0x00,
      0xff,
      0xff,
      0x00,
      0x00,
      0x00,
      0x02,
      0x00,
      0x01,
      0x00,
      0x00,
      0x00,
      0x00, // compressed data
      0x49,
      0x45,
      0x4e,
      0x44, // IEND
      0xae,
      0x42,
      0x60,
      0x82, // CRC
    ]);

    const imageBytes = Array.from(imageData);
    const imageBytesStr = imageBytes.map((b) => `\\x${b.toString(16).padStart(2, "0")}`).join("");

    const createMemoryCmd = `dfx canister call backend memories_create '(
      "${capsuleId}",
      vec { blob "${imageBytesStr}" },
      vec {},
      vec {},
      vec {},
      vec {},
      vec {},
      vec { blob "\\x2c\\xf2\\x4d\\xba\\x4f\\x8a\\x6c\\xba\\x1f\\x86\\xb8\\xe7\\xfe\\x74\\xfa\\x8d\\x80\\x31\\x24\\xca\\x06\\x62\\xea\\x4a\\x06\\x97\\x3f\\x8a\\x3e\\x4c\\x06\\x97" },
      vec { ("name", "test_image.png"); ("mime_type", "image/png"); ("bytes", "${
        imageBytes.length
      }"); ("width", "1"); ("height", "1") },
      "test_memory_${Date.now()}"
    )' --output raw`;

    const { stdout: memoryOutput } = await execAsync(createMemoryCmd);
    memoryId = memoryOutput.trim();
    logSuccess(`✅ Memory created: ${memoryId}`);

    // Step 4: Verify we can read the memory with same identity
    logInfo("Step 4: Verifying memory read access...");
    const readMemoryCmd = `dfx canister call backend memories_read '("${memoryId}")' --output idl`;
    const { stdout: readOutput } = await execAsync(readMemoryCmd);

    if (readOutput.includes("Ok")) {
      logSuccess("✅ Memory read successful - we have access!");
    } else {
      logError("❌ Memory read failed");
      logInfo(`Read output: ${readOutput}`);
      return { success: false, reason: "memory_read_failed" };
    }

    // Step 5: Try to mint HTTP token with same identity
    logInfo("Step 5: Attempting to mint HTTP token...");
    const mintTokenCmd = `dfx canister call backend mint_http_token '("${memoryId}", vec { "thumbnail" }, null, 300)' --output raw`;

    try {
      const { stdout: tokenOutput } = await execAsync(mintTokenCmd);
      token = tokenOutput.trim();
      logSuccess(`✅ Token minted successfully: ${token}`);
    } catch (tokenError) {
      logError(`❌ Token minting failed: ${tokenError.message}`);
      return { success: false, reason: "token_minting_failed", error: tokenError.message };
    }

    // Step 6: Test HTTP access with token
    logInfo("Step 6: Testing HTTP access...");
    const canisterId = "uxrrr-q7777-77774-qaaaq-cai";
    const httpUrl = `http://${canisterId}.localhost:4943/asset/${memoryId}/thumbnail?token=${token}`;

    logInfo(`HTTP URL: ${httpUrl}`);

    try {
      const { stdout: curlOutput } = await execAsync(`curl -s -i ${httpUrl}`);
      logInfo(`Curl Response:\n${curlOutput}`);

      if (curlOutput.includes("HTTP/1.1 200 OK")) {
        logSuccess("✅ HTTP access successful! Complete flow works!");
        return { success: true, httpUrl, token, memoryId, capsuleId };
      } else {
        logError("❌ HTTP access failed");
        return { success: false, reason: "http_access_failed", curlOutput };
      }
    } catch (curlError) {
      logError(`❌ Curl test failed: ${curlError.message}`);
      return { success: false, reason: "curl_failed", error: curlError.message };
    }
  } catch (error) {
    logError(`❌ Test failed: ${error.message}`);
    return { success: false, reason: "general_error", error: error.message };
  } finally {
    // Cleanup
    if (memoryId) {
      try {
        logInfo("Cleaning up memory...");
        await execAsync(`dfx canister call backend memories_delete '("${memoryId}", true)'`);
        logSuccess("✅ Memory cleaned up");
      } catch (cleanupError) {
        logError(`❌ Cleanup failed: ${cleanupError.message}`);
      }
    }
  }
}

async function main() {
  logHeader("🚀 Same Identity Flow Test");

  const result = await testSameIdentityFlow();

  logHeader("📊 Test Results");
  if (result.success) {
    logSuccess("🎉 Same identity flow test PASSED!");
    logInfo(`Memory ID: ${result.memoryId}`);
    logInfo(`Token: ${result.token}`);
    logInfo(`HTTP URL: ${result.httpUrl}`);
    logInfo("");
    logInfo("🌐 You can now open this URL in your browser to see the image!");
  } else {
    logError(`❌ Same identity flow test FAILED: ${result.reason}`);
    if (result.error) {
      logError(`Error: ${result.error}`);
    }
    if (result.curlOutput) {
      logInfo("Curl output:");
      logInfo(result.curlOutput);
    }
  }
}

main().catch(console.error);
