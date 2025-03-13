use super::*;

use crate::task::Task;

use ctl_client::Nertboard;
use generational_arena::Index;

type TaskRes<T> = Option<Task<anyhow::Result<T>>>;

pub struct LevelCache {
    geng: Geng,
    pub inner: RefCell<LevelCacheImpl>,
    fs: Rc<fs::Controller>,
}

pub struct LevelCacheImpl {
    tasks: CacheTasks,

    /// List of downloadable level groups.
    pub group_list: CacheState<Vec<GroupInfo>>,

    pub groups: Arena<Rc<CachedGroup>>,

    pub notifications: Vec<String>,
}

pub enum CacheState<T> {
    Offline,
    Loading,
    Loaded(T),
}

pub struct CacheTasks {
    client: Option<Arc<Nertboard>>,

    fs: VecDeque<Task<anyhow::Result<()>>>,

    fetch_groups: TaskRes<Vec<GroupInfo>>,
    // downloading_groups: HashSet<Id>,
    download_group: VecDeque<(Id, Task<Result<CachedGroup>>)>,
    get_recommended: TaskRes<Vec<GroupInfo>>,

    notifications: Vec<String>,
}

enum CacheAction {
    GroupList(Vec<GroupInfo>),
    Group(CachedGroup),
    DownloadGroups(Vec<Id>),
}

impl CacheTasks {
    pub fn new(client: Option<&Arc<Nertboard>>) -> Self {
        Self {
            client: client.cloned(),

            fs: VecDeque::new(),
            fetch_groups: None,
            // downloading_groups: HashSet::new(),
            download_group: VecDeque::new(),
            get_recommended: None,

            notifications: Vec::new(),
        }
    }

    fn poll(&mut self) -> Option<CacheAction> {
        macro_rules! error {
            ($pat:literal, $($arg:expr),*) => {{
                let message = format!($pat, $($arg),*);
                self.notifications.push(message);
            }}
        }

        if let Some(task) = self.fs.pop_front() {
            match task.poll() {
                Err(task) => self.fs.push_front(task),
                Ok(Err(err)) => error!("File system task failed: {:?}", err),
                Ok(Ok(())) => {}
            }
        } else if let Some(task) = self.fetch_groups.take() {
            match task.poll() {
                Err(task) => self.fetch_groups = Some(task),
                Ok(result) => {
                    if let Ok(groups) = result {
                        return Some(CacheAction::GroupList(groups));
                    }
                }
            }
        } else if let Some(task) = self.get_recommended.take() {
            match task.poll() {
                Err(task) => self.get_recommended = Some(task),
                Ok(Err(err)) => error!("Failed to get recommended levels: {:?}", err),
                Ok(Ok(groups)) => {
                    let group_ids = groups.into_iter().map(|group| group.id).collect();
                    return Some(CacheAction::DownloadGroups(group_ids));
                }
            }
        } else if let Some((group_id, task)) = self.download_group.pop_front() {
            match task.poll() {
                Err(task) => self.download_group.push_front((group_id, task)),
                Ok(Err(err)) => {
                    error!("Failed to download group {}: {:?}", group_id, err);
                }
                Ok(Ok(group)) => {
                    log::debug!("downloaded group {}", group_id);
                    return Some(CacheAction::Group(group));
                }
            }
        }

        None
    }
}

impl LevelCache {
    pub async fn new(client: Option<&Arc<Nertboard>>, geng: &Geng) -> Self {
        let inner = LevelCacheImpl {
            tasks: CacheTasks::new(client),

            group_list: CacheState::Offline,

            groups: Arena::new(),

            notifications: Vec::new(),
        };
        Self {
            geng: geng.clone(),
            inner: RefCell::new(inner),
            fs: Rc::new(
                fs::Controller::new(geng)
                    .await
                    .expect("failed to init file system"),
            ),
        }
    }

