use crate::sys::JImeView;
use jni::objects::JObject;
use jni::JNIEnv;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

////////////////////////////////////////////////////////////////////////////////
static HANDLERS: LazyLock<Mutex<HashMap<u64, Arc<dyn AndroidImeConnectionHandler>>>> = LazyLock::new(Mutex::default);

pub(crate) fn find_handler(id: u64) -> Option<Arc<dyn AndroidImeConnectionHandler>> {
    HANDLERS.lock().get(&id).cloned()
}

////////////////////////////////////////////////////////////////////////////////
pub trait AndroidImeConnectionHandler: 'static + Send + Sync {
    fn connection_closed(&self);
    fn send_key_event(&self, key_code: i32) -> bool;
    fn perform_context_menu_action(&self, action_id: i32) -> bool;
    fn perform_editor_action(&self, editor_action: i32) -> bool;

    fn commit_text(&self, text: &str, new_cursor_position: i32) -> bool;
    fn delete_surrounding_text(&self, before: usize, after: usize) -> bool;
    fn delete_surrounding_text_in_code_points(&self, before: usize, after: usize) -> bool;

    fn set_selection(&self, start: usize, end: usize) -> bool;
    fn set_composing_region(&self, start: usize, end: usize) -> bool;
    fn set_composing_text(&self, input: &str, new_cursor_position: i32) -> bool;
    fn finish_composing_text(&self) -> bool;

    fn get_selected_text(&self, flags: i32) -> Option<&str>;
    fn get_text_after_cursor(&self, count: usize, flags: i32) -> Option<&str>;
    fn get_text_before_cursor(&self, count: usize, flags: i32) -> Option<&str>;
    fn get_cursor_caps_mode(&self, req_modes: i32) -> i32;
    fn request_cursor_updates(&self, cursor_update_mode: i32) -> bool;
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
