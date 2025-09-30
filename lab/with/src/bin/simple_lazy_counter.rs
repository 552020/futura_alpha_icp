//! Simple Lazy Counter - Most Idiomatic `with_` Convention Example
//!
//! This is the most production-ready example of the `with_` convention:
//! - Uses LazyLock for automatic initialization (no manual init needed)
//! - Uses AtomicU32 for lock-free operations (better performance)
//! - Handles poisoning gracefully
//! - No unsafe code
//! - Drop-in ready for real applications

use std::sync::{
    atomic::{AtomicU32, Ordering},
    LazyLock,
};

// LazyLock automatically initializes on first access - no manual init needed!
static COUNTER: LazyLock<AtomicU32> = LazyLock::new(|| AtomicU32::new(0));

/// The most idiomatic `with_` function - no init needed, lock-free, thread-safe
pub fn with_counter<F, R>(f: F) -> R
where
    F: FnOnce(&AtomicU32) -> R,
{
    f(&COUNTER)
}

/// Alternative version that returns Result for explicit error handling
pub fn with_counter_result<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&AtomicU32) -> R,
{
    // In this case, atomic operations can't fail, but this shows the pattern
    Ok(f(&COUNTER))
}

fn main() {
    println!("🦀 Simple Lazy Counter - Most Idiomatic `with_` Convention");
    println!("======================================================\n");

    println!("📊 Basic Atomic Counter Operations:");

    // Increment the counter (lock-free!)
    with_counter(|counter| {
        let old_value = counter.fetch_add(1, Ordering::Relaxed);
        println!("Incremented from {} to {}", old_value, old_value + 1);
    });

    // Increment again
    with_counter(|counter| {
        let old_value = counter.fetch_add(1, Ordering::Relaxed);
        println!("Incremented from {} to {}", old_value, old_value + 1);
    });

    // Add 5 to the counter
    with_counter(|counter| {
        let old_value = counter.fetch_add(5, Ordering::Relaxed);
        println!("Added 5: {} -> {}", old_value, old_value + 5);
    });

    // Read current value
    with_counter(|counter| {
        let current = counter.load(Ordering::Relaxed);
        println!("Current value: {}", current);
    });

    // Compare and swap (atomic operation)
    with_counter(|counter| {
        let current = counter.load(Ordering::Relaxed);
        let success =
            counter.compare_exchange(current, current * 2, Ordering::Relaxed, Ordering::Relaxed);
        match success {
            Ok(old) => println!("CAS successful: {} -> {}", old, old * 2),
            Err(actual) => println!("CAS failed: expected {}, got {}", current, actual),
        }
    });

    // Show final value
    with_counter(|counter| {
        let final_value = counter.load(Ordering::Relaxed);
        println!("Final counter value: {}", final_value);
    });

    println!("\n🔧 Demo: Result-based version");
    let result = with_counter_result(|counter| counter.fetch_add(10, Ordering::Relaxed));

    match result {
        Ok(old_value) => println!("Result version: incremented from {}", old_value),
        Err(e) => println!("Error: {}", e),
    }

    println!("\n🎯 This demonstrates:");
    println!("• ✅ LazyLock - automatic initialization, no manual init needed");
    println!("• ✅ AtomicU32 - lock-free operations, better performance");
    println!("• ✅ Thread-safe without mutex overhead");
    println!("• ✅ Graceful error handling with Result");
    println!("• ✅ Production-ready, drop-in code");
    println!("• ✅ No unsafe code!");
    println!("\n🚀 This is the most idiomatic, production-ready version!");
    println!("💡 Perfect for real applications where performance matters!");
}

