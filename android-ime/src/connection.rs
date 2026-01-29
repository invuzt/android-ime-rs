use crate::sys::JImeView;
use jni::objects::JObject;
use jni::JNIEnv;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};
use crate::handler::AndroidImeConnectionHandler;

////////////////////////////////////////////////////////////////////////////////
static HANDLERS: LazyLock<Mutex<HashMap<u64, Arc<dyn AndroidImeConnectionHandler>>>> = LazyLock::new(Mutex::default);

pub(crate) fn find_handler(id: u64) -> Option<Arc<dyn AndroidImeConnectionHandler>> {
    HANDLERS.lock().get(&id).cloned()
}

////////////////////////////////////////////////////////////////////////////////
pub struct AndroidImeConnection<T> {
    id: u64,
    view: JImeView,
    handler: Arc<T>,
}

impl<T: AndroidImeConnectionHandler> AndroidImeConnection<T> {
    pub fn new<'a>(env: &mut JNIEnv<'a>, activity: &JObject<'a>, handler: T) -> anyhow::Result<Self> {
        static ID: AtomicU64 = AtomicU64::new(0);

        let view = JImeView::from(env, activity)?;
        let id = ID.fetch_add(1, Ordering::Relaxed);
        let handler = Arc::new(handler);
        HANDLERS.lock().insert(id, handler.clone());

        Ok(AndroidImeConnection { id, view, handler })
    }

    pub fn activate(&self) -> anyhow::Result<()> {
        self.view.activate(self.id)
    }

    pub fn deactivate(&self) -> anyhow::Result<()> {
        self.view.deactivate(self.id)
    }
}

impl<T> Drop for AndroidImeConnection<T> {
    fn drop(&mut self) {
        HANDLERS.lock().remove(&self.id);
    }
}

impl<T> Deref for AndroidImeConnection<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.handler
    }
}
