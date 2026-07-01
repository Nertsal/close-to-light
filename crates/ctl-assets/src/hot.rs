use std::path::{Path, PathBuf};

use ctl_core::types::Name;
use geng::{
    asset::{Future, Load, Manager},
    prelude::{future::FutureExt, *},
};

// pub struct MaybeHot<T: Load> {
//     current: RefCell<T>,
//     #[cfg(not(target_arch = "wasm32"))]
//     hot: Option<Hot<T>>,
// }

// // NOTE: issue with watching file edits done by (neo)vim/helix
// // <https://github.com/notify-rs/notify/issues/247>
// #[cfg(not(target_arch = "wasm32"))]
// struct Hot<T: Load> {
//     manager: Manager,
//     path: PathBuf,
//     options: T::Options,
//     need_update: Arc<std::sync::atomic::AtomicBool>,
//     need_reload_watcher: Arc<std::sync::atomic::AtomicBool>,
//     update: RefCell<Option<Future<T>>>,
//     #[allow(dead_code)] // This is here for delaying the drop of the watcher
//     file_watcher: RefCell<Option<notify::RecommendedWatcher>>,
//     #[allow(dead_code)] // This is here for delaying the drop of the watcher
//     dir_watcher: notify::RecommendedWatcher,
// }

// pub type Ref<'a, T> = std::cell::Ref<'a, T>;

// impl<T: Load> MaybeHot<T> {
//     pub fn load(manager: &Manager, path: &Path, options: &T::Options, hot: bool) -> Future<Self> {
//         #[cfg(target_arch = "wasm32")]
//         return Self::load_cold(manager, path, options);

//         #[cfg(not(target_arch = "wasm32"))]
//         if hot {
//             Self::load_hot(manager, path, options)
//         } else {
//             Self::load_cold(manager, path, options)
//         }
//     }

//     pub fn load_cold(manager: &Manager, path: &Path, options: &T::Options) -> Future<Self> {
//         let manager = manager.clone();
//         let path = path.to_owned();
//         let options = options.clone();
//         async move {
//             let initial = manager.load_with(&path, &options).await?;
//             Ok(Self {
//                 current: RefCell::new(initial),
//                 #[cfg(not(target_arch = "wasm32"))]
//                 hot: None,
//             })
//         }
//         .boxed_local()
//     }

//     #[cfg(not(target_arch = "wasm32"))]
//     pub fn load_hot(manager: &Manager, path: &Path, options: &T::Options) -> Future<Self> {
//         use notify::Watcher;

//         let manager = manager.clone();
//         let path = path.to_owned();
//         let options = options.clone();
//         let need_update = Arc::new(std::sync::atomic::AtomicBool::new(false));
//         let need_reload_watcher = Arc::new(std::sync::atomic::AtomicBool::new(true));

//         log::info!("watching {path:?}");
//         let dir_watcher = {
//             let need_reload_watcher = need_reload_watcher.clone();
//             let mut watcher =
//                 notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
//                     let event = result.unwrap();
//                     if event.kind.is_create() {
//                         need_reload_watcher.store(true, std::sync::atomic::Ordering::SeqCst);
//                     }
//                 })
//                 .unwrap();
//             watcher
//                 .watch(path.parent().unwrap(), notify::RecursiveMode::Recursive)
//                 .unwrap();
//             watcher
//         };

//         async move {
//             let initial = manager.load_with(&path, &options).await?;
//             Ok(Self {
//                 current: RefCell::new(initial),
//                 hot: Some(Hot {
//                     need_update,
//                     need_reload_watcher,
//                     options,
//                     manager: manager.clone(),
//                     path: path.to_owned(),
//                     update: RefCell::new(None),
//                     file_watcher: RefCell::new(None),
//                     dir_watcher,
//                 }),
//             })
//         }
//         .boxed_local()
//     }

