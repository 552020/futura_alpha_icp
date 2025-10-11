/**
 * Backend Actor Creation Utilities
 *
 * Provides standardized actor creation for testing
 * Based on ICP expert guidance for certificate verification
 */

import { Actor } from "@dfinity/agent";
import { createTestAgent, getEnvironmentInfo } from "./agent.js";
import { idlFactory } from "../../declarations/backend/backend.did.js";

/**
 * Create a test actor with proper configuration
 * @param {Object} options - Actor configuration options
 * @returns {Promise<Object>} Configured actor and agent
 */
export async function createTestActor(options = {}) {
  const { canisterId = null, agent = null, ...agentOptions } = options;

  try {
    // Get environment info
    const env = getEnvironmentInfo();
    const testCanisterId = canisterId || env.canisterId;

    // Create agent if not provided
    const testAgent = agent || (await createTestAgent(agentOptions));

    // Create actor
    const actor = Actor.createActor(idlFactory, {
      agent: testAgent,
      canisterId: testCanisterId,
    });

    return { actor, agent: testAgent, canisterId: testCanisterId };
  } catch (error) {
    throw new Error(`Failed to create test actor: ${error.message}`);
  }
}

/**
 * Create actor with specific canister ID
 * @param {string} canisterId - Canister ID
 * @param {Object} options - Additional options
 * @returns {Promise<Object>} Actor and agent
 */
export async function createActorForCanister(canisterId, options = {}) {
  return await createTestActor({
    canisterId,
    ...options,
  });
}

/**
 * Create actor with mainnet configuration
 * @param {string} canisterId - Canister ID
 * @param {Object} options - Additional options
 * @returns {Promise<Object>} Mainnet actor and agent
 */
export async function createMainnetActor(canisterId, options = {}) {
  return await createTestActor({
    canisterId,
    agent: await createTestAgent({ host: "https://ic0.app" }),
    ...options,
  });
}

/**
 * Create actor with local replica configuration
 * @param {string} canisterId - Canister ID
 * @param {Object} options - Additional options
 * @returns {Object} Local actor and agent
 */
export function createLocalActor(canisterId, options = {}) {
  const { agent, ...agentOptions } = options;

  return createTestActor({
    canisterId,
    agent: agent || createTestAgent(agentOptions),
    ...options,
  });
}
