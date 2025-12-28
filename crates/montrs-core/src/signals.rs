//! montrs-core/src/signals.rs: Fine-grained reactivity system based on Signals.
//! This module implements a thread-safe reactive runtime that tracks subscribers
//! and notifies them when signal values change.

use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashSet;
use slotmap::{DefaultKey, SlotMap};
use once_cell::sync::Lazy;

/// A unique identifier for a reactive effect or subscriber.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriberId(DefaultKey);

/// The reactive runtime responsible for managing subscribers and dependency tracking.
/// In v0.1, it provides a centralized storage for effect callbacks.
pub struct ReactiveRuntime {
    /// Mapping of keys to closure behaviors that run when a dependency changes.
    subscribers: RwLock<SlotMap<DefaultKey, Arc<dyn Fn() + Send + Sync>>>,
    /// Tracks which subscribers depend on which other keys (for future graph optimization).
    dependencies: RwLock<std::collections::HashMap<DefaultKey, HashSet<DefaultKey>>>,
}

impl ReactiveRuntime {
    /// Initializes a new reactive runtime.
    pub fn new() -> Self {
        Self {
            subscribers: RwLock::new(SlotMap::new()),
            dependencies: RwLock::new(std::collections::HashMap::new()),
        }
    }
}

/// Global lazy-initialized reactive runtime instance.
static RUNTIME: Lazy<ReactiveRuntime> = Lazy::new(ReactiveRuntime::new);

/// A thread-safe reactive signal that holds a value of type T.
/// Signals are the primary atomic unit of state in MontRS.
pub struct Signal<T> {
    /// The actual data protected by an RwLock for thread-safe access.
    value: Arc<RwLock<T>>,
    /// A set of subscriber keys that are notified when this signal changes.
    subscribers: Arc<RwLock<HashSet<DefaultKey>>>,
}

impl<T: Send + Sync + 'static> Signal<T> {
    /// Creates a new Signal with an initial value.
    pub fn new(val: T) -> Self {
        Self {
            value: Arc::new(RwLock::new(val)),
            subscribers: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Returns a clone of the current value.
    /// In future versions, this will automatically register the current reactive scope as a dependency.
    pub fn get(&self) -> T 
    where T: Clone {
        self.value.read().clone()
    }

    /// Updates the signal's value and notifies all subscribers.
    pub fn set(&self, val: T) {
        {
            let mut writer = self.value.write();
            *writer = val;
        }
        self.notify();
    }

    /// Mutates the signal's value in-place via a closure and notifies subscribers.
    pub fn mutate<F: FnOnce(&mut T)>(&self, f: F) {
        {
            let mut writer = self.value.write();
            f(&mut *writer);
        }
        self.notify();
    }

    /// Internal helper to notify all registered subscribers of a change.
    fn notify(&self) {
        let subs = self.subscribers.read();
        for &sub_key in subs.iter() {
            let runtime_subs = RUNTIME.subscribers.read();
            if let Some(effect) = runtime_subs.get(sub_key) {
                effect();
            }
        }
    }
}

impl<T: Clone> Clone for Signal<T> {
    /// Signals can be cheaply cloned as they use Arc internally.
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
            subscribers: Arc::clone(&self.subscribers),
        }
    }
}
