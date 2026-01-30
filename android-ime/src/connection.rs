use crate::context::AndroidImeContext;
use crate::handler::registry::AndroidImeConnectionRegistry;
use crate::handler::AndroidImeConnectionHandler;
use log::error;
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////
pub struct AndroidImeConnection<T> {
    id: u64,
    handler: Arc<T>,
    context: Option<AndroidImeContext>,
}

impl<T> AndroidImeConnection<T> {
    pub fn open<'a>(context: &AndroidImeContext, handler: T) -> anyhow::Result<Self>
    where
        T: AndroidImeConnectionHandler,
    {
        static ID: AtomicU64 = AtomicU64::new(0);

        let id = ID.fetch_add(1, Ordering::Relaxed);
        let handler = Arc::new(handler);
        AndroidImeConnectionRegistry::get().register(id, handler.clone());

        context.view.activate(id)?;

        Ok(AndroidImeConnection {
            id,
            handler,
            context: Some(context.clone()),
        })
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn detach(mut self) -> anyhow::Result<()> {
        self.detach_private()
    }

    fn detach_private(&mut self) -> anyhow::Result<()> {
        let Some(context) = self.context.take() else {
            return Ok(());
        };

        AndroidImeConnectionRegistry::get().unregister(self.id);

        context.view.deactivate(self.id)
    }
}

impl<T> Drop for AndroidImeConnection<T> {
    fn drop(&mut self) {
        if let Err(e) = self.detach_private() {
            error!("failed to deactivate IME connection: {e:?}");
        }
    }
}

impl<T> Deref for AndroidImeConnection<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.handler
    }
}
