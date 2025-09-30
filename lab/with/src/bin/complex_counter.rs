//! Complex Counter - Feature-Rich `with_` Convention Example
//!
//! This example shows the `with_` convention with multiple features:
//! - Multiple global resources (counter + message store)
//! - Access counting and tracking
//! - Helper functions that use the pattern
//! - Multiple `with_` variants (read/write)

use with::{Counter, MessageStore};

// Global resources (using unsafe for simplicity - see safe_counter.rs for idiomatic version)
static mut GLOBAL_COUNTER: Option<Counter> = None;
static mut MESSAGE_STORE: Option<MessageStore> = None;

/// Initialize global resources
fn init_globals() {
    unsafe {
        GLOBAL_COUNTER = Some(Counter::new());
        MESSAGE_STORE = Some(MessageStore::new());
    }
}

/// The `with_` convention: provide temporary access to the global counter
fn with_counter<F, R>(f: F) -> R
where
    F: FnOnce(&mut Counter) -> R,
{
    unsafe {
        if let Some(ref mut counter) = GLOBAL_COUNTER {
            f(counter)
        } else {
            panic!("Global counter not initialized! Call init_globals() first.");
        }
    }
}

/// The `with_` convention: provide temporary access to the message store
fn with_message_store<F, R>(f: F) -> R
where
    F: FnOnce(&mut MessageStore) -> R,
{
    unsafe {
        if let Some(ref mut store) = MESSAGE_STORE {
            f(store)
        } else {
            panic!("Global message store not initialized! Call init_globals() first.");
        }
    }
}

/// The `with_` convention: provide read-only access to the message store
fn with_message_store_read<F, R>(f: F) -> R
where
    F: FnOnce(&MessageStore) -> R,
{
    unsafe {
        if let Some(ref store) = MESSAGE_STORE {
            f(store)
        } else {
            panic!("Global message store not initialized! Call init_globals() first.");
        }
    }
}

/// Helper function that demonstrates the pattern
fn hello_world() -> String {
    with_counter(|counter| {
        counter.increment();
        format!("Hello! Counter is now: {}", counter.get_value())
    })
}

/// Add a message using the with_ convention
fn add_hello_message(key: String, message: String) {
    with_message_store(|store| {
        store.add_message(key, message);
    });
}

/// Get a message using the with_ convention
fn get_hello_message(key: &str) -> Option<String> {
    with_message_store_read(|store| store.get_message(key).cloned())
}

fn main() {
    println!("🦀 Complex Counter - Feature-Rich `with_` Convention");
    println!("================================================\n");

    // Initialize global resources
    init_globals();
    println!("✅ Initialized global resources\n");

    // Demo 1: Basic hello world with counter
    println!("📊 Demo 1: Hello World with Counter");
    println!("{}", hello_world());
    println!("{}", hello_world());
    println!("{}", hello_world());
    println!();

    // Demo 2: Direct use of with_ convention
    println!("🔧 Demo 2: Direct `with_` Convention Usage");
    with_counter(|counter| {
        println!("Counter value: {}", counter.get_value());
        println!("Access count: {}", counter.get_access_count());
    });
    println!();

    // Demo 3: Message store with with_ convention
    println!("💬 Demo 3: Message Store with `with_` Convention");

    // Add messages
    add_hello_message("greeting".to_string(), "Hello, Rust!".to_string());
    add_hello_message("farewell".to_string(), "Goodbye, World!".to_string());
    add_hello_message("question".to_string(), "How are you?".to_string());

    // Read messages
    if let Some(greeting) = get_hello_message("greeting") {
        println!("Greeting: {}", greeting);
    }

    if let Some(farewell) = get_hello_message("farewell") {
        println!("Farewell: {}", farewell);
    }

    if let Some(question) = get_hello_message("question") {
        println!("Question: {}", question);
    }
    println!();

    // Demo 4: Show controlled access
    println!("🔒 Demo 4: Controlled Access Pattern");
    with_message_store(|store| {
        println!("Total messages stored: {}", store.count_messages());

        // We have mutable access here, but it's controlled
        store.add_message("demo".to_string(), "This is a demo message".to_string());
        println!("Added demo message, total now: {}", store.count_messages());
    });

    // Outside the closure, we can't accidentally modify the store
    println!("✅ Access is automatically cleaned up when closure ends");
    println!();

    // Demo 5: Show the pattern benefits
    println!("🎯 Demo 5: Why `with_` Convention is Useful");
    println!("• ✅ Controlled access to shared resources");
    println!("• ✅ Automatic cleanup when done");
    println!("• ✅ Clear intent: 'give me temporary access'");
    println!("• ✅ Prevents accidental long-term borrowing");
    println!("• ✅ Makes resource management explicit");
    println!();

    println!("🚀 Complex demo complete! The `with_` convention provides safe, controlled access to resources.");
    println!("⚠️  Note: This uses `unsafe` - see safe_counter.rs for the thread-safe version!");
}

