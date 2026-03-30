use std::path::{Path, PathBuf};

use geng::{
    asset::{Future, Load, Manager},
    prelude::{future::FutureExt, *},
};

pub struct MaybeHot<T: Load> {
    current: RefCell<T>,
    #[cfg(not(target_arch = "wasm32"))]
    hot: Option<Hot<T>>,
}

// NOTE: issue with watching file edits done by (neo)vim/helix
// <https://github.com/notify-rs/notify/issues/247>
#[cfg(not(target_arch = "wasm32"))]
struct Hot<T: Load> {
    manager: Manager,
    path: PathBuf,
    options: T::Options,
    need_update: Arc<std::sync::atomic::AtomicBool>,
    need_reload_watcher: Arc<std::sync::atomic::AtomicBool>,
    update: RefCell<Option<Future<T>>>,
    #[allow(dead_code)] // This is here for delaying the drop of the watcher
    file_watcher: RefCell<Option<notify::RecommendedWatcher>>,
    #[allow(dead_code)] // This is here for delaying the drop of the watcher
    dir_watcher: notify::RecommendedWatcher,
}

pub type Ref<'a, T> = std::cell::Ref<'a, T>;

impl<T: Load> MaybeHot<T> {
    pub fn load(manager: &Manager, path: &Path, options: &T::Options, hot: bool) -> Future<Self> {
        #[cfg(target_arch = "wasm32")]
        return Self::load_cold(manager, path, options);

        #[cfg(not(target_arch = "wasm32"))]
        if hot {
            Self::load_hot(manager, path, options)
        } else {
            Self::load_cold(manager, path, options)
        }
    }

    pub fn load_cold(manager: &Manager, path: &Path, options: &T::Options) -> Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();
        let options = options.clone();
        async move {
            let initial = manager.load_with(&path, &options).await?;
            Ok(Self {
                current: RefCell::new(initial),
                #[cfg(not(target_arch = "wasm32"))]
                hot: None,
            })
        }
        .boxed_local()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_hot(manager: &Manager, path: &Path, options: &T::Options) -> Future<Self> {
        use notify::Watcher;

        let manager = manager.clone();
        let path = path.to_owned();
        let options = options.clone();
        let need_update = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let need_reload_watcher = Arc::new(std::sync::atomic::AtomicBool::new(true));

        log::info!("watching {path:?}");
        let dir_watcher = {
            let need_reload_watcher = need_reload_watcher.clone();
            let mut watcher =
                notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
                    let event = result.unwrap();
                    if event.kind.is_create() {
                        need_reload_watcher.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                })
                .unwrap();
            watcher
                .watch(path.parent().unwrap(), notify::RecursiveMode::Recursive)
                .unwrap();
            watcher
        };

        async move {
            let initial = manager.load_with(&path, &options).await?;
            Ok(Self {
                current: RefCell::new(initial),
                hot: Some(Hot {
                    need_update,
                    need_reload_watcher,
                    options,
                    manager: manager.clone(),
                    path: path.to_owned(),
                    update: RefCell::new(None),
                    file_watcher: RefCell::new(None),
                    dir_watcher,
                }),
            })
        }
        .boxed_local()
    }

    pub fn get(&'_ self) -> Ref<'_, T> {
        if let Some(hot) = &self.hot
            && let Ok(mut current) = self.current.try_borrow_mut()
        {
            let mut update = hot.update.borrow_mut();
            if let Some(future) = &mut *update {
                // Wait for update to finish
                if let std::task::Poll::Ready(result) = future.as_mut().poll(
                    &mut std::task::Context::from_waker(futures::task::noop_waker_ref()),
                ) {
                    *update = None;
                    match result {
                        Ok(new) => *current = new,
                        Err(e) => log::error!("Hot reloading failed for {:?}: {}", hot.path, e),
                    }
                    hot.need_update
                        .store(false, std::sync::atomic::Ordering::SeqCst);
                }
            } else if hot.need_update.load(std::sync::atomic::Ordering::SeqCst) {
                // Reload
                log::debug!("Hot reloading: {:?}", hot.path);
                *update = Some(hot.manager.load_with(&hot.path, &hot.options).boxed_local())
            } else if hot
                .need_reload_watcher
                .load(std::sync::atomic::Ordering::SeqCst)
            {
                // Update file watcher
                let file_watcher = {
                    use notify::Watcher;
                    let need_update = hot.need_update.clone();
                    let mut watcher = notify::recommended_watcher(
                        move |result: notify::Result<notify::Event>| {
                            let event = result.unwrap();
                            if event.kind.is_modify() {
                                need_update.store(true, std::sync::atomic::Ordering::SeqCst);
                            }
                        },
                    )
                    .unwrap();
                    watcher
                        .watch(&hot.path, notify::RecursiveMode::Recursive)
                        .unwrap();
                    watcher
                };
                *hot.file_watcher.borrow_mut() = Some(file_watcher);
            }
        }
        self.current.borrow()
    }
}

impl<T: Load> Load for MaybeHot<T> {
    type Options = T::Options;
    fn load(manager: &Manager, path: &Path, options: &Self::Options) -> Future<Self> {
        Self::load_cold(manager, path, options)
    }
    const DEFAULT_EXT: Option<&'static str> = T::DEFAULT_EXT;
}
