//! Integration tests for the capsule storage foundation
//!
//! These tests verify that both HashMap and Stable backends work correctly
//! through the Store enum, ensuring runtime polymorphism works as expected.

// Integration tests use the crate modules directly
use crate::capsule_store::{CapsuleId, CapsuleStore, Order, Store};
use crate::types::{Capsule, OwnerState, PersonRef};
use candid::Principal;
use std::collections::HashMap;

#[test]
fn test_store_enum_delegation() {
    // Hash backend
    let mut hash_store = Store::new_hash();
    test_store_operations(&mut hash_store, "HashMap");

    // Stable backend (works off-chain with DefaultMemoryImpl)
    let mut stable_store = Store::new_stable();
    test_store_operations(&mut stable_store, "StableBTreeMap");
}

#[test]
fn test_store_backend_identification() {
    let hash_store = Store::new_hash();
    assert_eq!(hash_store.backend_type(), "HashMap");

    match &hash_store {
        Store::Hash(_) => {}
        Store::Stable(_) => panic!("Should be HashMap backend"),
    }
}

#[test]
fn test_store_api_completeness() {
    let mut store = Store::new_hash();

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
    // Run against both backends
    for mut store in [Store::new_hash(), Store::new_stable()] {
        // Insert capsule A
        let id: CapsuleId = "cap-1".into();
        let mut cap = create_test_capsule(id.clone());
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
}

#[test]
fn test_pagination_cursor_semantics() {
    for mut store in [Store::new_hash(), Store::new_stable()] {
        // Insert 5 items with deterministic ids
        for i in 0..5 {
            let id = format!("cap-{i}");
            let cap = create_test_capsule(id.clone());
            store.put_if_absent(id, cap).unwrap();
        }

        // Page size 2, Asc order
        let p1 = store.paginate(None, 2, Order::Asc);
        assert_eq!(p1.items.len(), 2);
        let c1 = p1.next_cursor.clone().expect("cursor after page 1");

        // Page 2 starts strictly after c1 (exclusive)
        let p2 = store.paginate(p1.next_cursor, 2, Order::Asc);
        assert_eq!(p2.items.len(), 2);
        let c2 = p2.next_cursor.clone().expect("cursor after page 2");

        // Page 3 (last, possibly shorter)
        let p3 = store.paginate(Some(c2), 2, Order::Asc);
        assert_eq!(p3.items.len(), 1);
        assert!(p3.next_cursor.is_none(), "no cursor at end");
    }
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
        created_at: 1234567890,
        updated_at: 1234567890,
        bound_to_neon: false,
    }
}
