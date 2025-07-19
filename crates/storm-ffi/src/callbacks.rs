// File: crates/storm-ffi/src/callbacks.rs
// FFI callback management

use std::os::raw::c_void;

/// Generic callback type
pub type Callback = extern "C" fn(*mut c_void);

/// Callback manager
pub struct CallbackManager {
    callbacks: std::collections::HashMap<String, Callback>,
}

impl CallbackManager {
    pub fn new() -> Self {
        Self {
            callbacks: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, callback: Callback) {
        self.callbacks.insert(name, callback);
    }
}
