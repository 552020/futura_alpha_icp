# HashMap-vs-StableBTreeMap

- HashMap (std): in-memory, process-local, volatile. Average O(1) lookups. Loses data on canister upgrade/restart.
- StableBTreeMap (ic-stable-structures): key–value map backed by stable memory. Persists across upgrades. Operations are O(log n). Requires (de)serialization.

# Core differences (you’ll feel these in code)

1. Persistence

- HashMap: RAM only.
- StableBTreeMap: lives in stable memory; survives upgrades.

2. API surface

- HashMap: `get`, `insert`, `remove`, `get_mut`, `values`, `values_mut`, `iter`, etc.
- StableBTreeMap: `get`, `insert`, `remove`, `contains_key`, `len`, `is_empty`, `iter` (yields owned pairs), no `get_mut`, no `values`/`values_mut`.

3. Mutating in place

- HashMap: `if let Some(c) = map.get_mut(&id) { /* mutate */ }`
- StableBTreeMap: no mutable references to stored values. Do read-modify-write:

  ```rust
  if let Some(mut c) = map.get(&id) {
      // mutate c
      map.insert(id, c);
  }
  ```

4. Iteration & ownership

- HashMap: `for v in map.values()` yields `&V` (borrowed), can use `.cloned()`.
- StableBTreeMap: `for (k, v) in map.iter()` yields owned `(K, V)`; you already have `V` by value, so no `.cloned()`.

5. Ordering

- HashMap: arbitrary order.
- StableBTreeMap: ordered by key (B-tree). Enables keyset pagination and range scans.

6. Types & traits

- HashMap: any `K: Eq + Hash`.
- StableBTreeMap: `K: Ord + Storable`, `V: Storable` (often also `BoundedStorable`). You must define how to serialize keys/values to bytes (e.g., via Candid or bincode).

7. Memory plumbing

- HashMap: no extra types.
- StableBTreeMap: needs a concrete memory type as the 3rd generic (e.g., `VirtualMemory<DefaultMemoryImpl>`) and a `MemoryId`. You usually keep a `MemoryManager` and init each map with a distinct `MemoryId`.

8. Performance profile

- HashMap: very fast, O(1) avg.
- StableBTreeMap: O(log n), plus serialization overhead. Still fine for typical canister loads; avoid unnecessary full scans.

9. Costs/limits

- HashMap: free, but volatile and limited by heap.
- StableBTreeMap: uses stable memory (larger), persists; certain operations indirectly affect cycles when combined with upgrade hooks and state size. Be mindful of value sizes (BoundedStorable::MAX_SIZE).

10. Upgrades & schema evolution

- HashMap: nothing to preserve.
- StableBTreeMap: you must keep serialization compatible or write migrations.

# Common pattern mappings

Find one by predicate:

```rust
// HashMap
let x = map.values().find(|c| c.subject == subj).cloned();

// StableBTreeMap (scan)
let x = map.iter().find_map(|(_, c)| (c.subject == subj).then_some(c));
// Better: maintain a secondary index: subject -> id, then `idx.get(&subj).and_then(|id| map.get(&id))`
```

Update a value:

```rust
// HashMap
if let Some(c) = map.get_mut(&id) { c.title = new_title; }

// StableBTreeMap
if let Some(mut c) = map.get(&id) { c.title = new_title; map.insert(id, c); }
```

List + paginate (keyset):

```rust
// StableBTreeMap (ordered)
let (items, next) = {
    let mut out = Vec::new();
    let mut next = None;
    let start = cursor.unwrap_or(u64::MIN);
    for (k, v) in map.range(start..) {        // if you expose a range helper
        if out.len() == limit as usize { next = Some(k); break; }
        out.push(v);
    }
    (out, next)
};
```

# When to pick which

- Use HashMap:

  - Unit tests and ephemeral local state.
  - Caching derived data you can recompute.

- Use StableBTreeMap:

  - Anything you must keep across upgrades.
  - Data you want ordered scans/pagination for.
  - When you can’t risk RAM-only loss.

# Practical guidance for your codebase

- Don’t call StableBTreeMap directly from endpoints. Wrap it in a tiny repo API:

  - `get`, `put`, `remove`, `exists`
  - `update(id, |c| …)` (does read-modify-write internally)
  - Query helpers: `find_by_subject`, `list_by_owner`, `paginate`
  - Maintain secondary indexes inside `put/remove/update`.

- Keep `Principal` types consistent (use `candid::Principal` or alias).
- Implement `Storable`/`BoundedStorable` for your value types via Candid encode/decode.

If you want, I can sketch your `Capsule`’s `Storable` impl and the minimal StableBTreeMap wiring for your project’s memory manager with the secondary index you’ll need.

## Appendix I

Got it — here’s the missing context you can use as the **intro section** of your migration doc:

---

### Background: HashMap vs. StableBTreeMap

When working on the Internet Computer (ICP), we deal with two very different storage contexts:

#### `HashMap` (Rust standard library)

- Origin: Part of Rust’s standard library (`std::collections`), not ICP-specific.
- Purpose: In-memory, fast, general-purpose key–value storage.
- Characteristics:

  - Lives only in the WebAssembly heap (the canister’s volatile memory).
  - Data disappears on canister upgrade or restart.
  - Average O(1) lookups, extremely fast for ephemeral state.

This is what most Rust developers reach for by default when they need a map.

#### `StableBTreeMap` (DFINITY’s `ic-stable-structures` crate)

- Origin: Provided by DFINITY as part of the [ic-stable-structures](https://crates.io/crates/ic-stable-structures) library, specifically for ICP canisters.
- Purpose: Persistent storage abstraction built on top of ICP’s **stable memory**.
- Characteristics:

  - B-tree based (O(log n) operations) rather than hash-based.
  - Stores serialized values directly in stable memory, which survives upgrades.
  - Enforces `Storable`/`BoundedStorable` traits so keys/values can be written to raw stable memory.
  - Ordered by key, enabling range queries and pagination.

It was created because the IC runtime guarantees you access to up to hundreds of GB of stable memory, but no direct “HashMap” API for it. The `ic-stable-structures` crate bridges that gap by providing durable versions of familiar data structures (`StableBTreeMap`, `StableLog`, `StableVec`, etc.).

---

**In short:**

- `HashMap` is a general Rust tool, great for ephemeral state.
- `StableBTreeMap` is an ICP-specific tool, designed for canisters that need **data persistence across upgrades**.
