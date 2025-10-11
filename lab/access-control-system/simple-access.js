// ============================================================================
// SIMPLE ACCESS CONTROL SYSTEM - Just Entity Sharing
// ============================================================================

// Simple permissions
const Permissions = {
  VIEW: 1 << 0, // 1
  EDIT: 1 << 1, // 2
  DELETE: 1 << 2, // 4
  SHARE: 1 << 3, // 8
};

// Helper functions
function combinePermissions(...perms) {
  return perms.reduce((acc, perm) => acc | perm, 0);
}

function hasPermission(grantedMask, requiredPerm) {
  return (grantedMask & requiredPerm) === requiredPerm;
}

function getPermissionNames(mask) {
  const names = [];
  if (mask & Permissions.VIEW) names.push("VIEW");
  if (mask & Permissions.EDIT) names.push("EDIT");
  if (mask & Permissions.DELETE) names.push("DELETE");
  if (mask & Permissions.SHARE) names.push("SHARE");
  return names;
}

// Simple access entry - who has access to what
class AccessEntry {
  constructor(entityId, userId, permissions, grantedBy = null) {
    this.entityId = entityId;
    this.userId = userId;
    this.permissions = permissions;
    this.grantedBy = grantedBy;
    this.grantedAt = Date.now();
  }
}

// Simple access manager - stores all access relationships
class AccessManager {
  constructor() {
    // Map of entityId -> [AccessEntry]
    this.accessMap = new Map();
  }

  // Grant access to an entity
  grantAccess(entityId, userId, permissions, grantedBy = null) {
    if (!this.accessMap.has(entityId)) {
      this.accessMap.set(entityId, []);
    }

    const entries = this.accessMap.get(entityId);

    // Remove existing access for this user
    const existingIndex = entries.findIndex((entry) => entry.userId === userId);
    if (existingIndex !== -1) {
      entries.splice(existingIndex, 1);
    }

    // Add new access
    entries.push(new AccessEntry(entityId, userId, permissions, grantedBy));
  }

  // Revoke access to an entity
  revokeAccess(entityId, userId) {
    if (!this.accessMap.has(entityId)) return;

    const entries = this.accessMap.get(entityId);
    const index = entries.findIndex((entry) => entry.userId === userId);
    if (index !== -1) {
      entries.splice(index, 1);
    }
  }

  // Check if user has permission for entity
  hasPermission(entityId, userId, requiredPermission) {
    if (!this.accessMap.has(entityId)) return false;

    const entries = this.accessMap.get(entityId);
    const userEntry = entries.find((entry) => entry.userId === userId);

    if (!userEntry) return false;

    return hasPermission(userEntry.permissions, requiredPermission);
  }

  // Get all permissions for user on entity
  getUserPermissions(entityId, userId) {
    if (!this.accessMap.has(entityId)) return 0;

    const entries = this.accessMap.get(entityId);
    const userEntry = entries.find((entry) => entry.userId === userId);

    return userEntry ? userEntry.permissions : 0;
  }

  // Get all users who have access to entity
  getEntityAccess(entityId) {
    return this.accessMap.get(entityId) || [];
  }

  // Get all entities user has access to
  getUserEntities(userId) {
    const entities = [];
    for (const [entityId, entries] of this.accessMap) {
      if (entries.some((entry) => entry.userId === userId)) {
        entities.push(entityId);
      }
    }
    return entities;
  }
}

