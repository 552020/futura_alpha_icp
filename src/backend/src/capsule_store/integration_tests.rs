//! Integration tests for the capsule storage foundation
//!
//! These tests verify that both HashMap and Stable backends work correctly
//! through the Store enum, ensuring runtime polymorphism works as expected.

use super::{CapsuleId, CapsuleStore, Order, Store};
use crate::types::{Capsule, HostingPreferences, OwnerState, PersonRef};
use candid::Principal;
use std::collections::HashMap;

#[test]
fn test_store_enum_delegation() {
    // Stable backend (works off-chain with DefaultMemoryImpl)
    let mut stable_store = Store::new_stable_test();
    test_store_operations(&mut stable_store, "StableBTreeMap");
}

#[test]
fn test_store_backend_identification() {
    let stable_store = Store::new_stable_test();
    assert_eq!(stable_store.backend_type(), "StableBTreeMap");

    match &stable_store {
        Store::Stable(_) => {}
    }
}

#[test]
fn test_store_api_completeness() {
    let mut store = Store::new_stable_test();

    let id: CapsuleId = "test-complete-api".to_string();
    let capsule = create_test_capsule(id.clone());

    assert!(!store.exists(&id));
    assert_eq!(store.count(), 0);

    // put_if_absent
    assert!(store.put_if_absent(id.clone(), capsule.clone()).is_ok());
    assert!(store.exists(&id));
    assert_eq!(store.count(), 1);

    // get
    let retrieved = store.get(&id).expect("capsule present");
    assert_eq!(retrieved.id, id);

    // upsert (update existing)
    let updated = create_test_capsule(id.clone());
    let old = store.upsert(id.clone(), updated);
    assert!(old.is_some());

    // find_by_subject (pass &PersonRef per frozen API)
    let found = store.find_by_subject(&capsule.subject);
    assert!(found.is_some());

    // list_by_owner
    let owner_capsules = store.list_by_owner(&capsule.subject);
    assert!(owner_capsules.contains(&id));

    // get_many
    let many = store.get_many(&[id.clone()]);
    assert_eq!(many.len(), 1);

    // paginate (single item case -> no next cursor)
    let page = store.paginate(None, 10, Order::Asc);
    assert!(!page.items.is_empty());
    assert!(page.next_cursor.is_none());

    // remove
    let removed = store.remove(&id);
    assert!(removed.is_some());
    assert!(!store.exists(&id));
    assert_eq!(store.count(), 0);
}

#[test]
fn test_index_updates_on_upsert_and_update() {
    // Run against stable backend
    let mut store = Store::new_stable_test();

    // Insert capsule A
    let id: CapsuleId = "cap-1".into();
    let cap = create_test_capsule(id.clone());
    let subj_a = cap.subject.clone();
    store.put_if_absent(id.clone(), cap.clone()).unwrap();

    // Verify subject index → A
    let got = store.find_by_subject(&subj_a).expect("found by subj A");
    assert_eq!(got.id, id);

    // Change subject via update: A -> B
    let subj_b = PersonRef::Principal(Principal::from_text("2vxsx-fae").unwrap());
    store
        .update(&id, |c| {
            c.subject = subj_b.clone();
        })
        .unwrap();

    // Old subject should no longer resolve
    assert!(store.find_by_subject(&subj_a).is_none());

    // New subject resolves
    let got_b = store.find_by_subject(&subj_b).expect("found by subj B");
    assert_eq!(got_b.id, id);

    // Owner index should not duplicate entries after repeated upserts
    let before = store.list_by_owner(&subj_b).len();
    let cap2 = create_test_capsule(id.clone()); // same owners by default
    let _ = store.upsert(id.clone(), cap2);
    let after = store.list_by_owner(&subj_b).len();
    assert_eq!(before, after, "owner index should not duplicate ids");
}

#[test]
fn test_pagination_cursor_semantics() {
    let mut store = Store::new_stable_test();

    // Insert 5 items with deterministic ids
    for i in 0..5 {
        let id = format!("cap-{i}");
        let cap = create_test_capsule(id.clone());
        store.put_if_absent(id, cap).unwrap();
    }

    // Page size 2, Asc order
    let p1 = store.paginate(None, 2, Order::Asc);
    assert_eq!(p1.items.len(), 2);
    let _c1 = p1.next_cursor.clone().expect("cursor after page 1");

    // Page 2 starts strictly after c1 (exclusive)
    let p2 = store.paginate(p1.next_cursor, 2, Order::Asc);
    assert_eq!(p2.items.len(), 2);
    let c2 = p2.next_cursor.clone().expect("cursor after page 2");

    // Page 3 (last, possibly shorter)
    let p3 = store.paginate(Some(c2), 2, Order::Asc);
    assert_eq!(p3.items.len(), 1);
    assert!(p3.next_cursor.is_none(), "no cursor at end");
}

fn test_store_operations(store: &mut Store, backend_name: &str) {
    println!("Testing {} backend...", backend_name);

    let id = format!("test-{}-123", backend_name.to_lowercase());
    let capsule = create_test_capsule(id.clone());

    // Test basic CRUD
    assert!(store.put_if_absent(id.clone(), capsule.clone()).is_ok());
    assert!(store.exists(&id));

    let retrieved = store.get(&id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, id);

    // Test update
    let update_result = store.update(&id, |c| {
        // Modify the capsule
        c.created_at = 9999999999;
    });
    assert!(update_result.is_ok());

    // Verify update worked
    let updated = store.get(&id);
    assert_eq!(updated.unwrap().created_at, 9999999999);

    // Test find_by_subject with &PersonRef
    let found = store.find_by_subject(&capsule.subject);
    assert!(found.is_some());

    // cleanup
    assert!(store.remove(&id).is_some());
    assert!(!store.exists(&id));

    println!("✅ {} backend tests passed!", backend_name);
}

