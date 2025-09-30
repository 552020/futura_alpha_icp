//! Simple Safe Counter - Basic `with_` Convention (Thread-Safe)
//!
//! This is the most basic example of the `with_` convention but thread-safe:
//! - Single global variable using OnceLock + Mutex
//! - One `with_` function
//! - Simple closure that modifies the variable
//! - No unsafe code

use std::sync::{Mutex, OnceLock};

// Thread-safe global counter (no unsafe!)
static COUNTER: OnceLock<Mutex<u32>> = OnceLock::new();

fn init_counter() {
    let _ = COUNTER.set(Mutex::new(0));
}

fn with_counter<F, R>(f: F) -> R
where
    F: FnOnce(&mut u32) -> R,
{
    let m = COUNTER.get().expect("call init_counter() first");
    let mut c = m.lock().unwrap();
    f(&mut *c)
}

fn main() {
    println!("🦀 Simple Safe Counter - Basic `with_` Convention (Thread-Safe)");
    println!("==========================================================\n");

    // Initialize the thread-safe counter
    init_counter();
    println!("✅ Initialized thread-safe counter\n");

    println!("📊 Basic Counter Operations:");

    // Increment the counter
    with_counter(|counter| {
        *counter += 1;
        println!("Counter: {}", counter);
    });

    // Increment again
    with_counter(|counter| {
        *counter += 1;
        println!("Counter: {}", counter);
    });

    // Add 5 to the counter
    with_counter(|counter| {
        *counter += 5;
        println!("Counter: {}", counter);
    });

    // Show final value
    with_counter(|counter| {
        println!("Final counter value: {}", counter);
    });

    println!("\n🎯 This demonstrates:");
    println!("• ✅ Single global resource (counter)");
    println!("• ✅ One `with_` function for access");
    println!("• ✅ Simple closure that modifies the resource");
    println!("• ✅ Access ends when closure returns");
    println!("• ✅ Thread-safe with OnceLock + Mutex");
    println!("• ✅ No unsafe code!");
    println!("\n🚀 This is the idiomatic, production-ready version of the basic pattern!");
}
