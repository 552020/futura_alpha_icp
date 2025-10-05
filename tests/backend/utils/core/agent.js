/**
 * ICP Agent Setup Utilities
 *
 * Provides standardized agent creation for testing
 * Based on ICP expert guidance for certificate verification
 */

import { HttpAgent } from "@dfinity/agent";
import { loadDfxIdentity, makeMainnetAgent } from "../../shared-capsule/upload/ic-identity.js";

// Runtime fetch detection (Node vs Browser)
let runtimeFetch;
try {
  // Browser: global fetch exists
  runtimeFetch = fetch;
} catch {
  // Node: use node-fetch
  runtimeFetch = (await import("node-fetch")).default;
}

// Configuration
const HOST = process.env.IC_HOST || "http://127.0.0.1:4943";
const IS_MAINNET = process.env.IC_HOST === "https://ic0.app" || process.env.IC_HOST === "https://icp0.io";

/**
 * Create a test agent with proper configuration
 * Based on ICP expert guidance for certificate verification
 * @param {Object} options - Agent configuration options
 * @returns {Promise<HttpAgent>} Configured agent
 */
export async function createTestAgent(options = {}) {
  const { host = HOST, identity = null, dev = !IS_MAINNET } = options;

  try {
    let agent;

    if (IS_MAINNET) {
      agent = await makeMainnetAgent(identity);
    } else {
      const testIdentity = identity || loadDfxIdentity();
      agent = new HttpAgent({
        host,
        identity: testIdentity,
        fetch: runtimeFetch, // Use runtime-appropriate fetch
        // Optional: skip query sigs for faster non-certified queries locally
        verifyQuerySignatures: !dev,
      });

      // CRITICAL for local dfx: trust local root key
      if (dev) {
        await agent.fetchRootKey();
      }
    }

    return agent;
  } catch (error) {
    throw new Error(`Failed to create test agent: ${error.message}`);
  }
}

/**
 * Create agent with mainnet configuration
 * @param {Object} identity - Optional identity
 * @returns {Promise<HttpAgent>} Mainnet agent
 */
export async function createMainnetAgent(identity = null) {
  return await makeMainnetAgent(identity);
}

/**
 * Create agent with local replica configuration
 * @param {Object} options - Local agent options
 * @returns {HttpAgent} Local agent
 */
export function createLocalAgent(options = {}) {
  const { host = HOST, verifyQuerySignatures = false, identity = null, fetch = null } = options;

  const testIdentity = identity || loadDfxIdentity();

  return new HttpAgent({
    host,
    identity: testIdentity,
    verifyQuerySignatures,
    fetch,
  });
}

/**
 * Get current environment configuration
 * @returns {Object} Environment info
 */
export function getEnvironmentInfo() {
  return {
    host: HOST,
    isMainnet: IS_MAINNET,
    canisterId: process.env.BACKEND_CANISTER_ID || process.env.BACKEND_ID || "uxrrr-q7777-77774-qaaaq-cai",
  };
}
