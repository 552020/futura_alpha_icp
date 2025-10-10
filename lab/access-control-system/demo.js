// ============================================================================
// CAPSULE - Access Control System
// ============================================================================

import { Perm, combinePermissions, hasPermission, getPermissionNames, RoleTemplates } from "./permissions.js";
import {
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
} from "./access-index.js";

// Capsule for access control
class Capsule {
  constructor() {
    this.memories = new Set(["memory_1", "memory_2"]);
    this.galleries = new Set(["gallery_1"]);
    this.owners = new Set(["alice"]);
    this.controllers = new Set(["bob"]);
  }

  isOwner(principal) {
    return this.owners.has(principal);
  }

  isController(principal) {
    return this.controllers.has(principal);
  }
}

function runDemo() {
  console.log("🔐 Access Control System Demo\n");

  // Create a capsule
  const capsule = new Capsule();

  // Create access index
  const accessIndex = new AccessIndex();

  // Create some resources
  const memory1 = new ResKey(ResourceType.Memory, "memory_1");
  const memory2 = new ResKey(ResourceType.Memory, "memory_2");
  const gallery1 = new ResKey(ResourceType.Gallery, "gallery_1");

  console.log("📁 Resources created:");
  console.log(`  - ${memory1.toString()}`);
  console.log(`  - ${memory2.toString()}`);
  console.log(`  - ${gallery1.toString()}\n`);

  // Add some access entries
  const aliceAccess = new AccessEntry(
    "access_1",
    "alice",
    GrantSource.User,
    null,
    ResourceRole.Owner,
    RoleTemplates.owner.perm_mask,
    null,
    Date.now(),
    Date.now()
  );

  const bobAccess = new AccessEntry(
    "access_2",
    "bob",
    GrantSource.User,
    null,
    ResourceRole.Admin,
    RoleTemplates.admin.perm_mask,
    "alice",
    Date.now(),
    Date.now()
  );

  const charlieAccess = new AccessEntry(
    "access_3",
    "charlie",
    GrantSource.User,
    null,
    ResourceRole.Member,
    RoleTemplates.member.perm_mask,
    "alice",
    Date.now(),
    Date.now()
  );

  // Add access to resources
  accessIndex.addAccess(memory1, aliceAccess);
  accessIndex.addAccess(memory1, bobAccess);
  accessIndex.addAccess(memory1, charlieAccess);

  accessIndex.addAccess(memory2, aliceAccess);
  accessIndex.addAccess(memory2, bobAccess);

  accessIndex.addAccess(gallery1, aliceAccess);
  accessIndex.addAccess(gallery1, charlieAccess);

  console.log("👥 Access entries added:");
  console.log(`  - Alice (owner): ${getPermissionNames(aliceAccess.perm_mask).join(", ")}`);
  console.log(`  - Bob (admin): ${getPermissionNames(bobAccess.perm_mask).join(", ")}`);
  console.log(`  - Charlie (member): ${getPermissionNames(charlieAccess.perm_mask).join(", ")}\n`);

  // Add public policy for memory2
  const publicPolicy = new PublicPolicy(
    PublicMode.PublicAuth,
    combinePermissions(Perm.VIEW, Perm.DOWNLOAD),
    Date.now(),
    Date.now()
  );
  accessIndex.setPublicPolicy(memory2, publicPolicy);

  console.log("🌐 Public policy added:");
  console.log(
    `  - ${memory2.toString()}: ${publicPolicy.mode} (${getPermissionNames(publicPolicy.perm_mask).join(", ")})\n`
  );

  // Test permission evaluation
  console.log("🔍 Permission Evaluation Tests:\n");

  const testCases = [
    { user: "alice", resource: memory1, description: "Alice accessing her own memory" },
    { user: "bob", resource: memory1, description: "Bob accessing memory as admin" },
    { user: "charlie", resource: memory1, description: "Charlie accessing memory as member" },
    { user: "dave", resource: memory1, description: "Dave accessing memory (no access)" },
    { user: "alice", resource: memory2, description: "Alice accessing memory with public policy" },
    { user: "dave", resource: memory2, description: "Dave accessing public memory" },
    { user: "alice", resource: gallery1, description: "Alice accessing gallery" },
    { user: "charlie", resource: gallery1, description: "Charlie accessing gallery" },
    { user: "bob", resource: gallery1, description: "Bob accessing gallery (no access)" },
  ];

  for (const testCase of testCases) {
    const ctx = new PrincipalContext(testCase.user);
    const permissions = effectivePermMask(testCase.resource, ctx, accessIndex, capsule);
    const permissionNames = getPermissionNames(permissions);

    console.log(`  ${testCase.description}:`);
    console.log(`    User: ${testCase.user}`);
    console.log(`    Resource: ${testCase.resource.toString()}`);
    console.log(`    Permissions: ${permissionNames.length > 0 ? permissionNames.join(", ") : "NONE"}`);
    console.log(`    Can VIEW: ${hasPermission(permissions, Perm.VIEW) ? "✅" : "❌"}`);
    console.log(`    Can DOWNLOAD: ${hasPermission(permissions, Perm.DOWNLOAD) ? "✅" : "❌"}`);
    console.log(`    Can SHARE: ${hasPermission(permissions, Perm.SHARE) ? "✅" : "❌"}`);
    console.log(`    Can MANAGE: ${hasPermission(permissions, Perm.MANAGE) ? "✅" : "❌"}`);
    console.log(`    Can OWN: ${hasPermission(permissions, Perm.OWN) ? "✅" : "❌"}`);
    console.log("");
  }

  console.log("🎯 Key Benefits:");
  console.log("  ✅ Centralized access control - one place to check permissions");
  console.log("  ✅ Fine-grained permissions - VIEW, DOWNLOAD, SHARE, MANAGE, OWN");
  console.log("  ✅ Multiple access sources - direct grants, groups, public policies, magic links");
  console.log("  ✅ Efficient evaluation - ownership fast-path, bitwise operations");
  console.log("  ✅ Extensible - easy to add new resource types and permission types");
}

// Export the demo function
export { runDemo };
