use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
// Import the correct guard type
use tokio::sync::{Mutex, OwnedMutexGuard};

// The inner state that the main Mutex will protect.
struct SwapperState<T: Eq + Hash> {
    mutexes: HashMap<T, Arc<Mutex<()>>>,
    ref_counts: HashMap<T, usize>,
}

// The main MutexSwapper struct.
pub struct MutexSwapper<T: Eq + Hash> {
    state: Mutex<SwapperState<T>>,
}

impl<T: Eq + Hash + Clone> MutexSwapper<T> {
    // Creates a new MutexSwapper.
    pub fn new() -> Self {
        Self {
            state: Mutex::new(SwapperState {
                mutexes: HashMap::new(),
                ref_counts: HashMap::new(),
            }),
        }
    }

    // Acquires a lock for a given key.
    // Returns a guard that will release the lock when dropped.
    pub async fn lock(&self, key: T) -> OwnedMutexGuard<()> {
        let per_key_mutex = {
            let mut state = self.state.lock().await;

            // Increment the reference count for the key.
            *state.ref_counts.entry(key.clone()).or_insert(0) += 1;

            // Get or create the mutex for the key.
            state
                .mutexes
                .entry(key)
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };

        // This line is now correct because the function signature matches.
        per_key_mutex.lock_owned().await
    }

    // We don't need the `unlock` method anymore because the RAII guard
    // handles everything when it goes out of scope. For simplicity and
    // to prevent potential misuse, we can remove it.
}