    pub fn take_notifications(&self) -> Vec<String> {
        let inner = &mut *self.inner.borrow_mut();
        inner.notifications.append(&mut inner.tasks.notifications);
        std::mem::take(&mut inner.notifications)
    }

    pub fn client(&self) -> Option<Arc<Nertboard>> {
        let inner = self.inner.borrow();
        inner.tasks.client.as_ref().cloned()
    }

    /// Load from the local storage.
    pub async fn load(client: Option<&Arc<Nertboard>>, geng: &Geng) -> Result<Self> {
        log::info!("Loading local storage");
        let timer = Timer::new();
        let local = Self::new(client, geng).await;

        local.load_all().await?;

        log::debug!("Loaded cache in {:.2}s", timer.elapsed().as_secs_f64());

        Ok(local)
    }

    async fn load_all(&self) -> Result<()> {
        let groups = self.fs.load_groups_all().await?;
        let group_loaders = groups.into_iter().map(|local| self.insert_group(local));

        let groups = future::join_all(group_loaders).await;
        let mut inner = self.inner.borrow_mut();
        for group in groups.into_iter().flatten() {
            if group.local.music.is_none() {
                log::warn!("group {:?} loaded without music", group.local.path);
            }
            inner.groups.insert(Rc::new(group));
        }
        log::debug!("loaded groups: {}", inner.groups.len());

        Ok(())
    }

