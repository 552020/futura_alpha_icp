/**
 * Frontend Token Manager for HTTP Asset Serving
 *
 * Provides efficient token caching and bulk request handling for dashboard scenarios.
 * Reduces repeated token requests by caching tokens with expiry management.
 */

class TokenManager {
  constructor(actor) {
    this.actor = actor;
    this.cache = new Map(); // memory_id -> { token, expires_at, variants }
    this.bulkCache = new Map(); // cache_key -> { tokens, expires_at }
  }

  /**
   * Get a single token for a memory, using cache if available
   * @param {string} memoryId - The memory ID
   * @param {string[]} variants - Array of variants (e.g., ['thumbnail', 'preview'])
   * @param {string[]} assetIds - Optional specific asset IDs
   * @param {number} ttlSecs - Token TTL in seconds (default: 180)
   * @returns {Promise<string>} The token
   */
  async getToken(memoryId, variants = ["thumbnail"], assetIds = null, ttlSecs = 180) {
    const cacheKey = this._getCacheKey(memoryId, variants, assetIds);
    const cached = this.cache.get(cacheKey);

    // Check if cached token is still valid
    if (cached && Date.now() < cached.expires_at) {
      console.log(`🎯 Token cache hit for memory: ${memoryId}`);
      return cached.token;
    }

    console.log(`🔄 Fetching fresh token for memory: ${memoryId}`);

    try {
      // Convert null to empty array for Candid compatibility
      const assetIdsParam = assetIds ? assetIds : [];
      const token = await this.actor.mint_http_token(memoryId, variants, assetIdsParam, ttlSecs);

      // Cache the token with expiry
      const expiresAt = Date.now() + ttlSecs * 1000 - 10000; // 10s buffer
      this.cache.set(cacheKey, {
        token,
        expires_at: expiresAt,
        variants,
        asset_ids: assetIds,
      });

      return token;
    } catch (error) {
      console.error(`❌ Failed to mint token for memory ${memoryId}:`, error);
      throw error;
    }
  }

  /**
   * Get tokens for multiple memories in a single bulk request
   * @param {string[]} memoryIds - Array of memory IDs
   * @param {string[]} variants - Array of variants
   * @param {string[]} assetIds - Optional specific asset IDs
   * @param {number} ttlSecs - Token TTL in seconds (default: 180)
   * @returns {Promise<Map<string, string>>} Map of memory_id -> token
   */
  async getBulkTokens(memoryIds, variants = ["thumbnail"], assetIds = null, ttlSecs = 180) {
    const bulkCacheKey = this._getBulkCacheKey(memoryIds, variants, assetIds);
    const cached = this.bulkCache.get(bulkCacheKey);

    // Check if bulk cache is still valid
    if (cached && Date.now() < cached.expires_at) {
      console.log(`🎯 Bulk token cache hit for ${memoryIds.length} memories`);
      return new Map(cached.tokens);
    }

    console.log(`🔄 Fetching bulk tokens for ${memoryIds.length} memories`);

    try {
      // Convert null to empty array for Candid compatibility
      const assetIdsParam = assetIds ? assetIds : [];
      const tokenPairs = await this.actor.mint_http_tokens_bulk(memoryIds, variants, assetIdsParam, ttlSecs);

      // Convert to Map and update individual cache
      const tokenMap = new Map();
      const expiresAt = Date.now() + ttlSecs * 1000 - 10000; // 10s buffer

      for (const [memoryId, token] of tokenPairs) {
        tokenMap.set(memoryId, token);

        // Update individual cache
        const cacheKey = this._getCacheKey(memoryId, variants, assetIds);
        this.cache.set(cacheKey, {
          token,
          expires_at: expiresAt,
          variants,
          asset_ids: assetIds,
        });
      }

      // Update bulk cache
      this.bulkCache.set(bulkCacheKey, {
        tokens: Array.from(tokenMap.entries()),
        expires_at: expiresAt,
      });

      console.log(`✅ Bulk tokens fetched: ${tokenMap.size}/${memoryIds.length} successful`);
      return tokenMap;
    } catch (error) {
      console.error(`❌ Failed to mint bulk tokens:`, error);
      throw error;
    }
  }

