# Migrate users.rs to Capsule Architecture

## Overview

We are migrating from a separate `users.rs` module to a pure capsule-based architecture where all user functionality is handled through capsules (subject/owner relationships).

## Migration Checklist

### ✅ Completed Functions

- [x] ~~`register_user()` → `register_capsule()` (self-registration)~~ ✅ DONE
- [x] ~~`mark_user_bound()` → `mark_capsule_bound_to_web2()` (Web2 binding)~~ ✅ DONE
- [x] ~~`list_all_users()` → Updated to use admin system~~ ✅ DONE

### 🔄 Functions to Migrate

#### Storage Migration (Priority 1)

- [x] ~~Replace `USERS` HashMap with `CAPSULES` HashMap in memory.rs~~ ✅ DONE
- [x] ~~Move capsule storage from capsule.rs to memory.rs (centralize storage)~~ ✅ DONE
- [x] ~~Update all functions to use centralized CAPSULES storage in memory.rs~~ ✅ DONE

#### User Registration & Management

- [x] ~~`get_user()` → Get caller's self-capsule~~ ✅ DONE (implemented as `get_capsule_info()`)
- [ ] `get_user_by_principal(principal)` → Get capsule where principal is owner
- [ ] `update_user_activity()` → Update capsule/owner activity timestamps

#### User Statistics & Analytics

- [ ] `get_user_stats()` → Capsule-based statistics (total capsules, bound capsules, etc.)

#### Admin System (Moved to admin.rs)

- [x] ~~`is_admin(principal)` → Keep as is (canister-level admin)~~ ✅ DONE (moved to admin.rs)
- [x] ~~`add_admin(principal)` → Keep as is (canister-level admin)~~ ✅ DONE (moved to admin.rs)
- [x] ~~`remove_admin(principal)` → Keep as is (canister-level admin)~~ ✅ DONE (moved to admin.rs)
- [x] ~~`list_admins()` → Keep as is (canister-level admin)~~ ✅ DONE (moved to admin.rs)

#### Persistence Functions

- [x] ~~`export_users_for_upgrade()` → `export_capsules_for_upgrade()` (already exists)~~ ✅ DONE
- [x] ~~`import_users_from_upgrade()` → `import_capsules_from_upgrade()` (already exists)~~ ✅ DONE
- [x] ~~`export_admins_for_upgrade()` → Keep as is~~ ✅ DONE (moved to admin.rs)
- [x] ~~`import_admins_from_upgrade()` → Keep as is~~ ✅ DONE (moved to admin.rs)

### 🗂️ Files to Update

#### Backend Files

- [ ] `src/backend/src/users.rs` → Remove user-specific functions, keep admin system
- [ ] `src/backend/src/lib.rs` → Update exposed functions
- [ ] `src/backend/src/types.rs` → Remove User struct, keep admin types
- [ ] `src/backend/backend.did` → Regenerate after changes

#### Frontend Files (Future)

- [ ] Update frontend to use capsule-based endpoints
- [ ] Remove references to old user endpoints

### 🎯 Migration Goals

1. **Single Source of Truth**: Everything user-related goes through capsules
2. **Consistent Architecture**: No separate user tracking outside capsules
3. **Future-Proof**: Works for self-capsules and memorial capsules
4. **Clean Separation**: Admin system stays separate (canister-level concern)

### 📋 Notes

- Admin system remains in `users.rs` as it's canister-level functionality
- User statistics should be calculated from capsule data
- Activity tracking moves to capsule/owner level
- Web2 binding is already capsule-based

### 🚀 Next Steps

1. Migrate `get_user()` functions to capsule-based approach
2. Update user statistics to use capsule data
3. Remove User struct and related types
4. Update lib.rs exposed functions
5. Test all functionality works with capsule architecture
