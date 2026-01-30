mod helpers;

use crate::context::AndroidImeContext;
use crate::editable::helpers::*;
use crate::{AndroidImeConnection, AndroidImeConnectionHandler};
use log::{debug, error};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::borrow::Cow;
use std::ops::{Deref, DerefMut, Range};
use std::sync::{Arc, Weak};

////////////////////////////////////////////////////////////////////////////////
pub struct AndroidImeEditable {
    state: Arc<RwLock<AndroidImeEditableState>>,
}

impl AndroidImeEditable {
    pub fn new(context: &AndroidImeContext, handler: impl AndroidImeEditableHandler) -> Self {
        let content = AndroidImeEditableContent {
            text: "".to_string(),
            selection: 0..0,
            composition: None,
        };

        let state = AndroidImeEditableState {
            handler: Arc::new(handler),
            context: context.clone(),
            content,
            connection: None,
        };

        Self {
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub fn content(&self) -> AndroidImeEditableRef<'_> {
        AndroidImeEditableRef {
            guard: self.state.read(),
        }
    }

    pub fn content_mut(&mut self) -> AndroidImeEditableMut<'_> {
        AndroidImeEditableMut {
            modified: false,
            guard: self.state.write(),
        }
    }

    pub fn show_soft_keyboard(&mut self) -> anyhow::Result<()> {
        let state = &mut *self.state.write();
        state.connection = Some(AndroidImeConnection::open(
            &state.context,
            Handler {
                state: Arc::downgrade(&self.state),
            },
        )?);
        Ok(())
    }

    pub fn hide_soft_keyboard(&mut self) -> anyhow::Result<()> {
        let Some(connection) = self.state.write().connection.take() else {
            return Ok(());
        };
        connection.detach()
    }
}

////////////////////////////////////////////////////////////////////////////////
struct AndroidImeEditableState {
    handler: Arc<dyn AndroidImeEditableHandler>,
    context: AndroidImeContext,
    content: AndroidImeEditableContent,
    connection: Option<AndroidImeConnection<Handler>>,
}

