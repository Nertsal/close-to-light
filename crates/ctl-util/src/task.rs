use std::future::Future;

use geng::prelude::{Geng, future::FutureExt};

pub struct Task<T> {
    inner: async_executor::Task<T>,
}

impl<T: 'static> Task<T> {
    pub fn new(geng: &Geng, future: impl Future<Output = T> + 'static) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let future = async_compat::Compat::new(future);

        Self {
            inner: geng.window().spawn(future),
        }
    }

    pub fn poll(self) -> Result<T, Self> {
        if !self.inner.is_finished() {
            return Err(self);
        }
        Ok(self.inner.now_or_never().unwrap())
    }
}