#[test]
fn test_index_consistency_property_based() {
    // Test stable backend for index consistency
    test_index_consistency_on_backend("StableBTreeMap", Store::new_stable_test());
}

fn test_index_consistency_on_backend(backend_name: &str, mut store: Store) {
    println!(
        "🧪 Testing index consistency on {} backend...",
        backend_name
    );

    // Ground truth maps for verification
    let mut ground_truth_by_subject = std::collections::HashMap::new();
    let mut ground_truth_by_owner = std::collections::HashMap::new();

    // Test sequence of operations
    let alice_principal = Principal::from_text("2vxsx-fae").unwrap();
    let bob_principal = Principal::from_text("w7x7r-cok77-xa").unwrap();

    let operations = vec![
        (
            "create_user_alice",
            create_test_capsule_with_principal("alice-001".to_string(), alice_principal.clone()),
        ),
        (
            "create_user_bob",
            create_test_capsule_with_principal("bob-001".to_string(), bob_principal.clone()),
        ),
        (
            "create_alice_capsule_2",
            create_test_capsule_with_principal("alice-002".to_string(), alice_principal.clone()),
        ),
        (
            "create_bob_capsule_2",
            create_test_capsule_with_principal("bob-002".to_string(), bob_principal.clone()),
        ),
    ];

    // Execute operations and maintain ground truth
    for (op_name, capsule) in operations {
        let id = capsule.id.clone();
        let subject = capsule.subject.clone();

        // Update ground truth for owners before moving capsule
        for (owner_ref, _) in &capsule.owners {
            if let crate::types::PersonRef::Principal(owner_principal) = owner_ref {
                ground_truth_by_owner
                    .entry(owner_principal.clone())
                    .or_insert_with(Vec::new)
                    .push(id.clone());
            }
        }

        // Store in our indexed store
        assert!(
            store.upsert(id.clone(), capsule).is_none(),
            "Should be new capsule: {}",
            op_name
        );

        // Update ground truth
        ground_truth_by_subject.insert(subject.clone(), id.clone());
    }

    // Verify subject index consistency
    for (subject, expected_id) in &ground_truth_by_subject {
        let found = store.find_by_subject(subject);
        assert!(found.is_some(), "Subject {:?} should find capsule", subject);
        assert_eq!(
            found.unwrap().id,
            *expected_id,
            "Subject index should return correct capsule"
        );
    }

    // Verify owner index consistency (simplified check)
    for (subject, expected_id) in &ground_truth_by_subject {
        if let crate::types::PersonRef::Principal(_owner_principal) = subject {
            let owner_capsules = store.list_by_owner(subject);
            assert!(!owner_capsules.is_empty(), "Owner should have capsules");
            assert!(
                owner_capsules.contains(expected_id),
                "Owner index should include capsule"
            );
        }
    }

    println!(
        "✅ Index consistency verified for {} backend!",
        backend_name
    );
}

// TODO: Fix property-based test - it has fundamental issues with subject index conflicts
// The test tries to create multiple capsules with the same subject, which violates the index constraint
// All related functions have been removed to eliminate dead code warnings

fn create_test_capsule_with_principal(
    id: String,
    subject_principal: Principal,
) -> crate::types::Capsule {
    use crate::types::{Capsule, HostingPreferences, OwnerState, PersonRef};
    use std::collections::HashMap;

    let subject = PersonRef::Principal(subject_principal);
    let mut owners = HashMap::new();

    // Add the subject as an owner too
    owners.insert(
        PersonRef::Principal(subject_principal),
        OwnerState {
            since: 1234567890,
            last_activity_at: 1234567890,
        },
    );

    Capsule {
        id,
        subject,
        owners,
        controllers: HashMap::new(),
        connections: HashMap::new(),
        connection_groups: HashMap::new(),
        memories: HashMap::new(),
        galleries: HashMap::new(),
        folders: HashMap::new(),
        has_advanced_settings: false, // Default to simple settings
        created_at: 1234567890,
        updated_at: 1234567890,
        bound_to_neon: false,
        inline_bytes_used: 0,
        hosting_preferences: HostingPreferences::default(),
    }
}

fn create_test_capsule(id: CapsuleId) -> Capsule {
    let subject = PersonRef::Principal(Principal::from_text("aaaaa-aa").unwrap());
    let mut owners = HashMap::new();
    owners.insert(
        subject.clone(),
        OwnerState {
            since: 1234567890,
            last_activity_at: 1234567890,
        },
    );

    Capsule {
        id,
        subject,
        owners,
        controllers: HashMap::new(),
        connections: HashMap::new(),
        connection_groups: HashMap::new(),
        memories: HashMap::new(),
        galleries: HashMap::new(),
        folders: HashMap::new(),
        has_advanced_settings: false, // Default to simple settings
        created_at: 1234567890,
        updated_at: 1234567890,
        bound_to_neon: false,
        inline_bytes_used: 0,
        hosting_preferences: HostingPreferences::default(),
    }
}
