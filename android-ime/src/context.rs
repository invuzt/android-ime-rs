use crate::sys::JImeView;
use jni::objects::JObject;
use jni::JNIEnv;

////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct AndroidImeContext {
    pub(crate) view: JImeView,
}

impl AndroidImeContext {
    pub fn new<'a>(env: &mut JNIEnv<'a>, activity: &JObject<'a>) -> anyhow::Result<Self> {
        Ok(Self {
            view: JImeView::from(env, activity)?,
        })
    }
}
