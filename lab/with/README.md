# `with_` Convention in Rust

In Rust, the `with_` naming convention isn't enforced by the compiler, but it's a pretty common idiom you'll encounter in libraries and codebases. It usually means one of a few things depending on context:

### 1. Builder-style methods

A `with_foo` method often sets a field or configuration option and returns the modified object, supporting a builder pattern:

```rust
struct Config {
    timeout: u64,
    retries: u32,
}

impl Config {
    fn new() -> Self {
        Config { timeout: 30, retries: 3 }
    }

    fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }
}

let cfg = Config::new().with_timeout(60).with_retries(5);
```

Here `with_timeout` reads naturally as “give me this object, **with** timeout set to X.”

---

### 2. Passing a dependency or resource

Sometimes `with_` is used to indicate that the function requires you to supply something it will work “with”:

```rust
fn with_logger<F, R>(logger: &Logger, f: F) -> R
where
    F: FnOnce(&Logger) -> R,
{
    f(logger)
}
```

This pattern says “run something with this resource.” Similar to Haskell’s `withFile`, Python’s `with` context managers, etc.

---

### 3. Consistency/readability in APIs

It helps differentiate between plain setters vs. builder-style fluent setters:

- `set_timeout(&mut self, t: u64)` → mutates in place
- `with_timeout(self, t: u64) -> Self` → returns a new owned value

---

### Quick rules of thumb

- `set_foo`: mutates in place, needs `&mut self`.
- `with_foo`: consumes `self` (or clones it) and returns a new one.
- `into_foo`: converts `self` into something else.
- `as_foo`: borrows as another view.
- `to_foo`: creates a new owned conversion.

So: `with_` is about **builder-like ergonomics** and **fluent configuration**.

---

Do you want me to also show how `with_` gets used in big crates (like `tokio`, `serde`, `hyper`), so you can see concrete ecosystem patterns?

Alright — here’s how the “with” uses in other languages influenced Rust’s **`with_` convention**:

### 1. Builder-style setters (Rust influence: none of the keywords, just API ergonomics)

- In Rust, `with_timeout`, `with_capacity`, etc. are mainly from the **builder pattern tradition** (seen in Java, C#, etc.).
- The idea is: `set_x` mutates, `with_x` consumes & returns new.
- This doesn’t really come from Pascal/JS/Python, but from general OO/functional design patterns.

---

### 2. Resource/context management (inspired by Haskell/Python)

- Functions like `with_logger`, `with_connection`, or `with_env` in Rust mirror **Haskell’s `withFile`** or **Python’s `with open()`**.
- You pass in a closure; the function ensures setup/teardown, then calls your closure with the resource.
- Example in Rust:

  ```rust
  fn with_file<F, R>(path: &str, f: F) -> std::io::Result<R>
  where
      F: FnOnce(&mut std::fs::File) -> R,
  {
      let mut file = std::fs::File::open(path)?;
      Ok(f(&mut file))
  }
  ```

  This is a direct analogue of `withFile` in Haskell or `with open` in Python.

---

### 3. Scope shorteners (Pascal/JS influence: **not adopted**)

- Pascal’s `with Point do …` or JS’s `with(obj) { … }` made field access shorter.
- Rust _avoids_ this kind of `with` because it prefers explicitness and disambiguation.
- Instead, Rust uses `use`, `let p = &mut obj;`, or pattern matching (`let Point { x, y } = …;`) to achieve similar conciseness safely.

---

### Conclusion

- Rust’s **`with_` builder-style** → comes from OO/functional naming conventions, not keywords.
- Rust’s **`with_resource` closure helpers** → influenced by Haskell/Python `with`.
- Rust **avoids scope-shortening `with`** (Pascal/JS) because it clashes with Rust’s emphasis on clarity and safety.

So the `with_` convention in Rust is a **blend of Haskell/Python’s resource pattern and builder-pattern ergonomics**, but explicitly _not_ Pascal/JS’s “scope injection.”

Do you want me to dig up a few **real crate examples** (e.g. `with_capacity` in `Vec`, `with_context` in `anyhow`, `with_subscriber` in `tracing`) to see how these two influences play out in practice?

---

## Running the Examples

This crate demonstrates the `with_` convention with five progressive examples:

```bash
# 1. Simple Counter - Basic with_ pattern (uses unsafe)
cargo run --bin simple_counter

# 2. Simple Safe Counter - Basic with_ pattern (thread-safe, no unsafe)
cargo run --bin simple_safe_counter

# 3. Simple Lazy Counter - Most idiomatic version (LazyLock + AtomicU32)
cargo run --bin simple_lazy_counter

# 4. Complex Counter - Feature-rich example (uses unsafe)
cargo run --bin complex_counter

# 5. Safe Counter - Thread-safe, idiomatic version (no unsafe)
cargo run --bin safe_counter

# Run tests to see the pattern in action
cargo test
```

### Example Progression:

1. **Simple Counter**: Minimal `with_` pattern with a single global variable (uses `unsafe`)
2. **Simple Safe Counter**: Same as #1 but thread-safe with `OnceLock<Mutex<T>>` (no `unsafe`)
3. **Simple Lazy Counter**: Most idiomatic version with `LazyLock<AtomicU32>` (lock-free, auto-init)
4. **Complex Counter**: Multiple resources, access counting, helper functions (uses `unsafe`)
5. **Safe Counter**: Thread-safe with `OnceLock<Mutex<T>>`, builder patterns, file I/O (no `unsafe`)

The examples show:

- ✅ Controlled access to shared resources
- ✅ Automatic cleanup when closures end
- ✅ Clear intent: 'give me temporary access'
- ✅ Prevents accidental long-term borrowing
- ✅ Makes resource management explicit
- ✅ Thread-safe alternatives to `unsafe` code
