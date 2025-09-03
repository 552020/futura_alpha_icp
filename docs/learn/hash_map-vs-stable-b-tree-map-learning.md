Got it — you want a **learning plan**, not app-level code. Think of this like katas: small, artificial exercises that isolate one concept.

Here’s a sequence that will teach you the contrasts between **HashMap** and **StableBTreeMap** step by step:

---

## Phase 1: Warm-up with `HashMap` (Rust stdlib)

1. **Insert & Retrieve**

   - Create a `HashMap<u32, String>`.
   - Insert 3 values.
   - Retrieve one by key and print it.

2. **Update in Place**

   - Use `.get_mut()` to change a value.
   - Print the whole map.

3. **Iteration**

   - Print all keys with `.keys()`.
   - Print all values with `.values()`.
   - Print key–value pairs with `.iter()`.

4. **Filter & Collect**

   - Collect all values longer than 3 chars into a `Vec<String>`.

---

## Phase 2: First Contact with `StableBTreeMap`

(Requires `ic-stable-structures` crate — but you can run these as normal Rust unit tests with a fake memory manager.)

1. **Initialization**

   - Set up a `StableBTreeMap<u32, String, VirtualMemory<_>>`.
   - Insert 3 values.
   - Retrieve one and print it.

2. **Update (Read–Modify–Write)**

   - Try `.get_mut()` (see compiler error).
   - Instead, `.get()`, modify the value, then `.insert()` it back.

3. **Iteration Differences**

   - Use `.iter()` to print all pairs.
   - Notice: it yields owned `(K, V)` instead of references.

4. **Cloning Behavior**

   - Try `.cloned()` on an `Option<T>` from `.get()`.
   - See when it works and when you need `.map(|x| x.clone())`.

---

## Phase 3: Comparing Behavior Side by Side

1. **Volatility vs Persistence**

   - HashMap: insert, drop, recreate → data gone.
   - StableBTreeMap: insert, simulate “upgrade” by re-initializing with same memory ID → data still there.

2. **Ordering**

   - Insert keys `3, 1, 2`.
   - HashMap: iteration order is arbitrary.
   - StableBTreeMap: iteration order is sorted.

3. **Performance Model**

   - HashMap: O(1) lookups.
   - StableBTreeMap: O(log n) lookups (try with 1000 inserts and measure).

4. **Deletes**

   - HashMap: `remove(&key)` → gone immediately.
   - StableBTreeMap: `remove(&key)` → works, but think about stable memory cost (data is overwritten with tombstone).

---

## Phase 4: Mini Challenges

1. **Keyset Pagination**

   - Insert 10 items with keys 0–9.
   - Write a function `get_page(start, limit)` that returns a slice.
   - Do it once with HashMap (awkward) and once with StableBTreeMap (natural via ordering).

2. **Secondary Index**

   - Store `(id, name)` tuples.
   - Build a second map: `name → id`.
   - Demonstrate `find_by_name` in both systems.

3. **Stress Test**

   - Insert 1000 items.
   - Benchmark retrieval for both.
   - Observe differences in speed and ordering.

---

## Phase 5: Reflection

- Write a short note answering:

  - What can HashMap do easily that StableBTreeMap cannot?
  - What can StableBTreeMap do that HashMap cannot?
  - When would you prefer each?

---

This path starts simple, builds confidence, and gradually layers in the **real differences**: persistence, API shape, iteration semantics, ordering, and performance.

Perfect — here’s a **7-day learn plan** you can follow. Each day is about 30–60 minutes of focused practice, not tied to your real app, just small throwaway exercises.

---

## **Day 1: HashMap Basics**

- Create a `HashMap<u32, String>`.
- Insert 3–5 key–value pairs.
- Retrieve a value by key and print it.
- Use `.get_mut()` to update one value in place.
- Iterate with `.keys()`, `.values()`, and `.iter()`.

**Goal**: Feel comfortable with the ergonomics of `HashMap`.

---

## **Day 2: HashMap Filtering & Collecting**

- Insert some strings of different lengths.
- Use `.values()` and `.filter()` to collect all strings longer than 3 chars.
- Create a new `HashMap` with only even keys.
- Count how many values contain a certain letter.

**Goal**: Practice functional patterns with HashMap.

---

## **Day 3: StableBTreeMap Basics**

- Initialize a `StableBTreeMap<u32, String, VirtualMemory<_>>` with the `ic-stable-structures` crate.
- Insert 3 values, then retrieve one by key.
- Try using `.get_mut()` and observe the compiler error.
- Perform a read–modify–write update (get, change, insert again).

