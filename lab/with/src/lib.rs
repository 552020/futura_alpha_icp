//! Library for demonstrating the `with_` naming convention
//!
//! This crate provides utilities for the `with_` naming convention,
//! which is used for functions that provide temporary access to resources
//! and clean up automatically.

use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

/// A simple counter that tracks how many times it's been accessed
#[derive(Debug, Default)]
pub struct Counter {
    pub value: u32,
    pub access_count: u32,
}

impl Counter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&mut self) {
        self.value += 1;
        self.access_count += 1;
    }

    pub fn get_value(&self) -> u32 {
        self.value
    }

    pub fn get_access_count(&self) -> u32 {
        self.access_count
    }
}

/// A simple message store
#[derive(Debug, Default)]
pub struct MessageStore {
    pub messages: HashMap<String, String>,
}

impl MessageStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_message(&mut self, key: String, message: String) {
        self.messages.insert(key, message);
    }

    pub fn get_message(&self, key: &str) -> Option<&String> {
        self.messages.get(key)
    }

    pub fn count_messages(&self) -> usize {
        self.messages.len()
    }
}

/// Configuration struct for builder pattern examples
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub timeout: u64,
    pub retries: u32,
}

impl Config {
    pub fn new() -> Self {
        Self {
            timeout: 30,
            retries: 3,
        }
    }

    // set_: in-place mutation, needs &mut self
    pub fn set_timeout(&mut self, t: u64) {
        self.timeout = t;
    }

    pub fn set_retries(&mut self, r: u32) {
        self.retries = r;
    }

    // with_: fluent builder, consumes self and returns a new one
    pub fn with_timeout(mut self, t: u64) -> Self {
        self.timeout = t;
        self
    }

    pub fn with_retries(mut self, r: u32) -> Self {
        self.retries = r;
        self
    }

    // as_: borrow a view
    pub fn as_timeout(&self) -> &u64 {
        &self.timeout
    }

    pub fn as_retries(&self) -> &u32 {
        &self.retries
    }

    // to_: produce a new owned value from a borrow
    pub fn to_string_repr(&self) -> String {
        format!("timeout={}, retries={}", self.timeout, self.retries)
    }

    // into_: turn self into something else, consuming it
    pub fn into_tuple(self) -> (u64, u32) {
        (self.timeout, self.retries)
    }
}

/// Thread-safe global counter using OnceLock + Mutex
pub struct SafeCounter;

impl SafeCounter {
    static COUNTER: OnceLock<Mutex<Counter>> = OnceLock::new();

    pub fn init() {
        let _ = Self::COUNTER.set(Mutex::new(Counter::new()));
    }

    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Counter) -> R,
    {
        let m = Self::COUNTER.get().expect("call SafeCounter::init() first");
        let mut c = m.lock().unwrap();
        f(&mut *c)
    }
}

/// Thread-safe global message store using OnceLock + Mutex
pub struct SafeMessageStore;

impl SafeMessageStore {
    static STORE: OnceLock<Mutex<MessageStore>> = OnceLock::new();

    pub fn init() {
        let _ = Self::STORE.set(Mutex::new(MessageStore::new()));
    }

    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut MessageStore) -> R,
    {
        let m = Self::STORE.get().expect("call SafeMessageStore::init() first");
        let mut s = m.lock().unwrap();
        f(&mut *s)
    }

    pub fn with_read<F, R>(f: F) -> R
    where
        F: FnOnce(&MessageStore) -> R,
    {
        let m = Self::STORE.get().expect("call SafeMessageStore::init() first");
        let s = m.lock().unwrap();
        f(&*s)
    }
}