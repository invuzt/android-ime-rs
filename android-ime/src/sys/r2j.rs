use crate::sys::j2r::*;
use jni::objects::{GlobalRef, JClass, JMethodID, JObject, JStaticMethodID, JValue};
use jni::signature::{Primitive, ReturnType};
use jni::{JNIEnv, JavaVM, NativeMethod};
use log::debug;
use parking_lot::{const_mutex, Mutex};
use std::os::raw::c_void;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////
pub struct JImeView {
    jni: Arc<JImeViewJni>,
    object: GlobalRef,
}

impl JImeView {
    ////////////////////////////////////////////////////////////////////////////////
    pub fn from<'a>(env: &mut JNIEnv<'a>, activity: &JObject<'a>) -> anyhow::Result<Self> {
        debug!("JImeView::from");

        let jni = JImeViewJni::load(env)?;
        let object = unsafe {
            env.call_static_method_unchecked(
                &jni.class,
                jni.m_from,
                ReturnType::Object,
                &[JValue::from(&activity).as_jni()],
            )?
        };

        let object = JObject::try_from(object)?;
        let object = env.new_global_ref(object)?;

        Ok(Self { jni, object })
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub fn activate<'a>(&self, id: u64) -> anyhow::Result<()> {
        debug!("JImeView::activate: id = {id}");

        let mut env = self.jni.java_vm.attach_current_thread()?;
        unsafe {
            env.call_method_unchecked(
                &self.object,
                self.jni.m_activate,
                ReturnType::Primitive(Primitive::Void),
                &[JValue::from(id as i64).as_jni()],
            )?
        };
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub fn deactivate<'a>(&self, id: u64) -> anyhow::Result<()> {
        debug!("JImeView::deactivate: id = {id}");

        let mut env = self.jni.java_vm.attach_current_thread()?;
        unsafe {
            env.call_method_unchecked(
                &self.object,
                self.jni.m_deactivate,
                ReturnType::Primitive(Primitive::Void),
                &[JValue::from(id as i64).as_jni()],
            )?
        };
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
struct JImeViewJni {
    java_vm: JavaVM,
    class: GlobalRef,
    m_from: JStaticMethodID,
    m_activate: JMethodID,
    m_deactivate: JMethodID,
}

impl JImeViewJni {
    ////////////////////////////////////////////////////////////////////////////////
    fn load(env: &mut JNIEnv) -> anyhow::Result<Arc<Self>> {
        static CELL: Mutex<Option<Arc<JImeViewJni>>> = const_mutex(None);

        let lock = &mut *CELL.lock();
        if let Some(value) = lock.as_ref() {
            return Ok(value.clone());
        }

        debug!("loading custom class loader");
        let class_loader = Self::load_classes_dex(env)?;

        debug!("get ImeView glue class");
        let c_ImeView_name = env.new_string("dev.matrix.rust.ime.glue.ImeView")?;
        let c_ImeView = env.call_method(
            class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&c_ImeView_name)],
        )?;

        debug!("c_ImeView = {c_ImeView:?}");
        let c_ImeView = JClass::from(c_ImeView.l()?);

        debug!("register ImeView native methods");
        Self::register_native_methods(env, &c_ImeView)?;

        debug!("loading ImeView::from method");
        let m_from = env.get_static_method_id(
            &c_ImeView,
            "from",
            "(Landroid/app/Activity;)Ldev/matrix/rust/ime/glue/ImeView;",
        )?;
        debug!("ImeView::from = {m_from:?}");

        debug!("loading ImeView::activate method");
        let m_activate = env.get_method_id(&c_ImeView, "activate", "(J)V")?;
        debug!("ImeView::activate = {m_activate:?}");

        debug!("loading ImeView::deactivate method");
        let m_deactivate = env.get_method_id(&c_ImeView, "deactivate", "(J)V")?;
        debug!("ImeView::deactivate = {m_deactivate:?}");