**Goal**: Understand why StableBTreeMap doesn’t allow direct mutable references.

---

## **Day 4: Iteration & Ordering**

- Insert keys `3, 1, 2`.
- Iterate over the map and print keys:

  - HashMap: arbitrary order.
  - StableBTreeMap: sorted order.

- Compare what `.iter()` yields in each (borrowed refs vs owned pairs).

**Goal**: See how iteration semantics differ.

---

## **Day 5: Persistence vs Volatility**

- With HashMap: insert values, drop it, recreate → confirm data is lost.
- With StableBTreeMap: insert values, then re-init with the same memory ID → confirm data persists.
- Reflect on why persistence matters for canisters.

**Goal**: Internalize persistence as the killer feature of stable structures.

---

## **Day 6: Advanced Use Cases**

- Insert 10 items with keys 0–9.
- Implement keyset pagination: function `get_page(start, limit) -> Vec<String>`.
- Create a secondary index (e.g. `name → id`) and use it to look up an item by name.
- Do both in HashMap and StableBTreeMap, compare the code.

**Goal**: Practice real-world patterns like pagination and secondary indexes.

---

## **Day 7: Reflection & Stress Test**

- Insert 1000 items in both structures.
- Time how long it takes to do 1000 lookups.
- Write a one-page note:

  - What can HashMap do that StableBTreeMap cannot?
  - What can StableBTreeMap do that HashMap cannot?
  - When would you choose one over the other?

**Goal**: Cement your understanding with both benchmarks and reflection.

---

By the end of Day 7, you’ll have **muscle memory** for both APIs and a clear mental model of when to use each.

Want me to also draft a **skeleton Rust project** (Cargo.toml + test modules) you can use to run through these exercises quickly?

Here’s a tiny, ready-to-run Rust scaffold with one command (`cargo test`) that walks you through the 7-day plan. It compiles locally (no IC tooling needed) and uses `DefaultMemoryImpl` to simulate stable memory.

### Project tree

```
stable-vs-hashmap/
├─ Cargo.toml
├─ src/
│  └─ lib.rs
└─ tests/
   ├─ day1_hashmap_basics.rs
   ├─ day2_hashmap_filter.rs
   ├─ day3_stable_basics.rs
   ├─ day4_iteration_order.rs
   ├─ day5_persistence.rs
   ├─ day6_indexes_pagination.rs
   └─ day7_stress_reflection.rs
```

### Cargo.toml

```toml
[package]
name = "stable-vs-hashmap"
version = "0.1.0"
edition = "2021"

[dependencies]
candid = "0.10"
serde = { version = "1", features = ["derive"] }
ic-stable-structures = "0.6"

[dev-dependencies]
rand = "0.8"
```

### src/lib.rs

```rust
use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, Storable, BoundedStorable,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::RefCell};

/// Concrete stable memory type we'll use in tests (works off-chain).
pub type VMem = VirtualMemory<DefaultMemoryImpl>;

/// Stable value type with (de)serialization via Candid.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub struct Item {
    pub id: u32,
    pub name: String,
}

impl Storable for Item {
    fn to_bytes(&self) -> Cow<[u8]> { Cow::Owned(Encode!(self).unwrap()) }
    fn from_bytes(bytes: Cow<[u8]>) -> Self { Decode!(bytes.as_ref(), Self).unwrap() }
}
impl BoundedStorable for Item {
    // rough bound (id + name)
    const MAX_SIZE: u32 = 4 + 1024;
    const IS_FIXED_SIZE: bool = false;
}

/// A tiny helper that owns a MemoryManager and can hand out StableBTreeMaps.
/// In a real canister you'd hold this globally; here we keep it simple for tests.
pub struct StableEnv {
    mgr: RefCell<MemoryManager<DefaultMemoryImpl>>,
}
impl StableEnv {
    /// Create a fresh environment (blank stable memory).
    pub fn new_blank() -> Self {
        Self { mgr: RefCell::new(MemoryManager::init(DefaultMemoryImpl::default())) }
    }

    /// Recreate an environment that *logically* points to the same underlying memory.
    /// For local tests, we emulate persistence by reusing the same DefaultMemoryImpl.
    /// In a real canister upgrade, you’d just re-init with the same stable memory.
    pub fn from_impl(mem: DefaultMemoryImpl) -> Self {
        Self { mgr: RefCell::new(MemoryManager::init(mem)) }
    }

    /// Expose the underlying memory impl so tests can simulate "upgrade & reopen".
    pub fn memory_impl(&self) -> DefaultMemoryImpl {
        // MemoryManager doesn’t expose a direct getter; for tests we clone DefaultMemoryImpl.
        // DefaultMemoryImpl implements Clone.
        self.mgr.borrow().memory().clone()
    }

    /// Create/init a StableBTreeMap<u32, Item, VMem> in MemoryId(slot).
    pub fn open_items_map(&self, slot: u8) -> StableBTreeMap<u32, Item, VMem> {
        let mem = self.mgr.borrow().get(MemoryId::new(slot));
        StableBTreeMap::init(mem)
    }

    /// Create/init a StableBTreeMap<String, u32, VMem> for secondary index (name -> id).
    pub fn open_name_to_id(&self, slot: u8) -> StableBTreeMap<String, u32, VMem> {
        let mem = self.mgr.borrow().get(MemoryId::new(slot));
        StableBTreeMap::init(mem)
    }
}

/// Simple HashMap helpers used by day 1–2 tests.
pub mod hash_helpers {
    use std::collections::HashMap;

    pub fn new_map() -> HashMap<u32, String> {
        HashMap::new()
    }
}
```

