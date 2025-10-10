// ============================================================================
// SIMPLE OBJECTS - Full Access Control System on Simple Objects
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

// Simple object manager - like a capsule but for basic objects
class SimpleObjectManager {
  constructor() {
    this.objects = new Set(["doc1", "doc2", "image1", "folder1"]);
    this.memories = new Set(["doc1", "doc2", "image1"]); // For compatibility with access-index.js
    this.galleries = new Set(["folder1"]); // For compatibility with access-index.js
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

function runSimpleObjectsDemo() {
  console.log("📁 Simple Objects Access Control Demo\n");

  // Create object manager
  const objectManager = new SimpleObjectManager();

  // Create access index
  const accessIndex = new AccessIndex();

  // Create some simple objects
  const doc1 = new ResKey(ResourceType.Memory, "doc1"); // Using Memory type for documents
  const doc2 = new ResKey(ResourceType.Memory, "doc2");
  const image1 = new ResKey(ResourceType.Memory, "image1");
  const folder1 = new ResKey(ResourceType.Folder, "folder1");

  console.log("📦 Simple Objects created:");
  console.log(`  - ${doc1.toString()}`);
  console.log(`  - ${doc2.toString()}`);
  console.log(`  - ${image1.toString()}`);
  console.log(`  - ${folder1.toString()}\n`);

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

  // Add access to objects
  accessIndex.addAccess(doc1, aliceAccess);
  accessIndex.addAccess(doc1, bobAccess);
  accessIndex.addAccess(doc1, charlieAccess);

  accessIndex.addAccess(doc2, aliceAccess);
  accessIndex.addAccess(doc2, bobAccess);

  accessIndex.addAccess(image1, aliceAccess);
  accessIndex.addAccess(image1, charlieAccess);

  accessIndex.addAccess(folder1, aliceAccess);
  accessIndex.addAccess(folder1, bobAccess);
  accessIndex.addAccess(folder1, charlieAccess);

  console.log("👥 Access entries added:");
  console.log(`  - Alice (owner): ${getPermissionNames(aliceAccess.perm_mask).join(", ")}`);
  console.log(`  - Bob (admin): ${getPermissionNames(bobAccess.perm_mask).join(", ")}`);
  console.log(`  - Charlie (member): ${getPermissionNames(charlieAccess.perm_mask).join(", ")}\n`);

  // Add public policy for doc2
  const publicPolicy = new PublicPolicy(
    PublicMode.PublicAuth,
    combinePermissions(Perm.VIEW, Perm.DOWNLOAD),
    Date.now(),
    Date.now()
  );
  accessIndex.setPublicPolicy(doc2, publicPolicy);

  console.log("🌐 Public policy added:");
  console.log(
    `  - ${doc2.toString()}: ${publicPolicy.mode} (${getPermissionNames(publicPolicy.perm_mask).join(", ")})\n`
  );

  // Test permission evaluation
  console.log("🔍 Permission Evaluation Tests:\n");

  const testCases = [
    { user: "alice", object: doc1, description: "Alice accessing her own document" },
    { user: "bob", object: doc1, description: "Bob accessing document as admin" },
    { user: "charlie", object: doc1, description: "Charlie accessing document as member" },
    { user: "dave", object: doc1, description: "Dave accessing document (no access)" },
    { user: "alice", object: doc2, description: "Alice accessing document with public policy" },
    { user: "dave", object: doc2, description: "Dave accessing public document" },
    { user: "alice", object: image1, description: "Alice accessing image" },
    { user: "charlie", object: image1, description: "Charlie accessing image" },
    { user: "bob", object: image1, description: "Bob accessing image (no access)" },
    { user: "alice", object: folder1, description: "Alice accessing folder" },
    { user: "charlie", object: folder1, description: "Charlie accessing folder" },
  ];

  for (const testCase of testCases) {
    const ctx = new PrincipalContext(testCase.user);
    const permissions = effectivePermMask(testCase.object, ctx, accessIndex, objectManager);
    const permissionNames = getPermissionNames(permissions);

    console.log(`  ${testCase.description}:`);
    console.log(`    User: ${testCase.user}`);
    console.log(`    Object: ${testCase.object.toString()}`);
    console.log(`    Permissions: ${permissionNames.length > 0 ? permissionNames.join(", ") : "NONE"}`);
    console.log(`    Can VIEW: ${hasPermission(permissions, Perm.VIEW) ? "✅" : "❌"}`);
    console.log(`    Can DOWNLOAD: ${hasPermission(permissions, Perm.DOWNLOAD) ? "✅" : "❌"}`);
    console.log(`    Can SHARE: ${hasPermission(permissions, Perm.SHARE) ? "✅" : "❌"}`);
    console.log(`    Can MANAGE: ${hasPermission(permissions, Perm.MANAGE) ? "✅" : "❌"}`);
    console.log(`    Can OWN: ${hasPermission(permissions, Perm.OWN) ? "✅" : "❌"}`);
    console.log("");
  }

  console.log("🎯 Key Benefits:");
  console.log("  ✅ Full access control system on simple objects");
  console.log("  ✅ Fine-grained permissions - VIEW, DOWNLOAD, SHARE, MANAGE, OWN");
  console.log("  ✅ Multiple access sources - direct grants, groups, public policies, magic links");
  console.log("  ✅ Efficient evaluation - ownership fast-path, bitwise operations");
  console.log("  ✅ Extensible - easy to add new object types and permission types");
}

// Export the demo function
export { runSimpleObjectsDemo };
