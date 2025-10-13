#!/usr/bin/env node

/**
 * Debug Actor Functions
 *
 * This script lists all available functions in the actor interface
 * to help us understand what's available.
 */

import { logHeader, logInfo, logSuccess, logError } from "../utils/helpers/logging.js";
import { createTestActor } from "../utils/core/actor.js";

async function debugActorFunctions() {
  logHeader("🔍 Debugging Actor Functions");

  try {
    const { actor } = await createTestActor();

    logInfo("Available functions in actor:");
    const functions = Object.getOwnPropertyNames(actor).filter(
      (name) => typeof actor[name] === "function" && !name.startsWith("_")
    );

    functions.forEach((func) => {
      logInfo(`  - ${func}`);
    });

    logInfo("");
    logInfo("Functions containing 'mint' or 'http':");
    const relevantFunctions = functions.filter(
      (name) => name.toLowerCase().includes("mint") || name.toLowerCase().includes("http")
    );

    if (relevantFunctions.length > 0) {
      relevantFunctions.forEach((func) => {
        logInfo(`  ✅ ${func}`);
      });
    } else {
      logInfo("  ❌ No functions found containing 'mint' or 'http'");
    }

    logInfo("");
    logInfo("All function names (for reference):");
    functions.forEach((func) => {
      logInfo(`  ${func}`);
    });
  } catch (error) {
    logError(`❌ Failed to debug actor functions: ${error.message}`);
  }
}

debugActorFunctions().catch(console.error);