//     pub fn get(&'_ self) -> Ref<'_, T> {
//         #[cfg(not(target_arch = "wasm32"))]
//         if let Some(hot) = &self.hot
//             && let Ok(mut current) = self.current.try_borrow_mut()
//         {
//             let mut update = hot.update.borrow_mut();
//             if let Some(future) = &mut *update {
//                 // Wait for update to finish
//                 if let std::task::Poll::Ready(result) = future.as_mut().poll(
//                     &mut std::task::Context::from_waker(futures::task::noop_waker_ref()),
//                 ) {
//                     *update = None;
//                     match result {
//                         Ok(new) => *current = new,
//                         Err(e) => log::error!("Hot reloading failed for {:?}: {}", hot.path, e),
//                     }
//                     hot.need_update
//                         .store(false, std::sync::atomic::Ordering::SeqCst);
//                 }
//             } else if hot.need_update.load(std::sync::atomic::Ordering::SeqCst) {
//                 // Reload
//                 log::debug!("Hot reloading: {:?}", hot.path);
//                 *update = Some(hot.manager.load_with(&hot.path, &hot.options).boxed_local())
//             } else if hot
//                 .need_reload_watcher
//                 .load(std::sync::atomic::Ordering::SeqCst)
//             {
//                 // Update file watcher
//                 let file_watcher = {
//                     use notify::Watcher;
//                     let need_update = hot.need_update.clone();
//                     let mut watcher = notify::recommended_watcher(
//                         move |result: notify::Result<notify::Event>| {
//                             let event = result.unwrap();
//                             if event.kind.is_modify() {
//                                 need_update.store(true, std::sync::atomic::Ordering::SeqCst);
//                             }
//                         },
//                     )
//                     .unwrap();
//                     watcher
//                         .watch(&hot.path, notify::RecursiveMode::Recursive)
//                         .unwrap();
//                     watcher
//                 };
//                 *hot.file_watcher.borrow_mut() = Some(file_watcher);
//             }
//         }
//         self.current.borrow()
//     }

//     pub fn freeze(&self) -> Self
//     where
//         T: Clone,
//     {
//         Self {
//             current: RefCell::new(self.get().clone()),
//             #[cfg(not(target_arch = "wasm32"))]
//             hot: None,
//         }
//     }
// }

// impl<T: Load> Load for MaybeHot<T> {
//     type Options = T::Options;
//     fn load(manager: &Manager, path: &Path, options: &Self::Options) -> Future<Self> {
//         Self::load_cold(manager, path, options)
//     }
//     const DEFAULT_EXT: Option<&'static str> = T::DEFAULT_EXT;
// }

pub struct MaybeHotDir<T: Load> {
    current: RefCell<HashMap<Name, T>>,
    #[cfg(not(target_arch = "wasm32"))]
    hot: Option<HotDir<T>>,
}

impl<T: Load> Default for MaybeHotDir<T> {
    fn default() -> Self {
        Self {
            current: RefCell::new(HashMap::new()),
            #[cfg(not(target_arch = "wasm32"))]
            hot: None,
        }
    }
}

// NOTE: issue with watching file edits done by (neo)vim/helix
// <https://github.com/notify-rs/notify/issues/247>
#[cfg(not(target_arch = "wasm32"))]
struct HotDir<T: Load> {
    manager: Manager,
    path: PathBuf,
    options: T::Options,
    need_remove: Arc<Mutex<Option<Name>>>,
    need_reload: Arc<Mutex<Vec<PathBuf>>>,
    // need_reload_watcher: Arc<std::sync::atomic::AtomicBool>,
    update: RefCell<Option<(Name, Future<T>)>>,
    #[allow(dead_code)] // This is here for delaying the drop of the watcher
    dir_watcher: notify::RecommendedWatcher,
}

// pub type Ref<'a, T> = std::cell::Ref<'a, T>;

impl<T: Load> MaybeHotDir<T> {
    pub fn load(manager: &Manager, path: &Path, options: &T::Options, hot: bool) -> Future<Self> {
        Self::load_list(manager, path, Vec::new(), options, hot)
    }

    pub fn load_list(
        manager: &Manager,
        path: &Path,
        files: Vec<PathBuf>,
        options: &T::Options,
        hot: bool,
    ) -> Future<Self> {
        #[cfg(target_arch = "wasm32")]
        return Self::load_cold(manager, path, files, options);

        #[cfg(not(target_arch = "wasm32"))]
        if hot {
            Self::load_hot(manager, path, files, options)
        } else {
            Self::load_cold(manager, path, files, options)
        }
    }

    pub fn load_cold(
        manager: &Manager,
        path: &Path,
        files: Vec<PathBuf>,
        options: &T::Options,
    ) -> Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();
        let options = options.clone();

