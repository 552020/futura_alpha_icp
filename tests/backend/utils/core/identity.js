/**
 * Identity Management Utilities
 *
 * Provides identity handling for testing
 */

import { loadDfxIdentity, makeMainnetAgent } from "../../shared-capsule/upload/ic-identity.js";

/**
 * Get current DFX identity
 * @returns {Object} DFX identity
 */
export function getCurrentIdentity() {
  return loadDfxIdentity();
}

/**
 * Get identity principal as string
 * @returns {string} Principal string
 */
export function getCurrentPrincipal() {
  const identity = getCurrentIdentity();
  return identity.getPrincipal().toString();
}

/**
 * Create mainnet agent with identity
 * @param {Object} identity - Optional identity
 * @returns {Promise<Object>} Mainnet agent
 */
export async function createMainnetAgentWithIdentity(identity = null) {
  return await makeMainnetAgent(identity);
}

/**
 * Get identity information for logging
 * @returns {Object} Identity info
 */
export function getIdentityInfo() {
  const identity = getCurrentIdentity();
  const principal = identity.getPrincipal();

  return {
    principal: principal.toString(),
    principalId: principal.toText(),
    identityType: identity.constructor.name,
  };
}

