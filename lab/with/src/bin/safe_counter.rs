//! Safe Counter - Thread-Safe, Idiomatic `with_` Convention Example
//!
//! This demonstrates the proper, thread-safe way to use the `with_` convention
//! without `unsafe` code, plus shows the builder pattern vs setter pattern.

use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

use with::{Config, SafeCounter, SafeMessageStore};

// Classic "with a resource" helper (à la Haskell/Python)
fn with_file_write<P, F, R>(path: P, f: F) -> io::Result<R>
where
    P: AsRef<Path>,
    F: FnOnce(&mut File) -> R,
{
    let mut file = File::create(path)?;
    Ok(f(&mut file)) // file closes automatically (RAII)
}

fn main() {
    println!("🦀 Safe Counter - Thread-Safe, Idiomatic `with_` Convention");
    println!("=======================================================\n");

    // ============================================================================
    // Demo 1: Thread-safe "with resource" pattern
    // ============================================================================
    println!("🔒 Demo 1: Thread-Safe 'with resource' Pattern");
    SafeCounter::init();
    SafeMessageStore::init();

    // Multiple calls to show thread safety
    let result1 = SafeCounter::with(|counter| {
        counter.increment();
        counter.increment();
        format!("Counter: {}, Accesses: {}", counter.get_value(), counter.get_access_count())
    });

    let result2 = SafeCounter::with(|counter| {
        counter.increment();
        format!("Counter: {}, Accesses: {}", counter.get_value(), counter.get_access_count())
    });

    println!("{}", result1);
    println!("{}", result2);
    println!();

    // ============================================================================
    // Demo 2: File resource with automatic cleanup
    // ============================================================================
    println!("📁 Demo 2: File Resource with Automatic Cleanup");
    
    let temp_file = "/tmp/rust_with_demo.txt";
    let result = with_file_write(temp_file, |file| {
        writeln!(file, "Hello from the with_ convention!").unwrap();
        writeln!(file, "This file will be automatically closed.").unwrap();
        "File written successfully!"
    });

    match result {
        Ok(msg) => println!("✅ {}", msg),
        Err(e) => println!("❌ Error: {}", e),
    }
    println!();

    // ============================================================================
    // Demo 3: Builder vs Setter patterns
    // ============================================================================
    println!("🔧 Demo 3: Builder vs Setter Patterns");

    // Using setters (mutate in place)
    let mut config1 = Config::new();
    println!("Initial config: {:?}", config1);
    
    config1.set_timeout(60);
    config1.set_retries(5);
    println!("After setters: {:?}", config1);

    // Using builder pattern (fluent chaining)
    let config2 = Config::new()
        .with_timeout(120)
        .with_retries(10);
    println!("Builder pattern: {:?}", config2);
    println!();

    // ============================================================================
    // Demo 4: as_ vs to_ vs into_ patterns
    // ============================================================================
    println!("🔄 Demo 4: as_ vs to_ vs into_ Patterns");

    let config = Config::new().with_timeout(90).with_retries(7);

    // as_: borrow a view
    let timeout_ref = config.as_timeout();
    let retries_ref = config.as_retries();
    println!("as_ (borrowed views): timeout={}, retries={}", timeout_ref, retries_ref);

    // to_: produce new owned value from borrow
    let description = config.to_string_repr();
    println!("to_ (new owned): {}", description);

    // into_: consume self into another type
    let (timeout, retries) = config.into_tuple();
    println!("into_ (consumed): timeout={}, retries={}", timeout, retries);
    println!();

    // ============================================================================
    // Demo 5: Thread-safe message store
    // ============================================================================
    println!("💬 Demo 5: Thread-Safe Message Store");
    
    SafeMessageStore::with(|store| {
        store.add_message("greeting".to_string(), "Hello, Safe Rust!".to_string());
        store.add_message("farewell".to_string(), "Goodbye, Unsafe Code!".to_string());
    });
    
    SafeMessageStore::with_read(|store| {
        println!("Total messages: {}", store.count_messages());
        if let Some(greeting) = store.get_message("greeting") {
            println!("Greeting: {}", greeting);
        }
    });
    println!();

    // ============================================================================
    // Demo 6: When to use which pattern
    // ============================================================================
    println!("🎯 Demo 6: When to Use Which Pattern");
    println!("• set_foo(&mut self, ...) → straightforward in-place updates");
    println!("• with_foo(self, ...) → Self → fluent configuration/builders");
    println!("• with_resource(|...| ...) → scoped setup/teardown around resources");
    println!("• as_foo(&self) → borrow a view");
    println!("• to_foo(&self) → produce new owned value from borrow");
    println!("• into_foo(self) → consume self into another type");
    println!();

    println!("🚀 Safe demo complete! No unsafe code, thread-safe, and shows all patterns.");
    println!("✅ This is the production-ready, idiomatic way to use the `with_` convention!");
}