    pub fn is_downloading_group(&self) -> Vec<Id> {
        let inner = self.inner.borrow();
        inner
            .tasks
            .download_group
            .iter()
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn get_group(&self, group: Index) -> Option<Rc<CachedGroup>> {
        let inner = self.inner.borrow();
        inner.groups.get(group).cloned()
    }

    pub fn get_group_id(&self, group_id: Id) -> Option<(Index, Rc<CachedGroup>)> {
        let inner = self.inner.borrow();
        inner
            .groups
            .iter()
            .find(|(_, group)| group.local.data.id == group_id)
            .map(|(idx, group)| (idx, Rc::clone(group)))
    }

    pub fn get_level(&self, group: Index, level: usize) -> Option<Rc<LevelFull>> {
        let inner = self.inner.borrow();
        inner
            .groups
            .get(group)
            .and_then(|group| group.local.data.levels.get(level))
            .cloned()
    }

    async fn insert_group(&self, group: LocalGroup) -> Result<CachedGroup> {
        let result = async {
            let hash = group.data.calculate_hash();

            let origin = if group.data.id == 0 {
                None
            } else if let Some(client) = self.client() {
                match client.get_group_info(group.data.id).await {
                    Err(err) => {
                        log::error!("failed to check group info: {:?}", err);
                        None
                    }
                    Ok(info) => Some(info),
                }
            } else {
                None
            };

            let level_hashes = group
                .data
                .levels
                .iter()
                .map(|level| level.data.calculate_hash())
                .collect();

            let group = CachedGroup {
                local: group,
                hash,
                origin,
                level_hashes,
            };

            Ok(group)
        }
        .await;

        if let Err(err) = &result {
            log::error!("failed to load group: {:?}", err);
        }
        result
    }

    fn save_group(&self, group: &Rc<CachedGroup>) {
        let mut inner = self.inner.borrow_mut();
        let future = {
            let fs = self.fs.clone();
            let group = group.clone();
            async move {
                fs.save_group(&group).await?;
                Ok(())
            }
        };
        inner.tasks.fs.push_back(Task::new(&self.geng, future));
    }

    // Dont use paths because the actual structure is flat
    fn move_group(&self, group: &Rc<CachedGroup>, old_path: impl AsRef<Path>) {
        let mut inner = self.inner.borrow_mut();
        let future = {
            let fs = self.fs.clone();
            let group = group.clone();
            let old_path = old_path.as_ref().to_owned();
            async move {
                fs.save_group(&group).await?;
                if old_path != group.local.path {
                    fs.remove_group(old_path).await?;
                }
                Ok(())
            }
        };
        inner.tasks.fs.push_back(Task::new(&self.geng, future));
    }

    fn remove_group(&self, path: impl AsRef<Path>) {
        let mut inner = self.inner.borrow_mut();
        let future = {
            let fs = self.fs.clone();
            let path = path.as_ref().to_owned();
            async move {
                fs.remove_group(path).await?;
                Ok(())
            }
        };
        inner.tasks.fs.push_back(Task::new(&self.geng, future));
    }

    pub fn new_group(&self, music: Option<Rc<LocalMusic>>) -> Index {
        let inner = self.inner.borrow();

        // Generate a non-occupied path
        let path = loop {
            let path = fs::generate_group_path(0);
            if !inner
                .groups
                .iter()
                .any(|(_, group)| group.local.path == path)
            {
                break path;
            }
        };

        let data = LevelSet {
            id: 0,
            // TODO: set the logged in user
            owner: UserInfo {
                id: 0,
                name: "".into(),
            },
            levels: Vec::new(),
        };
        let group = CachedGroup {
            hash: data.calculate_hash(),
            origin: None,
            level_hashes: vec![],
            local: LocalGroup {
                path,
                meta: GroupMeta {
                    music: music.as_ref().map(|music| music.meta.clone()),
                },
                music,
                data,
            },
        };

        // Write to fs
        drop(inner);
        let group = Rc::new(group);
        self.save_group(&group);

        self.inner.borrow_mut().groups.insert(group)
    }

    pub fn fetch_groups(&self) {
        let mut inner = self.inner.borrow_mut();
        if inner.tasks.fetch_groups.is_none() {
            if let Some(client) = inner.tasks.client.clone() {
                let future = async move {
                    let groups = client
                        .get_group_list(&GroupsQuery { recommended: false })
                        .await?;
                    Ok(groups)
                };
                inner.tasks.fetch_groups = Some(Task::new(&self.geng, future));
                inner.group_list = CacheState::Loading;
            }
        }
    }

    pub fn download_recommended(&self) {
        let mut inner = self.inner.borrow_mut();
        if inner.tasks.get_recommended.is_none() {
            if let Some(client) = inner.tasks.client.clone() {
                let future = async move {
                    let list = client
                        .get_group_list(&GroupsQuery { recommended: true })
                        .await?;
                    Ok(list)
                };
                inner.tasks.get_recommended = Some(Task::new(&self.geng, future));
            }
        }
    }

    pub fn download_group(&self, group_id: Id) {
        // let mut inner = self.inner.borrow_mut();
        // if inner
        //     .groups
        //     .iter()
        //     .any(|(_, group)| group_id == group.local.data.id)
        // {
        //     // Already downloaded
        //     // TODO: check version
        //     return;
        // }

        // if let Some(client) = inner.tasks.client.clone() {
        //     let geng = self.geng.clone();
        //     let fs = self.fs.clone();
        //     let music_list = inner.music.clone();

        //     let future = {
        //         async move {
        //             log::debug!("Downloading group {}", group_id);

        //             // Download group
        //             let info = client.get_group_info(group_id).await?;
        //             let bytes = client.download_group(group_id).await?.to_vec();
        //             let hash = ctl_client::core::util::calculate_hash(&bytes);
        //             // let data: LevelSet = bincode::deserialize(&bytes)?;
        //             let data: LevelSet = cbor4ii::serde::from_slice(&bytes)?;

        //             // Download music
        //             let music = match music_list.get(&data.music) {
        //                 Some(music) => Rc::clone(music),
        //                 None => {
        //                     log::debug!("Downloading music {}", data.music);
        //                     // Music is not local so we need to download it
        //                     let meta = client.get_music_info(data.music).await?;
        //                     let bytes = client.download_music(data.music).await?.to_vec();

        //                     log::debug!("Decoding downloaded music bytes");
        //                     let music = geng.audio().decode(bytes.clone()).await?;
        //                     let music = LocalMusic::new(meta, music);

        //                     if let Err(err) = fs.save_music(&music, &bytes).await {
        //                         log::error!("Failed to save music locally: {:?}", err);
        //                     }

        //                     Rc::new(music)
        //                 }
        //             };

        //             let level_hashes = data
        //                 .levels
        //                 .iter()
        //                 .map(|level| level.data.calculate_hash())
        //                 .collect();

        //             let group = CachedGroup {
        //                 path: fs::generate_group_path(data.id),
        //                 music: Some(music),
        //                 data,
        //                 origin: Some(info),
        //                 hash,
        //                 level_hashes,
        //             };

        //             // Write to fs
        //             if let Err(err) = fs.save_group(&group).await {
        //                 log::error!("Failed to save group locally: {:?}", err);
        //             }

        //             Ok(group)
        //         }
        //     };
        //     inner
        //         .tasks
        //         .download_group
        //         .push_back((group_id, Task::new(&self.geng, future)));
        // }

        todo!()
    }

    // pub fn fetch_music(&self) {
    //     let mut inner = self.inner.borrow_mut();
    //     if inner.tasks.fetch_music.is_none() {
    //         if let Some(client) = inner.tasks.client.clone() {
    //             let future = async move {
    //                 let music = client.get_music_list().await?;
    //                 Ok(music)
    //             };
    //             inner.tasks.fetch_music = Some(Task::new(&self.geng, future));
    //             inner.music_list = CacheState::Loading;
    //         }
    //     }
    // }

    // pub fn download_music(&self, music_id: Id) {
    //     let mut inner = self.inner.borrow_mut();

    //     if inner.music.contains_key(&music_id) {
    //         // Already downloaded
    //         // TODO: check version
    //         return;
    //     }

    //     if let Some(client) = inner.tasks.client.clone() {
    //         log::debug!("Downloading music {}", music_id);
    //         let geng = self.geng.clone();
    //         let fs = self.fs.clone();
    //         let future = async move {
    //             let meta = client.get_music_info(music_id).await?;
    //             let bytes = client.download_music(music_id).await?.to_vec();

    //             log::debug!("Decoding downloaded music bytes");
    //             let music = geng.audio().decode(bytes.clone()).await?;

    //             let music = LocalMusic::new(meta, music);

    //             // Write to fs
    //             if let Err(err) = fs.save_music(&music, &bytes).await {
    //                 log::error!("Failed to save music locally: {:?}", err);
    //             } else {
    //                 log::info!("Music saved successfully");
    //             }

    //             Ok(music)
    //         };
    //         inner
    //             .tasks
    //             .download_music
    //             .push_back((music_id, Task::new(&self.geng, future)));
    //     }
    // }

    pub fn poll(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(action) = inner.tasks.poll() {
            match action {
                CacheAction::GroupList(groups) => inner.group_list = CacheState::Loaded(groups),
                CacheAction::Group(mut group) => {
                    // // Check music
                    // if group.music.is_none() {
                    //     if let Some(music) = inner.music.get(&group.data.music) {
                    //         group.music = Some(music.clone());
                    //     }
                    // }

                    // let name = group
                    //     .music
                    //     .as_ref()
                    //     .map_or(&group.data.owner.name, |music| &music.meta.name);
                    // inner.notifications.push(format!(
                    //     "Downloaded level {} - {}",
                    //     name, group.data.owner.name
                    // ));

                    // if let Some(music) = &group.music {
                    //     inner.music.insert(music.meta.id, Rc::clone(music));
                    // }
                    // inner.groups.insert(Rc::new(group));

                    todo!()
                }
                CacheAction::DownloadGroups(ids) => {
                    drop(inner);
                    for group_id in ids {
                        self.download_group(group_id);
                    }
                }
            }
        }
    }

    pub fn synchronize(&self, group_index: Index, info: GroupInfo) -> Option<Rc<CachedGroup>> {
        log::debug!("Synchronizing cached group {:?}: {:?}", group_index, info);

        // let inner = self.inner.borrow();
        // let group = inner.groups.get(group_index)?;
        // let mut new_group = group.data.clone();

        // if new_group.levels.len() != info.levels.len() {
        //     log::error!("tried synchorinizing but groups have incompatible level counts");
        //     return None;
        // }
        // for (level, info) in new_group.levels.iter_mut().zip(&info.levels) {
        //     let mut lvl = (**level).clone();
        //     lvl.meta = info.clone();
        //     *level = Rc::new(lvl);
        // }
        // new_group.id = info.id;
        // new_group.owner = info.owner.clone();
        // new_group.music = info.music.id;

        // drop(inner);

        // self.update_group(group_index, new_group, Some(info))

        todo!()
    }

    pub fn update_group(
        &self,
        group_index: Index,
        group: LevelSet,
        reset_origin: Option<GroupInfo>,
    ) -> Option<Rc<CachedGroup>> {
        // let mut inner = self.inner.borrow_mut();
        // let cached = inner.groups.get_mut(group_index)?;

        // let mut new_group: CachedGroup = (**cached).clone();

        // let old_path = new_group.path.clone();

        // new_group.hash = group.calculate_hash();
        // if let Some(info) = reset_origin {
        //     new_group.origin = Some(info);
        // }
        // new_group.level_hashes = group
        //     .levels
        //     .iter()
        //     .map(|level| level.data.calculate_hash())
        //     .collect();
        // new_group.data = group;
        // new_group.path = fs::generate_group_path(new_group.data.id);

        // let group = Rc::new(new_group);
        // *cached = Rc::clone(&group);

        // // Write to fs
        // drop(inner);
        // self.move_group(&group, old_path);

        // Some(group)

        todo!()
    }

    pub fn update_level(
        &self,
        group_index: Index,
        level_index: usize,
        level: Level,
        name: String,
    ) -> Option<(Rc<CachedGroup>, Rc<LevelFull>)> {
        // let inner = self.inner.borrow();
        // let mut new_group = inner.groups.get(group_index)?.data.clone();
        // let new_level = new_group.levels.get_mut(level_index)?;

        // let mut meta = new_level.meta.clone();
        // meta.name = name.into();
        // *new_level = Rc::new(LevelFull { meta, data: level });

        // drop(inner);
        // let group = self.update_group(group_index, new_group, None)?;
        // let level = group.data.levels.get(level_index)?.clone();
        // Some((group, level))

        todo!()
    }

    /// Delete the music and all associated groups.
    pub fn delete_music(&self, music_id: Id) {
        // let mut inner = self.inner.borrow_mut();
        // if let Some(_music) = inner.music.remove(&music_id) {
        //     let group_ids: Vec<_> = inner
        //         .groups
        //         .iter()
        //         .filter(|(_, group)| group.data.music == music_id)
        //         .map(|(idx, _)| idx)
        //         .collect();
        //     drop(inner);
        //     for idx in group_ids {
        //         self.delete_group(idx);
        //     }
        //     self.remove_music(music_id);
        // }

        todo!()
    }

    pub fn delete_group(&self, group: Index) {
        let mut inner = self.inner.borrow_mut();
        if let Some(group) = inner.groups.remove(group) {
            drop(inner);
            self.remove_group(&group.local.path);
        }
    }

    pub fn delete_level(&self, group_index: Index, level: usize) {
        let inner = self.inner.borrow();
        if let Some(group) = inner.groups.get(group_index) {
            let mut new_group = group.local.data.clone();
            if level < new_group.levels.len() {
                let _level = new_group.levels.remove(level);
                drop(inner);
                self.update_group(group_index, new_group, None);
            }
        }
    }
}