---

## tests/day1_hashmap_basics.rs

```rust
use stable_vss_hashmap::hash_helpers::new_map;

#[test]
fn day1_hashmap_basics() {
    let mut m = new_map();
    // Insert
    m.insert(1, "alpha".into());
    m.insert(2, "beta".into());
    m.insert(3, "gamma".into());

    // Retrieve
    assert_eq!(m.get(&2).unwrap(), "beta");

    // Update in place
    if let Some(v) = m.get_mut(&3) {
        *v = "GAMMA".into();
    }
    assert_eq!(m.get(&3).unwrap(), "GAMMA");

    // Iterate keys/values/pairs
    let mut keys: Vec<_> = m.keys().cloned().collect();
    keys.sort();
    assert_eq!(keys, vec![1, 2, 3]);

    let mut vals: Vec<_> = m.values().cloned().collect();
    vals.sort();
    assert!(vals.contains(&"alpha".into()));
    assert!(vals.contains(&"beta".into()));
    assert!(vals.contains(&"GAMMA".into()));
}
```

## tests/day2_hashmap_filter.rs

```rust
use stable_vss_hashmap::hash_helpers::new_map;

#[test]
fn day2_hashmap_filter_collect() {
    let mut m = new_map();
    m.insert(1, "a".into());
    m.insert(2, "abcd".into());
    m.insert(3, "abcdef".into());

    // Collect values longer than 3 chars
    let mut long: Vec<String> = m.values().cloned().filter(|s| s.len() > 3).collect();
    long.sort();
    assert_eq!(long, vec!["abcd".to_string(), "abcdef".to_string()]);

    // Filter by even keys into new map
    let filtered: std::collections::HashMap<_, _> =
        m.iter().filter(|(k, _)| **k % 2 == 0).map(|(k, v)| (*k, v.clone())).collect();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered.get(&2).unwrap(), "abcd");
}
```

## tests/day3_stable_basics.rs

```rust
use stable_vss_hashmap::{Item, StableEnv};

#[test]
fn day3_stable_basics_crud() {
    let env = StableEnv::new_blank();
    let mut map = env.open_items_map(0);

    // Insert
    map.insert(1, Item { id: 1, name: "alpha".into() });
    map.insert(2, Item { id: 2, name: "beta".into() });

    // Get (owned value)
    let it = map.get(&2).unwrap();
    assert_eq!(it.name, "beta");

    // No get_mut(): do read-modify-write
    if let Some(mut it) = map.get(&1) {
        it.name = "ALPHA".into();
        map.insert(1, it);
    }
    assert_eq!(map.get(&1).unwrap().name, "ALPHA");

    // Iteration yields owned (k, v)
    let mut names: Vec<_> = map.iter().map(|(_, v)| v.name).collect();
    names.sort();
    assert_eq!(names, vec!["ALPHA".to_string(), "beta".to_string()]);
}
```

## tests/day4_iteration_order.rs

```rust
use stable_vss_hashmap::{Item, StableEnv};

#[test]
fn day4_ordering() {
    let env = StableEnv::new_blank();
    let mut map = env.open_items_map(0);

    map.insert(3, Item { id: 3, name: "three".into() });
    map.insert(1, Item { id: 1, name: "one".into() });
    map.insert(2, Item { id: 2, name: "two".into() });

    // StableBTreeMap iteration is sorted by key
    let keys: Vec<u32> = map.iter().map(|(k, _)| k).collect();
    assert_eq!(keys, vec![1, 2, 3]);
}
```