// Demo function
function runSimpleDemo() {
  console.log("🔐 Simple Access Control System Demo\n");

  const accessManager = new AccessManager();

  // Create some entities
  const entities = ["document_1", "spreadsheet_2", "presentation_3"];
  const users = ["alice", "bob", "charlie", "dave"];

  console.log("📁 Entities:", entities.join(", "));
  console.log("👥 Users:", users.join(", "));
  console.log("");

  // Grant some access
  console.log("🔑 Granting Access:\n");

  // Alice owns document_1
  accessManager.grantAccess(
    "document_1",
    "alice",
    combinePermissions(Permissions.VIEW, Permissions.EDIT, Permissions.DELETE, Permissions.SHARE)
  );
  console.log("  - Alice gets full access to document_1");

  // Bob can view and edit document_1
  accessManager.grantAccess("document_1", "bob", combinePermissions(Permissions.VIEW, Permissions.EDIT), "alice");
  console.log("  - Bob gets VIEW+EDIT access to document_1 (granted by Alice)");

  // Charlie can only view document_1
  accessManager.grantAccess("document_1", "charlie", Permissions.VIEW, "alice");
  console.log("  - Charlie gets VIEW access to document_1 (granted by Alice)");

  // Alice owns spreadsheet_2
  accessManager.grantAccess(
    "spreadsheet_2",
    "alice",
    combinePermissions(Permissions.VIEW, Permissions.EDIT, Permissions.DELETE, Permissions.SHARE)
  );
  console.log("  - Alice gets full access to spreadsheet_2");

  // Bob can view spreadsheet_2
  accessManager.grantAccess("spreadsheet_2", "bob", Permissions.VIEW, "alice");
  console.log("  - Bob gets VIEW access to spreadsheet_2 (granted by Alice)");

  // Charlie owns presentation_3
  accessManager.grantAccess(
    "presentation_3",
    "charlie",
    combinePermissions(Permissions.VIEW, Permissions.EDIT, Permissions.DELETE, Permissions.SHARE)
  );
  console.log("  - Charlie gets full access to presentation_3");

  console.log("");

  // Test permission checks
  console.log("🔍 Permission Tests:\n");

  const testCases = [
    { user: "alice", entity: "document_1", permission: "VIEW", expected: true },
    { user: "alice", entity: "document_1", permission: "EDIT", expected: true },
    { user: "alice", entity: "document_1", permission: "DELETE", expected: true },
    { user: "alice", entity: "document_1", permission: "SHARE", expected: true },

    { user: "bob", entity: "document_1", permission: "VIEW", expected: true },
    { user: "bob", entity: "document_1", permission: "EDIT", expected: true },
    { user: "bob", entity: "document_1", permission: "DELETE", expected: false },
    { user: "bob", entity: "document_1", permission: "SHARE", expected: false },

    { user: "charlie", entity: "document_1", permission: "VIEW", expected: true },
    { user: "charlie", entity: "document_1", permission: "EDIT", expected: false },
    { user: "charlie", entity: "document_1", permission: "DELETE", expected: false },
    { user: "charlie", entity: "document_1", permission: "SHARE", expected: false },

    { user: "dave", entity: "document_1", permission: "VIEW", expected: false },
    { user: "dave", entity: "document_1", permission: "EDIT", expected: false },

    { user: "bob", entity: "spreadsheet_2", permission: "VIEW", expected: true },
    { user: "bob", entity: "spreadsheet_2", permission: "EDIT", expected: false },

    { user: "charlie", entity: "presentation_3", permission: "VIEW", expected: true },
    { user: "charlie", entity: "presentation_3", permission: "EDIT", expected: true },
    { user: "alice", entity: "presentation_3", permission: "VIEW", expected: false },
  ];

  for (const test of testCases) {
    const hasAccess = accessManager.hasPermission(test.entity, test.user, Permissions[test.permission]);
    const status = hasAccess === test.expected ? "✅" : "❌";
    console.log(`  ${status} ${test.user} can ${test.permission} ${test.entity}: ${hasAccess}`);
  }

  console.log("");

  // Show entity access lists
  console.log("📋 Entity Access Lists:\n");
  for (const entity of entities) {
    const access = accessManager.getEntityAccess(entity);
    console.log(`  ${entity}:`);
    for (const entry of access) {
      const perms = getPermissionNames(entry.permissions);
      console.log(`    - ${entry.userId}: ${perms.join(", ")} (granted by ${entry.grantedBy || "owner"})`);
    }
    console.log("");
  }

  // Show user entity lists
  console.log("👤 User Entity Lists:\n");
  for (const user of users) {
    const entities = accessManager.getUserEntities(user);
    if (entities.length > 0) {
      console.log(`  ${user} has access to: ${entities.join(", ")}`);
    } else {
      console.log(`  ${user} has no access to any entities`);
    }
  }

  console.log("");
  console.log("🎯 Key Benefits:");
  console.log("  ✅ Simple - just entities and users");
  console.log("  ✅ Flexible - any entity can be shared with any user");
  console.log("  ✅ Granular - fine-grained permissions (VIEW, EDIT, DELETE, SHARE)");
  console.log("  ✅ Trackable - know who granted access and when");
  console.log("  ✅ Efficient - fast permission checks");
}

// Run the demo
runSimpleDemo();
