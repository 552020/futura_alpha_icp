// ============================================================================
// PERMISSION SYSTEM - Bitflags for Access Control
// ============================================================================

// Bitflags for permissions (same as Rust version)
const Perm = {
  VIEW: 1 << 0, // 1
  DOWNLOAD: 1 << 1, // 2
  SHARE: 1 << 2, // 4
  MANAGE: 1 << 3, // 8
  OWN: 1 << 4, // 16
};

// Helper function to combine permissions
function combinePermissions(...perms) {
  return perms.reduce((acc, perm) => acc | perm, 0);
}

// Helper function to check if permission is granted
function hasPermission(grantedMask, requiredPerm) {
  return (grantedMask & requiredPerm) === requiredPerm;
}

// Helper function to get permission names
function getPermissionNames(mask) {
  const names = [];
  if (mask & Perm.VIEW) names.push("VIEW");
  if (mask & Perm.DOWNLOAD) names.push("DOWNLOAD");
  if (mask & Perm.SHARE) names.push("SHARE");
  if (mask & Perm.MANAGE) names.push("MANAGE");
  if (mask & Perm.OWN) names.push("OWN");
  return names;
}

// Role templates (same as Rust version)
const RoleTemplates = {
  owner: {
    name: "owner",
    perm_mask: combinePermissions(Perm.VIEW, Perm.DOWNLOAD, Perm.SHARE, Perm.MANAGE, Perm.OWN),
    description: "Full ownership access",
  },
  admin: {
    name: "admin",
    perm_mask: combinePermissions(Perm.VIEW, Perm.DOWNLOAD, Perm.SHARE, Perm.MANAGE),
    description: "Administrative access",
  },
  member: {
    name: "member",
    perm_mask: combinePermissions(Perm.VIEW, Perm.DOWNLOAD),
    description: "Standard member access",
  },
  guest: {
    name: "guest",
    perm_mask: Perm.VIEW,
    description: "Read-only access",
  },
};

export { Perm, combinePermissions, hasPermission, getPermissionNames, RoleTemplates };
