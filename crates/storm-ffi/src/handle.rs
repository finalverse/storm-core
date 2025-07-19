// File: crates/storm-ffi/src/handle.rs
// FFI handle management

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Opaque FFI handle
pub struct Handle {
    pub id: Uuid,
    pub data: Arc<RwLock<Option<Box<dyn std::any::Any + Send + Sync>>>>,
}

impl Handle {
    pub fn new<T: 'static + Send + Sync>(data: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            data: Arc::new(RwLock::new(Some(Box::new(data)))),
        }
    }
}

unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}