        async move {
            #[cfg(not(target_arch = "wasm32"))]
            let mut initial_paths: Vec<PathBuf> = std::fs::read_dir(&path)?
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<Result<Vec<_>, _>>()?;
            #[cfg(target_arch = "wasm32")]
            let mut initial_paths = vec![];

            initial_paths.extend(files);

            let mut initial = HashMap::new();
            for path in initial_paths {
                let value = manager
                    .load_with(&path, &options)
                    .await
                    .context(format!("failed to load asset at {:?}", path))?;
                if let Some(name) = path_to_name(path) {
                    initial.insert(name, value);
                }
            }

            Ok(Self {
                current: RefCell::new(initial),
                #[cfg(not(target_arch = "wasm32"))]
                hot: None,
            })
        }
        .boxed_local()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_hot(
        manager: &Manager,
        path: &Path,
        files: Vec<PathBuf>,
        options: &T::Options,
    ) -> Future<Self> {
        use notify::Watcher;

        let manager = manager.clone();
        let path = path.to_owned();
        let options = options.clone();
        let need_reload = Arc::new(Mutex::new(Vec::new()));
        let need_remove = Arc::new(Mutex::new(None));

        log::info!("watching {path:?}");
        let dir_watcher = {
            let need_reload = Arc::clone(&need_reload);
            let mut watcher =
                notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
                    let event = result.unwrap();
                    if event.kind.is_create() || event.kind.is_modify() {
                        // NOTE: in case of renaming, only the last path is relevant
                        if let Some(path) = event.paths.into_iter().next_back()
                            && let Ok(mut lock) = need_reload.lock()
                        {
                            lock.push(path);
                        }
                    } else if event.kind.is_remove() {
                        // TODO
                    }
                })
                .unwrap();
            watcher
                .watch(&path, notify::RecursiveMode::Recursive)
                .unwrap();
            watcher
        };

        async move {
            #[cfg(not(target_arch = "wasm32"))]
            let mut initial_paths: Vec<PathBuf> = std::fs::read_dir(&path)?
                .flat_map(|entry| entry.map(|entry| entry.path()))
                .collect::<Vec<_>>();
            #[cfg(target_arch = "wasm32")]
            let mut initial_paths = vec![];

            initial_paths.extend(files);

            let mut initial = HashMap::new();
            for path in initial_paths {
                let value = manager
                    .load_with(&path, &options)
                    .await
                    .context(format!("failed to load hot asset at {:?}", path))?;
                if let Some(name) = path_to_name(path) {
                    dbg!(&name);
                    initial.insert(name, value);
                }
            }

            Ok(Self {
                current: RefCell::new(initial),
                hot: Some(HotDir {
                    need_reload,
                    need_remove,
                    options,
                    manager: manager.clone(),
                    path: path.to_owned(),
                    update: RefCell::new(None),
                    dir_watcher,
                }),
            })
        }
        .boxed_local()
    }

    pub fn get(&'_ self) -> Ref<'_, HashMap<Name, T>> {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(hot) = &self.hot
            && let Ok(mut current) = self.current.try_borrow_mut()
        {
            let mut update = hot.update.borrow_mut();
            if let Some((name, future)) = &mut *update {
                // Wait for update to finish
                if let std::task::Poll::Ready(result) = future.as_mut().poll(
                    &mut std::task::Context::from_waker(futures::task::noop_waker_ref()),
                ) {
                    let name = name.clone();
                    *update = None;
                    match result {
                        Ok(new) => {
                            current.insert(name, new);
                        }
                        Err(e) => log::error!("Hot reloading failed for {:?}: {}", hot.path, e),
                    }
                }
            } else if let Ok(mut lock) = hot.need_reload.lock()
                && let Some(path) = lock.pop()
                && let Some(name) = path_to_name(path.clone())
            {
                // Reload
                log::debug!("Hot reloading: {:?}", path);
                let manager = hot.manager.clone();
                let options = hot.options.clone();
                *update = Some((
                    name,
                    async move {
                        let value = manager.load_with(&path, &options).await?;
                        Ok(value)
                    }
                    .boxed_local(),
                ))
            }
        }
        self.current.borrow()
    }

    pub fn freeze(&self) -> Self
    where
        T: Clone,
    {
        Self {
            current: RefCell::new(self.get().clone()),
            #[cfg(not(target_arch = "wasm32"))]
            hot: None,
        }
    }
}

impl<T: Load> Load for MaybeHotDir<T> {
    type Options = T::Options;
    fn load(manager: &Manager, path: &Path, options: &Self::Options) -> Future<Self> {
        Self::load_cold(manager, path, vec![], options)
    }
    const DEFAULT_EXT: Option<&'static str> = T::DEFAULT_EXT;
}

pub(super) fn path_to_name(mut path: PathBuf) -> Option<Name> {
    dbg!("path to name", &path);
    path.set_extension("");
    if let Some(name) = path.file_name()
        && let Some(name) = name.to_str()
    {
        Some(Name::from(name))
    } else {
        None
    }
}
