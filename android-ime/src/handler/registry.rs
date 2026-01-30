use crate::AndroidImeConnectionHandler;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

////////////////////////////////////////////////////////////////////////////////
pub struct AndroidImeConnectionRegistry {
    handlers: Mutex<HashMap<u64, Arc<dyn AndroidImeConnectionHandler>>>,
}

impl AndroidImeConnectionRegistry {
    pub fn get() -> &'static Self {
        static CELL: OnceLock<AndroidImeConnectionRegistry> = OnceLock::new();
        CELL.get_or_init(|| Self {
            handlers: Default::default(),
        })
    }

    pub fn register(&self, id: u64, handler: Arc<dyn AndroidImeConnectionHandler>) {
        self.handlers.lock().insert(id, handler);
    }

    pub fn unregister(&self, id: u64) {
        self.handlers.lock().remove(&id);
    }

    pub fn find_handler(&self, id: u64) -> Option<Arc<dyn AndroidImeConnectionHandler>> {
        self.handlers.lock().get(&id).cloned()
    }
}