impl AndroidImeEditableState {
    fn notify_selection_changed(&self) {
        let Some(connection) = self.connection.as_ref() else {
            return;
        };

        let result = self.context.view.update_selection(
            connection.id(),
            self.content.selection.clone(),
            self.content.composition.clone(),
        );

        if let Err(e) = result {
            error!("failed to update IME selection: {e:?}");
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct AndroidImeEditableContent {
    pub text: String,
    pub selection: Range<usize>,
    pub composition: Option<Range<usize>>,
}

impl AndroidImeEditableContent {
    pub fn replace(&mut self, range: Range<usize>, text: &str) {
        error!("replace: range = {range:?}, text = {text:?}");
        self.text.replace_range(range.clone(), text);

        self.selection.start = range.start + text.len();
        self.selection.end = self.selection.start;

        self.composition = None;
    }

    pub fn move_selection(&mut self, offset: isize) {
        if offset == 0 {
            return;
        }

        fn do_offset(text: &str, mut index: usize, offset: isize) -> usize {
            if offset > 0 {
                for _ in 0..offset {
                    index = text.ceil_char_boundary(index.saturating_add(1));
                }
            } else {
                for _ in offset..0 {
                    index = text.floor_char_boundary(index.saturating_sub(1));
                }
            }
            index
        }

        self.selection.start = do_offset(&self.text, self.selection.start, offset);
        self.selection.end = do_offset(&self.text, self.selection.end, offset);
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct AndroidImeEditableRef<'a> {
    guard: RwLockReadGuard<'a, AndroidImeEditableState>,
}

impl<'a> Deref for AndroidImeEditableRef<'a> {
    type Target = AndroidImeEditableContent;

    fn deref(&self) -> &Self::Target {
        &self.guard.content
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct AndroidImeEditableMut<'a> {
    modified: bool,
    guard: RwLockWriteGuard<'a, AndroidImeEditableState>,
}

impl Deref for AndroidImeEditableMut<'_> {
    type Target = AndroidImeEditableContent;

    fn deref(&self) -> &Self::Target {
        &self.guard.content
    }
}

impl DerefMut for AndroidImeEditableMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.modified = true;
        &mut self.guard.content
    }
}

impl Drop for AndroidImeEditableMut<'_> {
    fn drop(&mut self) {
        if self.modified {
            self.guard.notify_selection_changed();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
pub trait AndroidImeEditableHandler: 'static + Send + Sync {
    fn text_updated(&self);

    fn send_key_event(&self, key_code: i32) -> bool {
        let _ = key_code;
        false
    }

    fn perform_context_menu_action(&self, action: i32) -> bool {
        let _ = action;
        false
    }

    fn perform_editor_action(&self, action: i32) -> bool {
        let _ = action;
        false
    }
}

////////////////////////////////////////////////////////////////////////////////
struct Handler {
    state: Weak<RwLock<AndroidImeEditableState>>,
}

impl AndroidImeConnectionHandler for Handler {
    fn connection_closed(&self) {
        debug!("connection_closed");
    }

    fn send_key_event(&self, key_code: i32) -> bool {
        debug!("send_key_event: key_code = {key_code}");
        false
    }

    fn perform_context_menu_action(&self, action: i32) -> bool {
        debug!("perform_context_menu_action: action = {action}");
        false
    }

    fn perform_editor_action(&self, action: i32) -> bool {
        debug!("perform_editor_action: action = {action}");
        false
    }

    fn begin_batch_edit(&self) -> bool {
        debug!("begin_batch_edit");
        false
    }

    fn end_batch_edit(&self) -> bool {
        debug!("end_batch_edit");
        false
    }

    fn commit_text(&self, text: &str, new_cursor_position: isize) -> bool {
        debug!("commit_text: text = {text}, new_cursor_position = {new_cursor_position}");

        let Some(state) = self.state.upgrade() else {
            return false;
        };

        let state = &mut *state.write();
        let content = &mut state.content;

        match content.composition.take() {
            Some(e) => content.replace(e, text),
            None => content.replace(content.selection.clone(), text),
        }
        content.composition = None;
        content.selection = update_cursor_position(
            &content.text,
            (content.selection.end - text.len())..content.selection.end,
            new_cursor_position,
        );

        state.notify_selection_changed();
        state.handler.text_updated();
        true
    }

    fn delete_surrounding_text(&self, before: usize, after: usize) -> bool {
        debug!("delete_surrounding_text: before = {before}, after = {after}");

        let Some(state) = self.state.upgrade() else {
            return false;
        };

        let state = &mut *state.write();
        let content = &mut state.content;

        delete_text_after_in_utf16_code_units(&mut content.text, &mut content.selection, after);
        delete_text_before_in_utf16_code_units(&mut content.text, &mut content.selection, before);

        state.notify_selection_changed();
        state.handler.text_updated();
        true
    }

    fn delete_surrounding_text_in_code_points(&self, before: usize, after: usize) -> bool {
        debug!("delete_surrounding_text_in_code_points: before = {before}, after = {after}");

        let Some(state) = self.state.upgrade() else {
            return false;
        };

        let state = &mut *state.write();
        let content = &mut state.content;

        delete_text_after_in_utf16_code_points(&mut content.text, &mut content.selection, after);
        delete_text_before_in_utf16_code_points(&mut content.text, &mut content.selection, before);

        state.notify_selection_changed();
        state.handler.text_updated();
        true
    }

    fn set_selection(&self, start: usize, end: usize) -> bool {
        debug!("set_selection: start = {start}, end = {end}");

        let Some(state) = self.state.upgrade() else {
            return false;
        };

        let state = &mut *state.write();
        let content = &mut state.content;

        content.selection = char_range_to_index_range(&content.text, start..end);

        state.notify_selection_changed();
        state.handler.text_updated();
        true
    }

    fn set_composing_region(&self, start: usize, end: usize) -> bool {
        debug!("set_composing_region: start = {start}, end = {end}");

        let Some(state) = self.state.upgrade() else {
            return false;
        };

        let state = &mut *state.write();
        let content = &mut state.content;

        content.composition = Some(char_range_to_index_range(&content.text, start..end));

        state.handler.text_updated();
        true
    }

    fn set_composing_text(&self, text: &str, new_cursor_position: isize) -> bool {
        debug!("set_composing_text: text = {text}, new_cursor_position = {new_cursor_position}");

        let Some(state) = self.state.upgrade() else {
            return false;
        };

        let state = &mut *state.write();
        let content = &mut state.content;

        match content.composition.take() {
            Some(e) => {
                content.replace(e.clone(), text);
                content.composition = Some(e.start..(e.start + text.len()));
            }
            None => {
                let selection = content.selection.clone();
                content.replace(selection.clone(), text);
                content.composition = Some(selection.start..(selection.start + text.len()));
            }
        }
        content.selection = update_cursor_position(
            &content.text,
            (content.selection.end - text.len())..content.selection.end,
            new_cursor_position,
        );

        state.handler.text_updated();
        true
    }

    fn finish_composing_text(&self) -> bool {
        debug!("finish_composing_text");

        let Some(state) = self.state.upgrade() else {
            return false;
        };

        let state = &mut *state.write();
        let content = &mut state.content;

        content.composition = None;

        state.handler.text_updated();
        true
    }

    fn get_selected_text(&self) -> Option<Cow<'_, str>> {
        debug!("get_selected_text");

        let state = self.state.upgrade()?;
        let state = &mut *state.write();
        let content = &mut state.content;

        let text = content.text.get(content.selection.clone())?;

        Some(Cow::Owned(text.to_owned()))
    }

    fn get_text_after_cursor(&self, count: usize) -> Option<Cow<'_, str>> {
        debug!("get_text_after_cursor: count = {count}");

        let state = self.state.upgrade()?;
        let state = &mut *state.write();
        let content = &mut state.content;

        let text = get_slice_after(&content.text, content.selection.end, count)?;

        Some(Cow::Owned(text.to_owned()))
    }

    fn get_text_before_cursor(&self, count: usize) -> Option<Cow<'_, str>> {
        debug!("get_text_before_cursor: count = {count}");

        let state = self.state.upgrade()?;
        let state = &mut *state.write();
        let content = &mut state.content;

        let text = get_slice_before(&content.text, content.selection.end, count)?;

        Some(Cow::Owned(text.to_owned()))
    }

    fn get_cursor_caps_mode(&self, req_modes: i32) -> i32 {
        debug!("get_cursor_caps_mode: req_modes = {req_modes}");
        0
    }

    fn request_cursor_updates(&self, cursor_update_mode: i32) -> bool {
        debug!("request_cursor_updates: cursor_update_mode = {cursor_update_mode}");
        false
    }
}