  /**
   * Get asset URL for a memory with automatic token management
   * @param {string} memoryId - The memory ID
   * @param {string} variant - The variant (thumbnail, preview, original)
   * @param {string} assetId - Optional specific asset ID
   * @param {string} baseUrl - Base URL for the HTTP gateway
   * @returns {Promise<string>} Complete asset URL with token
   */
  async getAssetUrl(memoryId, variant, assetId = null, baseUrl = "http://localhost:4943") {
    const variants = [variant];
    const assetIds = assetId ? [assetId] : null;

    const token = await this.getToken(memoryId, variants, assetIds);

    let url = `${baseUrl}/asset/${memoryId}/${variant}`;
    if (assetId) {
      url += `?id=${encodeURIComponent(assetId)}&token=${encodeURIComponent(token)}`;
    } else {
      url += `?token=${encodeURIComponent(token)}`;
    }

    return url;
  }

  /**
   * Get asset URLs for multiple memories (dashboard scenario)
   * @param {string[]} memoryIds - Array of memory IDs
   * @param {string} variant - The variant to fetch
   * @param {string} baseUrl - Base URL for the HTTP gateway
   * @returns {Promise<Map<string, string>>} Map of memory_id -> asset_url
   */
  async getBulkAssetUrls(memoryIds, variant = "thumbnail", baseUrl = "http://localhost:4943") {
    const variants = [variant];
    const tokenMap = await this.getBulkTokens(memoryIds, variants);

    const urlMap = new Map();
    for (const [memoryId, token] of tokenMap) {
      const url = `${baseUrl}/asset/${memoryId}/${variant}?token=${encodeURIComponent(token)}`;
      urlMap.set(memoryId, url);
    }

    return urlMap;
  }

  /**
   * Clear expired tokens from cache
   */
  clearExpiredTokens() {
    const now = Date.now();

    // Clear individual cache
    for (const [key, value] of this.cache.entries()) {
      if (now >= value.expires_at) {
        this.cache.delete(key);
      }
    }

    // Clear bulk cache
    for (const [key, value] of this.bulkCache.entries()) {
      if (now >= value.expires_at) {
        this.bulkCache.delete(key);
      }
    }
  }

  /**
   * Clear all cached tokens
   */
  clearAllTokens() {
    this.cache.clear();
    this.bulkCache.clear();
  }

  /**
   * Get cache statistics
   * @returns {Object} Cache stats
   */
  getCacheStats() {
    const now = Date.now();
    let validTokens = 0;
    let expiredTokens = 0;

    for (const value of this.cache.values()) {
      if (now < value.expires_at) {
        validTokens++;
      } else {
        expiredTokens++;
      }
    }

    return {
      individual_cache: {
        total: this.cache.size,
        valid: validTokens,
        expired: expiredTokens,
      },
      bulk_cache: {
        total: this.bulkCache.size,
      },
    };
  }

  /**
   * Generate cache key for individual token
   * @private
   */
  _getCacheKey(memoryId, variants, assetIds) {
    const variantStr = variants.sort().join(",");
    const assetStr = assetIds ? assetIds.sort().join(",") : "null";
    return `${memoryId}:${variantStr}:${assetStr}`;
  }

  /**
   * Generate cache key for bulk tokens
   * @private
   */
  _getBulkCacheKey(memoryIds, variants, assetIds) {
    const memoryStr = memoryIds.sort().join(",");
    const variantStr = variants.sort().join(",");
    const assetStr = assetIds ? assetIds.sort().join(",") : "null";
    return `bulk:${memoryStr}:${variantStr}:${assetStr}`;
  }
}

// Export for use in tests
export { TokenManager };
export default TokenManager;

// Example usage:
/*
const tokenManager = new TokenManager(actor);

// Single token
const token = await tokenManager.getToken('memory-123', ['thumbnail']);

// Bulk tokens for dashboard
const memoryIds = ['memory-1', 'memory-2', 'memory-3'];
const tokenMap = await tokenManager.getBulkTokens(memoryIds, ['thumbnail']);

// Get asset URLs directly
const assetUrl = await tokenManager.getAssetUrl('memory-123', 'thumbnail');
const bulkUrls = await tokenManager.getBulkAssetUrls(memoryIds, 'thumbnail');

// Cache management
tokenManager.clearExpiredTokens();
console.log(tokenManager.getCacheStats());
*/