        let this = Self {
            java_vm: env.get_java_vm()?,
            class: env.new_global_ref(c_ImeView)?,
            m_from,
            m_activate,
            m_deactivate,
        };
        Ok(lock.insert(Arc::new(this)).clone())
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn load_classes_dex<'a>(env: &mut JNIEnv<'a>) -> anyhow::Result<JObject<'a>> {
        debug!("get current thread");
        let thread = env
            .call_static_method("java/lang/Thread", "currentThread", "()Ljava/lang/Thread;", &[])?
            .l()?;

        debug!("get current thread class loader");
        let parent_loader = env
            .call_method(thread, "getContextClassLoader", "()Ljava/lang/ClassLoader;", &[])?
            .l()?;

        debug!("creating classes.dex byte buffer");
        let dex_buffer = &mut Vec::from(include_bytes!("classes.dex"));
        let dex_buffer = unsafe { env.new_direct_byte_buffer(dex_buffer.as_mut_ptr(), dex_buffer.len())? };

        debug!("loading classes.dex");
        let c_DexClassLoader = env.find_class("dalvik/system/InMemoryDexClassLoader")?;
        let dex_class_loader = env.new_object(
            c_DexClassLoader,
            "(Ljava/nio/ByteBuffer;Ljava/lang/ClassLoader;)V",
            &[JValue::Object(&dex_buffer), JValue::Object(&parent_loader)],
        )?;

        Ok(dex_class_loader)
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn register_native_methods<'a>(env: &mut JNIEnv<'a>, class: &JClass<'a>) -> anyhow::Result<()> {
        macro_rules! native_method {
            ($name:literal, $sig:literal, $f:ident) => {
                NativeMethod {
                    name: $name.into(),
                    sig: $sig.into(),
                    fn_ptr: $f as *mut c_void,
                }
            };
        }

        let methods = [
            ////////////////////////////////////////////////////////////////////////////////
            // Session methods
            native_method! {
                "nativeConnectionClosed",
                "(J)V",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeConnectionClosed
            },
            native_method! {
                "nativeSendKeyEvent",
                "(JI)Z",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeSendKeyEvent
            },
            native_method! {
                "nativePerformContextMenuAction",
                "(JI)Z",
                Java_dev_matrix_rust_ime_glue_ImeView_nativePerformContextMenuAction
            },
            native_method! {
                "nativePerformEditorAction",
                "(JI)Z",
                Java_dev_matrix_rust_ime_glue_ImeView_nativePerformEditorAction
            },
            ////////////////////////////////////////////////////////////////////////////////
            // Text editing methods
            native_method! {
                "nativeCommitText",
                "(JLjava/lang/String;I)Z",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeCommitText
            },
            native_method! {
                "nativeDeleteSurroundingText",
                "(JII)Z",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeDeleteSurroundingText
            },
            native_method! {
                "nativeDeleteSurroundingTextInCodePoints",
                "(JII)Z",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeDeleteSurroundingTextInCodePoints
            },
            native_method! {
                "nativeSetComposingRegion",
                "(JII)Z",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeSetComposingRegion
            },
            native_method! {
                "nativeSetComposingText",
                "(JLjava/lang/String;I)Z",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeSetComposingText
            },
            native_method! {
                "nativeSetSelection",
                "(JII)Z",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeSetSelection
            },
            native_method! {
                "nativeFinishComposingText",
                "(J)Z",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeFinishComposingText
            },
            ////////////////////////////////////////////////////////////////////////////////
            // Text getter methods
            native_method! {
                "nativeGetSelectedText",
                "(JI)Ljava/lang/String;",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeGetSelectedText
            },
            native_method! {
                "nativeGetTextAfterCursor",
                "(JII)Ljava/lang/String;",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeGetTextAfterCursor
            },
            native_method! {
                "nativeGetTextBeforeCursor",
                "(JII)Ljava/lang/String;",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeGetTextBeforeCursor
            },
            native_method! {
                "nativeGetCursorCapsMode",
                "(JI)I",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeGetCursorCapsMode
            },
            native_method! {
                "nativeRequestCursorUpdates",
                "(JI)Z",
                Java_dev_matrix_rust_ime_glue_ImeView_nativeRequestCursorUpdates
            },
        ];

        Ok(env.register_native_methods(class, &methods)?)
    }
}
