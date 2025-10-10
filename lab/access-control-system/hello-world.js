// ============================================================================
// HELLO WORLD ACCESS CONTROL - Super Simple
// ============================================================================

// Simple permissions - just 4 basic ones
const PERM = {
  VIEW: 1, // 1
  EDIT: 2, // 2
  DELETE: 4, // 4
  SHARE: 8, // 8
};

// Simple access entry - who can do what
class Access {
  constructor(userId, permissions) {
    this.userId = userId;
    this.permissions = permissions;
  }

  can(permission) {
    return (this.permissions & permission) === permission;
  }
}

// Simple access manager - stores who has access to what
class AccessManager {
  constructor() {
    // Map: objectId -> [Access]
    this.access = new Map();
  }

  // Give someone access to something
  giveAccess(objectId, userId, permissions) {
    if (!this.access.has(objectId)) {
      this.access.set(objectId, []);
    }

    const objectAccess = this.access.get(objectId);

    // Remove old access if exists
    const existing = objectAccess.findIndex((a) => a.userId === userId);
    if (existing !== -1) {
      objectAccess.splice(existing, 1);
    }

    // Add new access
    objectAccess.push(new Access(userId, permissions));
  }

  // Check if user can do something
  canUser(objectId, userId, permission) {
    const objectAccess = this.access.get(objectId);
    if (!objectAccess) return false;

    const userAccess = objectAccess.find((a) => a.userId === userId);
    if (!userAccess) return false;

    return userAccess.can(permission);
  }

  // Show who has access to what
  showAccess(objectId) {
    const objectAccess = this.access.get(objectId);
    if (!objectAccess) {
      console.log(`No one has access to ${objectId}`);
      return;
    }

    console.log(`Access to ${objectId}:`);
    for (const access of objectAccess) {
      const perms = [];
      if (access.can(PERM.VIEW)) perms.push("VIEW");
      if (access.can(PERM.EDIT)) perms.push("EDIT");
      if (access.can(PERM.DELETE)) perms.push("DELETE");
      if (access.can(PERM.SHARE)) perms.push("SHARE");
      console.log(`  - ${access.userId}: ${perms.join(", ")}`);
    }
  }
}

// Demo
function helloWorld() {
  console.log("🌍 Hello World Access Control\n");

  const access = new AccessManager();

  // Create some simple objects
  const objects = ["obj1", "obj2", "obj3"];
  const users = ["alice", "bob", "charlie"];

  console.log("📦 Objects:", objects.join(", "));
  console.log("👥 Users:", users.join(", "));
  console.log("");

  // Give access
  console.log("🔑 Giving Access:\n");

  // Alice owns obj1
  access.giveAccess("obj1", "alice", PERM.VIEW | PERM.EDIT | PERM.DELETE | PERM.SHARE);
  console.log("  - Alice gets full access to obj1");

  // Bob can view and edit obj1
  access.giveAccess("obj1", "bob", PERM.VIEW | PERM.EDIT);
  console.log("  - Bob gets VIEW + EDIT access to obj1");

  // Charlie can only view obj1
  access.giveAccess("obj1", "charlie", PERM.VIEW);
  console.log("  - Charlie gets VIEW access to obj1");

  // Alice owns obj2
  access.giveAccess("obj2", "alice", PERM.VIEW | PERM.EDIT | PERM.DELETE | PERM.SHARE);
  console.log("  - Alice gets full access to obj2");

  // Bob can view obj2
  access.giveAccess("obj2", "bob", PERM.VIEW);
  console.log("  - Bob gets VIEW access to obj2");

  console.log("");

  // Test permissions
  console.log("🔍 Testing Permissions:\n");

  const tests = [
    { user: "alice", object: "obj1", action: "VIEW", can: true },
    { user: "alice", object: "obj1", action: "EDIT", can: true },
    { user: "alice", object: "obj1", action: "DELETE", can: true },
    { user: "bob", object: "obj1", action: "VIEW", can: true },
    { user: "bob", object: "obj1", action: "EDIT", can: true },
    { user: "bob", object: "obj1", action: "DELETE", can: false },
    { user: "charlie", object: "obj1", action: "VIEW", can: true },
    { user: "charlie", object: "obj1", action: "EDIT", can: false },
    { user: "charlie", object: "obj1", action: "DELETE", can: false },
    { user: "bob", object: "obj2", action: "VIEW", can: true },
    { user: "bob", object: "obj2", action: "EDIT", can: false },
    { user: "charlie", object: "obj2", action: "VIEW", can: false },
  ];

  for (const test of tests) {
    const canDo = access.canUser(test.object, test.user, PERM[test.action]);
    const status = canDo === test.can ? "✅" : "❌";
    console.log(`  ${status} ${test.user} can ${test.action} ${test.object}: ${canDo}`);
  }

  console.log("");

  // Show access lists
  console.log("📋 Access Lists:\n");
  for (const object of objects) {
    access.showAccess(object);
    console.log("");
  }

  console.log("🎯 That's it! Simple access control between users and objects.");
}

// Export the function
export { helloWorld };

// Run it
helloWorld();
