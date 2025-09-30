//! Simple Counter - Basic `with_` Convention Example
//!
//! This is the most basic example of the `with_` convention:
//! - Single global variable
//! - One `with_` function
//! - Simple closure that modifies the variable

// The absolute simplest with_ convention
static mut COUNTER: u32 = 0;

fn with_counter<F, R>(f: F) -> R
where
    F: FnOnce(&mut u32) -> R,
{
    unsafe { f(&mut COUNTER) }
}

fn main() {
    println!("🦀 Simple Counter - Basic `with_` Convention");
    println!("==========================================\n");

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

    println!("\n🎯 This demonstrates:");
    println!("• ✅ Single global resource (counter)");
    println!("• ✅ One `with_` function for access");
    println!("• ✅ Simple closure that modifies the resource");
    println!("• ✅ Access ends when closure returns");
    println!("\n⚠️  Note: This uses `unsafe` - see safe_counter.rs for the idiomatic version!");
}