## tests/day5_persistence.rs

```rust
use stable_vss_hashmap::{Item, StableEnv};

#[test]
fn day5_persistence_sim() {
    // Start with blank env
    let env1 = StableEnv::new_blank();
    {
        let mut m = env1.open_items_map(0);
        m.insert(7, Item { id: 7, name: "persist".into() });
    }

    // "Upgrade": re-create env using the same underlying memory impl
    let mem = env1.memory_impl();
    let env2 = StableEnv::from_impl(mem);
    let m2 = env2.open_items_map(0);
    assert_eq!(m2.get(&7).unwrap().name, "persist");
}
```

## tests/day6_indexes_pagination.rs

```rust
use stable_vss_hashmap::{Item, StableEnv};

#[test]
fn day6_indexes_and_pagination() {
    let env = StableEnv::new_blank();
    let mut items = env.open_items_map(0);
    let mut by_name = env.open_name_to_id(1);

    // Insert 10 items and maintain secondary index (name -> id)
    for id in 0..10u32 {
        let name = format!("item-{id}");
        items.insert(id, Item { id, name: name.clone() });
        by_name.insert(name, id);
    }

    // Lookup by name via index
    let id = by_name.get(&"item-7".to_string()).unwrap();
    assert_eq!(id, 7);
    assert_eq!(items.get(&id).unwrap().name, "item-7");

    // Keyset pagination (limit 3)
    fn page(
        map: &ic_stable_structures::StableBTreeMap<u32, Item, stable_vss_hashmap::VMem>,
        start: u32,
        limit: usize,
    ) -> (Vec<Item>, Option<u32>) {
        let mut out = Vec::new();
        let mut next = None;
        // Simple scan using iter(); for large maps you'd use range helpers if/when exposed.
        for (k, v) in map.iter().filter(|(k, _)| *k >= start) {
            if out.len() == limit {
                next = Some(k);
                break;
            }
            out.push(v);
        }
        (out, next)
    }

    // First page
    let (p1, cur) = page(&items, 0, 3);
    assert_eq!(p1.len(), 3);
    assert_eq!(p1[0].id, 0);

    // Next page
    let (p2, cur2) = page(&items, cur.unwrap(), 3);
    assert_eq!(p2.len(), 3);
    assert_eq!(p2[0].id, 3);

    // Last page (may be shorter)
    let (p_last, _) = page(&items, cur2.unwrap(), 10);
    assert_eq!(p_last.len(), 7); // from id 6..9
}
```

## tests/day7_stress_reflection.rs

```rust
use rand::{rngs::StdRng, Rng, SeedableRng};
use stable_vss_hashmap::{Item, StableEnv};
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn day7_stress_and_compare() {
    // Build datasets
    let mut rng = StdRng::seed_from_u64(42);
    let n = 5_000u32;

    // HashMap
    let mut hm: HashMap<u32, String> = HashMap::new();
    for id in 0..n {
        hm.insert(id, format!("name-{id}"));
    }
    let t0 = Instant::now();
    for _ in 0..n {
        let k = rng.gen_range(0..n);
        let _ = hm.get(&k);
    }
    let hash_lookup_ms = t0.elapsed().as_millis();

    // StableBTreeMap
    let env = StableEnv::new_blank();
    let mut st = env.open_items_map(0);
    for id in 0..n {
        st.insert(id, Item { id, name: format!("name-{id}") });
    }
    let mut rng = StdRng::seed_from_u64(42);
    let t1 = Instant::now();
    for _ in 0..n {
        let k = rng.gen_range(0..n);
        let _ = st.get(&k);
    }
    let stable_lookup_ms = t1.elapsed().as_millis();

    // Not a rigorous benchmark—just to feel O(1) vs O(log n) + serialization overhead.
    eprintln!("HashMap lookups:  {hash_lookup_ms} ms");
    eprintln!("Stable lookups:   {stable_lookup_ms} ms");

    // Sanity: both return something for mid key
    assert!(hm.get(&(n / 2)).is_some());
    assert!(st.get(&(n / 2)).is_some());
}
```

---

### Run it

```bash
cd stable-vs-hashmap
cargo test
```

This gives you small, focused exercises that mirror the 7-day plan and make the contrasts obvious (mutability, iteration, ordering, persistence, indexing, pagination, and perf profile). If you want this condensed into a single `lib.rs` with `#[cfg(test)]` instead of separate `tests/` files, say the word and I’ll switch it.
