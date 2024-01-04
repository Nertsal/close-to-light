use std::future::Future;
#[cfg(not(target_arch = "wasm32"))]
use std::thread::JoinHandle;

use geng::prelude::anyhow;
#[cfg(target_arch = "wasm32")]
use geng::prelude::{
    futures::{self, FutureExt},
    Pin,
};

/// Abstraction over an asynchronous task being executed.
pub struct Task<T> {
    #[cfg(target_arch = "wasm32")]
    future: Option<Pin<Box<dyn Future<Output = T>>>>,
    #[cfg(not(target_arch = "wasm32"))]
    handle: Option<JoinHandle<T>>,
}

#[cfg(target_arch = "wasm32")]
impl<T> Task<T> {
    pub fn new(future: impl Future<Output = T> + 'static) -> Self {
        Self {
            future: Some(future.boxed_local()),
        }
    }

    /// Attempt to fetch the result of the task.
    /// On web: polls a future.
    /// Natively checks if a thread has finished execution (non-blocking).
    pub fn poll(&mut self) -> Option<anyhow::Result<T>> {
        if let Some(future) = &mut self.future {
            if let std::task::Poll::Ready(value) = future.as_mut().poll(
                &mut std::task::Context::from_waker(futures::task::noop_waker_ref()),
            ) {
                self.future = None;
                return Some(Ok(value));
            }
        }
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + Sync + 'static> Task<T> {
    pub fn new(future: impl Future<Output = T> + Send + Sync + 'static) -> Self {
        // Spawn a tokio runtime in a new thread
        let handle = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(future)
        });
        Self {
            handle: Some(handle),
        }
    }

    /// Attempt to fetch the result of the task.
    /// On web: polls a future.
    /// Natively checks if a thread has finished execution (non-blocking).
    pub fn poll(&mut self) -> Option<anyhow::Result<T>> {
        if let Some(handle) = self.handle.take() {
            if handle.is_finished() {
                match handle.join() {
                    Ok(value) => {
                        return Some(Ok(value));
                    }
                    Err(_) => {
                        return Some(Err(anyhow::Error::msg("joining a thread has failed")));
                    }
                }
            } else {
                self.handle = Some(handle);
            }
        }
        None
    }
}
