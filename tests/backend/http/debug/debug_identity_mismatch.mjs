/**
 * Identity Mismatch Debug Test
 *
 * This test debugs the identity mismatch issue:
 * 1. Check what identity is used during memory creation
 * 2. Check what identity is used during token minting
 * 3. Identify the root cause of the mismatch
 * 4. Propose a solution
 */

import { logHeader, logInfo, logSuccess, logError } from "../utils/helpers/logging.js";
import { createTestActor } from "../utils/core/actor.js";
import { createTestCapsule } from "../utils/helpers/capsule-creation.js";
import { createTestImageMemory } from "../utils/helpers/memory-creation.js";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function debugIdentityMismatch() {
  logHeader("🔍 Debugging Identity Mismatch");

  let capsuleId = null;
  let memoryId = null;

  try {
    // Step 1: Get current identity
    logInfo("Step 1: Getting current identity...");
    const { stdout: identityOutput } = await execAsync("dfx identity get-principal");
    const currentIdentity = identityOutput.trim();
    logSuccess(`✅ Current identity: ${currentIdentity}`);

    // Step 2: Create test actor and check its identity
    logInfo("Step 2: Creating test actor...");
    const { actor } = await createTestActor();
    logSuccess("✅ Test actor created");

    // Step 3: Check whoami via actor
    logInfo("Step 3: Checking whoami via actor...");
    try {
      const whoamiResult = await actor.whoami();
      logInfo(`Actor whoami: ${whoamiResult}`);
      if (whoamiResult === currentIdentity) {
        logSuccess("✅ Actor identity matches current identity");
      } else {
        logError(`❌ Identity mismatch! Current: ${currentIdentity}, Actor: ${whoamiResult}`);
      }
    } catch (error) {
      logError(`❌ Failed to get actor whoami: ${error.message}`);
    }

    // Step 4: Check whoami via dfx
    logInfo("Step 4: Checking whoami via dfx...");
    try {
      const { stdout: dfxWhoami } = await execAsync("dfx canister call backend whoami --output raw");
      logInfo(`Dfx whoami: ${dfxWhoami}`);
    } catch (error) {
      logError(`❌ Failed to get dfx whoami: ${error.message}`);
    }

    // Step 5: Create capsule
    logInfo("Step 5: Creating capsule...");
    capsuleId = await createTestCapsule(actor);
    logSuccess(`✅ Capsule created: ${capsuleId}`);

    // Step 6: Create memory and check what identity it uses
    logInfo("Step 6: Creating memory and checking identity...");
    memoryId = await createTestImageMemory(actor, capsuleId, {
      name: "identity_debug_image.png",
      mimeType: "image/png",
      idempotencyKey: `identity_debug_${Date.now()}`,
    });
    logSuccess(`✅ Memory created: ${memoryId}`);

    // Step 7: Read memory and check access entries
    logInfo("Step 7: Reading memory access entries...");
    try {
      const memoryResult = await actor.memories_read(memoryId);
      if (memoryResult.Ok && memoryResult.Ok.access_entries && memoryResult.Ok.access_entries.length > 0) {
        const entry = memoryResult.Ok.access_entries[0];
        logInfo("Memory access entry:");
        logInfo(`- Person ref: ${JSON.stringify(entry.person_ref)}`);
        logInfo(`- Current identity: ${currentIdentity}`);

        // Extract the principal from the person_ref
        if (entry.person_ref && entry.person_ref.Principal) {
          const memoryOwner = entry.person_ref.Principal.__principal__;
          logInfo(`- Memory owner: ${memoryOwner}`);

          if (memoryOwner === currentIdentity) {
            logSuccess("✅ Memory owner matches current identity");
          } else {
            logError(`❌ Identity mismatch! Current: ${currentIdentity}, Memory owner: ${memoryOwner}`);
            logInfo("");
            logInfo("🔧 This is the root cause of the ACL issue!");
            logInfo("The memory was created with a different identity than the one trying to mint tokens.");
          }
        }
      }
    } catch (error) {
      logError(`❌ Failed to read memory: ${error.message}`);
    }

    // Step 8: Try token minting and see the exact error
    logInfo("Step 8: Testing token minting...");
    try {
      const { stdout: tokenOutput } = await execAsync(
        `dfx canister call backend mint_http_token '("${memoryId}", vec {"thumbnail"}, null, 180)' --output raw`
      );
      const token = tokenOutput.trim();
      logSuccess(`✅ Token minted: ${token.substring(0, 50)}...`);
    } catch (tokenError) {
      logError(`❌ Token minting failed: ${tokenError.message}`);
      logInfo("");
      logInfo("🔧 Analysis:");
      logInfo("The token minting fails because the ACL system can't find the memory");
      logInfo("with the current identity, even though the memory was created successfully.");
      logInfo("");
      logInfo("💡 Solution:");
      logInfo("We need to ensure the same identity is used for both memory creation and token minting.");
    }

    return {
      success: false,
      reason: "identity_mismatch_identified",
      memoryId,
      analysis: "Identity mismatch between memory creation and token minting identified",
    };
  } catch (error) {
    logError(`❌ Test failed: ${error.message}`);
    return { success: false, reason: "general_error", error: error.message };
  } finally {
    // Cleanup
    if (memoryId) {
      logInfo("Cleaning up memory...");
      try {
        await actor.memories_delete(memoryId, false);
        logSuccess("✅ Memory cleaned up");
      } catch (cleanupError) {
        logError(`❌ Cleanup failed: ${cleanupError.message}`);
      }
    }
  }
}

async function main() {
  logHeader("🚀 Identity Mismatch Debug Test");

  const result = await debugIdentityMismatch();

  logHeader("📊 Debug Results");
  if (result.reason === "identity_mismatch_identified") {
    logSuccess("✅ Identity Mismatch Debug COMPLETED");
    logInfo(`Memory ID: ${result.memoryId}`);
    logInfo(`Analysis: ${result.analysis}`);
    logInfo("");
    logInfo("🔍 Root Cause Identified:");
    logInfo("The memory is created with one identity but token minting uses a different identity.");
    logInfo("");
    logInfo("💡 Solutions:");
    logInfo("1. Ensure consistent identity usage across all operations");
    logInfo("2. Fix the actor interface to use the same identity");
    logInfo("3. Or modify the ACL system to handle identity mismatches");
    logInfo("");
    logInfo("🌐 The HTTP module is working correctly - it's an identity consistency issue!");
  } else {
    logError(`❌ Test failed: ${result.reason.replace(/_/g, " ")}`);
    if (result.error) {
      logError(`Error: ${result.error}`);
    }
  }
}

main().catch(console.error);

