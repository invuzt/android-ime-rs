use crate::connection::{find_handler, AndroidImeConnectionHandler};
use jni::objects::*;
use jni::sys::{jboolean, jint, jlong, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeConnectionClosed<'a>(
    _env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
) {
    with_handler_t(id, (), |handler| Ok(handler.connection_closed()))
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeSendKeyEvent<'a>(
    _env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    keyCode: jint,
) -> jboolean {
    with_handler(id, |handler| Ok(handler.send_key_event(keyCode)))
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativePerformContextMenuAction<'a>(
    _env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    actionId: jint,
) -> jboolean {
    with_handler(id, |handler| Ok(handler.perform_context_menu_action(actionId)))
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativePerformEditorAction<'a>(
    _env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    editorAction: jint,
) -> jboolean {
    with_handler(id, |handler| Ok(handler.perform_editor_action(editorAction)))
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeCommitText<'a>(
    env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    text: JString<'a>,
    new_cursor_position: jint,
) -> jboolean {
    with_handler(id, |handler| {
        let text = unsafe { env.get_string_unchecked(&text)? };
        let text = text.to_string_lossy();
        Ok(handler.commit_text(text.as_ref(), new_cursor_position))
    })
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeDeleteSurroundingText<'a>(
    _env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    before: jint,
    after: jint,
) -> jboolean {
    with_handler(id, |handler| {
        let before = usize::try_from(before)?;
        let after = usize::try_from(after)?;
        Ok(handler.delete_surrounding_text(before, after))
    })
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeDeleteSurroundingTextInCodePoints<'a>(
    _env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    before: jint,
    after: jint,
) -> jboolean {
    with_handler(id, |handler| {
        let before = usize::try_from(before)?;
        let after = usize::try_from(after)?;
        Ok(handler.delete_surrounding_text_in_code_points(before, after))
    })
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeSetSelection<'a>(
    _env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    start: jint,
    end: jint,
) -> jboolean {
    with_handler(id, |handler| {
        let start = usize::try_from(start)?;
        let end = usize::try_from(end)?;
        Ok(handler.set_selection(start, end))
    })
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeSetComposingRegion<'a>(
    _env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    start: jint,
    end: jint,
) -> jboolean {
    with_handler(id, |handler| {
        let start = usize::try_from(start)?;
        let end = usize::try_from(end)?;
        Ok(handler.set_composing_region(start, end))
    })
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeSetComposingText<'a>(
    env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    text: JString<'a>,
    new_cursor_position: jint,
) -> jboolean {
    with_handler(id, |handler| {
        let text = unsafe { env.get_string_unchecked(&text)? };
        let text = text.to_string_lossy();
        Ok(handler.set_composing_text(text.as_ref(), new_cursor_position))
    })
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeFinishComposingText<'a>(
    _env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
) -> jboolean {
    with_handler(id, |handler| Ok(handler.finish_composing_text()))
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeGetSelectedText<'a>(
    env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    flags: jint,
) -> jstring {
    with_handler_t(id, std::ptr::null_mut(), |handler| {
        let Some(text) = handler.get_selected_text(flags) else {
            return Ok(std::ptr::null_mut());
        };
        Ok(env.new_string(text)?.into_raw())
    })
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeGetTextAfterCursor<'a>(
    env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    n: jint,
    flags: jint,
) -> jstring {
    with_handler_t(id, std::ptr::null_mut(), |handler| {
        let n = usize::try_from(n)?;
        let Some(text) = handler.get_text_after_cursor(n, flags) else {
            return Ok(std::ptr::null_mut());
        };
        Ok(env.new_string(text)?.into_raw())
    })
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeGetTextBeforeCursor<'a>(
    env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    n: jint,
    flags: jint,
) -> jstring {
    with_handler_t(id, std::ptr::null_mut(), |handler| {
        let n = usize::try_from(n)?;
        let Some(text) = handler.get_text_before_cursor(n, flags) else {
            return Ok(std::ptr::null_mut());
        };
        Ok(env.new_string(text)?.into_raw())
    })
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeGetCursorCapsMode<'a>(
    _env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    reqModes: jint,
) -> jint {
    with_handler_t(id, 0, |handler| Ok(handler.get_cursor_caps_mode(reqModes)))
}

////////////////////////////////////////////////////////////////////////////////
#[unsafe(no_mangle)]
pub extern "C" fn Java_dev_matrix_rust_ime_glue_ImeView_nativeRequestCursorUpdates<'a>(
    _env: JNIEnv<'a>,
    _object: JObject<'a>,
    id: jlong,
    cursorUpdateMode: jint,
) -> jboolean {
    with_handler(id, |handler| Ok(handler.request_cursor_updates(cursorUpdateMode)))
}

////////////////////////////////////////////////////////////////////////////////
fn with_handler<F>(id: jlong, f: F) -> jboolean
where
    F: FnOnce(&dyn AndroidImeConnectionHandler) -> anyhow::Result<bool>,
{
    with_handler_t(id, JNI_FALSE, |handler| {
        Ok(match f(handler)? {
            true => JNI_TRUE,
            false => JNI_FALSE,
        })
    })
}

////////////////////////////////////////////////////////////////////////////////
fn with_handler_t<F, T>(id: jlong, fallback: T, f: F) -> T
where
    F: FnOnce(&dyn AndroidImeConnectionHandler) -> anyhow::Result<T>,
{
    let id = u64::from_ne_bytes(id.to_ne_bytes());
    match find_handler(id) {
        None => fallback,
        Some(e) => f(&*e).unwrap_or(fallback),
    }
}
