// File: crates/storm-core/src/core/handle.rs
// Opaque handle management for FFI interfaces

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::StormCore;

/// Opaque handle to StormCore instance for FFI
#[repr(C)]
pub struct StormHandle {
    pub(crate) id: Uuid,
    pub(crate) core: Arc<RwLock<Option<StormCore>>>,
}

impl StormHandle {
    pub fn new(core: StormCore) -> Self {
        Self {
            id: Uuid::new_v4(),
            core: Arc::new(RwLock::new(Some(core))),
        }
    }

    pub async fn with_core<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&StormCore) -> R,
    {
        let guard = self.core.read().await;
        guard.as_ref().map(f)
    }

    pub async fn with_core_mut<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut StormCore) -> R,
    {
        let mut guard = self.core.write().await;
        guard.as_mut().map(f)
    }

    pub async fn take_core(&self) -> Option<StormCore> {
        let mut guard = self.core.write().await;
        guard.take()
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

unsafe impl Send for StormHandle {}
unsafe impl Sync for StormHandle {}