// ============================================================================
// ACCESS INDEX SYSTEM - Centralized Access Control
// ============================================================================

import { Perm, combinePermissions, hasPermission } from "./permissions.js";

// Resource types (same as Rust version)
const ResourceType = {
  Memory: "Memory",
  Gallery: "Gallery",
  Folder: "Folder",
  Capsule: "Capsule",
};

// Grant sources
const GrantSource = {
  User: "User",
  Group: "Group",
  MagicLink: "MagicLink",
  PublicMode: "PublicMode",
  System: "System",
};

// Resource roles
const ResourceRole = {
  Owner: "Owner",
  SuperAdmin: "SuperAdmin",
  Admin: "Admin",
  Member: "Member",
  Guest: "Guest",
};

// Public modes
const PublicMode = {
  Private: "Private",
  PublicAuth: "PublicAuth",
  PublicLink: "PublicLink",
};

// Resource key - identifies any resource
class ResKey {
  constructor(type, id) {
    this.type = type;
    this.id = id;
  }

  toString() {
    return `${this.type}:${this.id}`;
  }
}

// Access entry - who has access to what
class AccessEntry {
  constructor(id, personRef, grantSource, sourceId, role, permMask, invitedBy, createdAt, updatedAt) {
    this.id = id;
    this.person_ref = personRef;
    this.grant_source = grantSource;
    this.source_id = sourceId;
    this.role = role;
    this.perm_mask = permMask;
    this.invited_by_person_ref = invitedBy;
    this.created_at = createdAt;
    this.updated_at = updatedAt;
  }
}

// Public policy - public access rules
class PublicPolicy {
  constructor(mode, permMask, createdAt, updatedAt) {
    this.mode = mode;
    this.perm_mask = permMask;
    this.created_at = createdAt;
    this.updated_at = updatedAt;
  }
}

// Principal context - who is asking for access
class PrincipalContext {
  constructor(principal, groups = [], link = null) {
    this.principal = principal;
    this.groups = groups;
    this.link = link;
    this.now_ns = Date.now() * 1000000; // Convert to nanoseconds
  }
}

// Access index - centralized storage for all access control
class AccessIndex {
  constructor() {
    // Map of ResKey -> [AccessEntry]
    this.entries = new Map();
    // Map of ResKey -> PublicPolicy
    this.policy = new Map();
  }

  // Add access entry for a resource
  addAccess(resKey, accessEntry) {
    if (!this.entries.has(resKey.toString())) {
      this.entries.set(resKey.toString(), []);
    }
    this.entries.get(resKey.toString()).push(accessEntry);
  }

  // Set public policy for a resource
  setPublicPolicy(resKey, policy) {
    this.policy.set(resKey.toString(), policy);
  }

  // Get access entries for a resource
  getAccessEntries(resKey) {
    return this.entries.get(resKey.toString()) || [];
  }

  // Get public policy for a resource
  getPublicPolicy(resKey) {
    return this.policy.get(resKey.toString());
  }
}

// Permission evaluation - the core logic
function effectivePermMask(key, ctx, idx, capsule) {
  // 1) Ownership fast-path - owners get everything
  if (isOwner(key, ctx, capsule)) {
    return combinePermissions(Perm.VIEW, Perm.DOWNLOAD, Perm.SHARE, Perm.MANAGE, Perm.OWN);
  }

  let mask = 0;

  // 2) Direct grants - check individual access entries
  const entries = idx.getAccessEntries(key);
  mask |= sumUserAndGroups(entries, ctx);

  // 3) Magic link - check if valid magic link is presented
  if (ctx.link) {
    mask |= linkMaskIfValid(key, ctx.link, idx, ctx.now_ns);
  }

  // 4) Public policy - check public access rules
  mask |= publicMaskIfAny(key, idx, ctx, ctx.now_ns);

  return mask;
}

// Check if principal is owner of the resource
function isOwner(key, ctx, capsule) {
  switch (key.type) {
    case ResourceType.Memory:
      // Check if memory exists and user is owner/controller
      return capsule.memories.has(key.id) && (capsule.isOwner(ctx.principal) || capsule.isController(ctx.principal));

    case ResourceType.Gallery:
      // Check if gallery exists and user is owner/controller
      return capsule.galleries.has(key.id) && (capsule.isOwner(ctx.principal) || capsule.isController(ctx.principal));

    case ResourceType.Folder:
      // TODO: Implement when folders are added
      return false;

    case ResourceType.Capsule:
      // User is owner/controller of the capsule itself
      return capsule.isOwner(ctx.principal) || capsule.isController(ctx.principal);

    default:
      return false;
  }
}

// Sum permissions from user and group memberships
function sumUserAndGroups(entries, ctx) {
  let mask = 0;

  for (const entry of entries) {
    // Direct user match
    if (entry.person_ref === ctx.principal) {
      mask |= entry.perm_mask;
    }

    // TODO: Add group membership checks
    // if (ctx.groups.includes(entry.source_id)) {
    //     mask |= entry.perm_mask;
    // }
  }

  return mask;
}

// Check magic link validity (simplified)
function linkMaskIfValid(key, token, idx, nowNs) {
  // TODO: Implement magic link validation
  // - Hash token
  // - Look up in access index
  // - Check expiration
  // - Return appropriate permission mask
  return 0;
}

// Check public policy access
function publicMaskIfAny(key, idx, ctx, nowNs) {
  const policy = idx.getPublicPolicy(key);
  if (!policy) return 0;

  switch (policy.mode) {
    case PublicMode.Private:
      return 0;

    case PublicMode.PublicAuth:
      // Authenticated users get policy permissions
      return policy.perm_mask;

    case PublicMode.PublicLink:
      // Only if magic link is presented
      return ctx.link ? policy.perm_mask : 0;

    default:
      return 0;
  }
}

export {
  ResourceType,
  GrantSource,
  ResourceRole,
  PublicMode,
  ResKey,
  AccessEntry,
  PublicPolicy,
  PrincipalContext,
  AccessIndex,
  effectivePermMask,
  isOwner,
  sumUserAndGroups,
  linkMaskIfValid,
  publicMaskIfAny,
};
