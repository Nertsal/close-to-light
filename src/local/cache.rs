use super::*;

use crate::task::Task;

use ctl_client::Nertboard;
use generational_arena::Index;

type TaskRes<T> = Option<Task<anyhow::Result<T>>>;

pub struct LevelCache {
    geng: Geng,
    pub inner: RefCell<LevelCacheImpl>,
}

pub struct LevelCacheImpl {
    tasks: CacheTasks,

    /// List of downloadable music.
    pub music_list: CacheState<Vec<MusicInfo>>,
    /// List of downloadable level groups.
    pub group_list: CacheState<Vec<GroupInfo>>,

    pub music: HashMap<Id, Rc<CachedMusic>>,
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

    fetch_music: TaskRes<Vec<MusicInfo>>,
    download_music: TaskRes<CachedMusic>,

    fetch_groups: TaskRes<Vec<GroupInfo>>,
    download_group: TaskRes<CachedGroup>,
}

#[derive(Debug)]
enum CacheAction {
    MusicList(Vec<MusicInfo>),
    Music(CachedMusic),
    GroupList(Vec<GroupInfo>),
    Group(CachedGroup),
}

impl CacheTasks {
    pub fn new(client: Option<&Arc<Nertboard>>) -> Self {
        Self {
            client: client.cloned(),

            fetch_music: None,
            download_music: None,

            fetch_groups: None,
            download_group: None,
        }
    }

    fn poll(&mut self) -> Option<CacheAction> {
        if let Some(task) = self.fetch_music.take() {
            match task.poll() {
                Err(task) => self.fetch_music = Some(task),
                Ok(result) => {
                    if let Ok(music) = result {
                        return Some(CacheAction::MusicList(music));
                    }
                }
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
        } else if let Some(task) = self.download_music.take() {
            match task.poll() {
                Err(task) => self.download_music = Some(task),
                Ok(Err(err)) => {
                    log::error!("failed to download music: {:?}", err);
                }
                Ok(Ok(music)) => {
                    log::debug!("downloaded music: {:?}", music);
                    return Some(CacheAction::Music(music));
                }
            }
        } else if let Some(task) = self.download_group.take() {
            match task.poll() {
                Err(task) => self.download_group = Some(task),
                Ok(Err(err)) => {
                    log::error!("failed to download group: {:?}", err);
                }
                Ok(Ok(group)) => {
                    log::debug!("downloaded group: {:?}", group.data.id);
                    return Some(CacheAction::Group(group));
                }
            }
        }

        None
    }
}

impl LevelCache {
    pub fn new(client: Option<&Arc<Nertboard>>, geng: &Geng) -> Self {
        let inner = LevelCacheImpl {
            tasks: CacheTasks::new(client),

            music_list: CacheState::Offline,
            group_list: CacheState::Offline,

            music: HashMap::new(),
            groups: Arena::new(),

            notifications: Vec::new(),
        };
        Self {
            geng: geng.clone(),
            inner: RefCell::new(inner),
        }
    }

    pub fn take_notifications(&self) -> Vec<String> {
        let mut inner = self.inner.borrow_mut();
        std::mem::take(&mut inner.notifications)
    }

    pub fn client(&self) -> Option<Arc<Nertboard>> {
        let inner = self.inner.borrow();
        inner.tasks.client.as_ref().cloned()
    }

    /// Load from the local storage.
    pub async fn load(client: Option<&Arc<Nertboard>>, geng: &Geng) -> Result<Self> {
        let mut timer = Timer::new();

        // TODO: report failures but continue working

        #[cfg(target_arch = "wasm32")]
        {
            return Ok(Self::new(client, geng));
        }

        log::info!("Loading local storage");
        let base_path = fs::base_path();
        std::fs::create_dir_all(&base_path)?;

        let local = Self::new(client, geng);

        let music_path = fs::all_music_path();
        if music_path.exists() {
            let paths: Vec<_> = std::fs::read_dir(music_path)?
                .flat_map(|entry| {
                    let entry = entry?;
                    let path = entry.path();
                    if !path.is_dir() {
                        log::error!("Unexpected file in music dir: {:?}", path);
                        return Ok(None);
                    }
                    anyhow::Ok(Some(path))
                })
                .flatten()
                .collect();
            let music_loaders = paths.iter().map(|path| local.load_music(path));
            let music = future::join_all(music_loaders).await;

            let mut inner = local.inner.borrow_mut();
            inner.music.extend(
                music
                    .into_iter()
                    .flatten()
                    .map(|music| (music.meta.id, music)),
            );
            log::debug!("loaded music: {:?}", inner.music);
        }

        let groups_path = fs::all_groups_path();
        if groups_path.exists() {
            let paths: Vec<_> = std::fs::read_dir(groups_path)?
                .flat_map(|entry| {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        log::error!("Unexpected directory inside levels: {:?}", path);
                        return Ok(None);
                    }
                    anyhow::Ok(Some(path))
                })
                .flatten()
                .collect();
            let group_loaders = paths.iter().map(|path| local.load_group(path));
            let groups = future::join_all(group_loaders).await;

            let mut inner = local.inner.borrow_mut();
            for (music, group) in groups.into_iter().flatten() {
                if let Some(music) = music {
                    inner.music.insert(music.meta.id, music);
                }
                inner.groups.insert(Rc::new(group));
            }
            log::debug!("loaded groups: {}", inner.groups.len());
        }

        log::debug!("Loaded cache in {:.2}s", timer.tick().as_secs_f64());

        Ok(local)
    }

    pub fn get_music(&self, music_id: Id) -> Option<Rc<CachedMusic>> {
        let inner = self.inner.borrow();
        inner.music.get(&music_id).cloned()
    }

    pub fn get_group(&self, group: Index) -> Option<Rc<CachedGroup>> {
        let inner = self.inner.borrow();
        inner.groups.get(group).cloned()
    }

    pub fn get_level(&self, group: Index, level: usize) -> Option<Rc<LevelFull>> {
        let inner = self.inner.borrow();
        inner
            .groups
            .get(group)
            .and_then(|group| group.data.levels.get(level))
            .cloned()
    }

    pub async fn load_music(&self, path: impl AsRef<std::path::Path>) -> Result<Rc<CachedMusic>> {
        let res = async {
            let path = path.as_ref();
            log::debug!("loading music at {:?}", path);
            let music = Rc::new(CachedMusic::load(self.geng.asset_manager(), path).await?);
            Ok(music)
        }
        .await;
        if let Err(err) = &res {
            log::error!("failed to load music: {:?}", err);
        }
        res
    }

    /// Load the group info at the given path without loading the levels.
    pub async fn load_group(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(Option<Rc<CachedMusic>>, CachedGroup)> {
        let result = async {
            let group_path = path.as_ref().to_path_buf();
            log::debug!("loading group at {:?}", group_path);

            let bytes = file::load_bytes(&group_path).await?;
            let hash = ctl_client::core::util::calculate_hash(&bytes);
            let group: LevelSet = bincode::deserialize(&bytes)?;

            let music = match self.get_music(group.music) {
                Some(music) => Some(music),
                None => {
                    let music_path = fs::music_path(group.music);
                    self.load_music(&music_path).await.ok()
                }
            };

            let origin = if group.id == 0 {
                None
            } else if let Some(client) = self.client() {
                match client.get_group_info(group.id).await {
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
                .levels
                .iter()
                .map(|level| level.data.calculate_hash())
                .collect();

            let group = CachedGroup {
                path: group_path,
                music: music.clone(),
                hash,
                data: group,
                origin,
                level_hashes,
            };

            Ok((music, group))
        }
        .await;

        if let Err(err) = &result {
            log::error!("failed to load group: {:?}", err);
        }
        result
    }

    pub fn new_group(&self, music_id: Id) {
        let mut inner = self.inner.borrow_mut();

        let music = inner.music.get(&music_id).cloned();
        let mut group = CachedGroup::new(LevelSet {
            id: 0,
            music: music_id,
            // TODO: set the logged in user
            owner: UserInfo {
                id: 0,
                name: "".into(),
            },
            levels: Vec::new(),
        });
        group.music = music;

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Write to fs
            group.path = fs::generate_group_path(0);
            if let Err(err) = fs::save_group(&group) {
                log::error!("Failed to save group: {:?}", err);
            }
        }

        inner.groups.insert(Rc::new(group));
    }

    pub fn fetch_groups(&self) {
        let mut inner = self.inner.borrow_mut();
        if inner.tasks.fetch_groups.is_none() {
            if let Some(client) = inner.tasks.client.clone() {
                let future = async move {
                    let groups = client.get_group_list().await?;
                    Ok(groups)
                };
                inner.tasks.fetch_groups = Some(Task::new(&self.geng, future));
                inner.group_list = CacheState::Loading;
            }
        }
    }

    pub fn download_group(&self, group_id: Id) {
        let mut inner = self.inner.borrow_mut();
        if inner
            .groups
            .iter()
            .any(|(_, group)| group.data.id == group_id)
        {
            // Already downloaded
            // TODO: check version
            return;
        }

        if inner.tasks.download_group.is_none() {
            if let Some(client) = inner.tasks.client.clone() {
                log::debug!("Downloading group {}", group_id);
                let geng = self.geng.clone();
                let music_list = inner.music.clone();

                let future = async move {
                    // Download group
                    let info = client.get_group_info(group_id).await?;
                    let bytes = client.download_group(group_id).await?.to_vec();
                    let hash = ctl_client::core::util::calculate_hash(&bytes);
                    let data: LevelSet = bincode::deserialize(&bytes)?;

                    // Download music
                    let music = match music_list.get(&data.music) {
                        Some(music) => Rc::clone(music),
                        None => {
                            log::debug!("Downloading music {}", data.music);
                            // Music is not local so we need to download it
                            let meta = client.get_music_info(data.music).await?;
                            let bytes = client.download_music(data.music).await?.to_vec();

                            log::debug!("Decoding downloaded music bytes");
                            let music = geng.audio().decode(bytes.clone()).await?;

                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                // Write to fs
                                if let Err(err) = fs::download_music(data.music, bytes, &meta) {
                                    log::error!("Failed to save music locally: {:?}", err);
                                } else {
                                    log::info!("Music saved successfully");
                                }
                            }

                            Rc::new(CachedMusic::new(meta, music))
                        }
                    };

                    let level_hashes = data
                        .levels
                        .iter()
                        .map(|level| level.data.calculate_hash())
                        .collect();

                    let group = CachedGroup {
                        path: fs::generate_group_path(data.id),
                        music: Some(music),
                        data,
                        origin: Some(info),
                        hash,
                        level_hashes,
                    };

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // Write to fs
                        if let Err(err) = fs::save_group(&group) {
                            log::error!("Failed to save group locally: {:?}", err);
                        }
                    }

                    Ok(group)
                };
                inner.tasks.download_group = Some(Task::new(&self.geng, future));
            }
        }
    }

    pub fn fetch_music(&self) {
        let mut inner = self.inner.borrow_mut();
        if inner.tasks.fetch_music.is_none() {
            if let Some(client) = inner.tasks.client.clone() {
                let future = async move {
                    let music = client.get_music_list().await?;
                    Ok(music)
                };
                inner.tasks.fetch_music = Some(Task::new(&self.geng, future));
                inner.music_list = CacheState::Loading;
            }
        }
    }

    pub fn download_music(&self, music_id: Id) {
        let mut inner = self.inner.borrow_mut();
        if inner.tasks.download_music.is_none() {
            if let Some(client) = inner.tasks.client.clone() {
                log::debug!("Downloading music {}", music_id);
                let geng = self.geng.clone();
                let future = async move {
                    let meta = client.get_music_info(music_id).await?;
                    let bytes = client.download_music(music_id).await?.to_vec();

                    log::debug!("Decoding downloaded music bytes");
                    let music = geng.audio().decode(bytes.clone()).await?;

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // Write to fs
                        if let Err(err) = fs::download_music(meta.id, bytes, &meta) {
                            log::error!("Failed to save music locally: {:?}", err);
                        } else {
                            log::info!("Music saved successfully");
                        }
                    }

                    let music = CachedMusic::new(meta, music);
                    Ok(music)
                };
                inner.tasks.download_music = Some(Task::new(&self.geng, future));
            }
        }
    }

    pub fn poll(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(action) = inner.tasks.poll() {
            match action {
                CacheAction::MusicList(music) => inner.music_list = CacheState::Loaded(music),
                CacheAction::Music(music) => {
                    inner.music.insert(music.meta.id, Rc::new(music));
                }
                CacheAction::GroupList(groups) => inner.group_list = CacheState::Loaded(groups),
                CacheAction::Group(group) => {
                    let name = group
                        .music
                        .as_ref()
                        .map_or(&group.data.owner.name, |music| &music.meta.name);
                    inner
                        .notifications
                        .push(format!("Downloaded level {}", name));

                    if let Some(music) = &group.music {
                        inner.music.insert(music.meta.id, Rc::clone(music));
                    }
                    inner.groups.insert(Rc::new(group));
                }
            }
        }
    }

    pub fn synchronize(&self, group_index: Index, info: GroupInfo) -> Option<Rc<CachedGroup>> {
        let inner = self.inner.borrow();
        let group = inner.groups.get(group_index)?;
        let mut new_group = group.data.clone();

        if new_group.levels.len() != info.levels.len() {
            log::error!("tried synchorinizing but groups have incompatible level counts");
            return None;
        }
        for (level, info) in new_group.levels.iter_mut().zip(&info.levels) {
            let mut lvl = (**level).clone();
            lvl.meta.id = info.id;
            *level = Rc::new(lvl);
        }
        new_group.id = info.id;
        new_group.owner = info.owner.clone();

        drop(inner);

        self.update_group(group_index, new_group, Some(info))
    }

    pub fn update_group(
        &self,
        group_index: Index,
        group: LevelSet,
        reset_origin: Option<GroupInfo>,
    ) -> Option<Rc<CachedGroup>> {
        let mut inner = self.inner.borrow_mut();
        let cached = inner.groups.get_mut(group_index)?;

        let mut new_group: CachedGroup = (**cached).clone();

        #[cfg(not(target_arch = "wasm32"))]
        let old_path = new_group.path.clone();

        new_group.hash = group.calculate_hash();
        if let Some(info) = reset_origin {
            new_group.origin = Some(info);
        }
        new_group.level_hashes = group
            .levels
            .iter()
            .map(|level| level.data.calculate_hash())
            .collect();
        new_group.data = group;
        new_group.path = fs::generate_group_path(new_group.data.id);

        let group = Rc::new(new_group);
        *cached = Rc::clone(&group);

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Write to fs
            if let Err(err) = fs::save_group(&group) {
                log::error!("Failed to save the group: {:?}", err);
            } else if old_path != group.path {
                // Remove the level from the old path
                log::debug!("Deleting the old group: {:?}", old_path);
                let _ = std::fs::remove_file(old_path);
            }
        }

        Some(group)
    }

    pub fn update_level(
        &self,
        group_index: Index,
        level_index: usize,
        level: Level,
        name: String,
    ) -> Option<(Rc<CachedGroup>, Rc<LevelFull>)> {
        let inner = self.inner.borrow();
        let mut new_group = inner.groups.get(group_index)?.data.clone();
        let new_level = new_group.levels.get_mut(level_index)?;

        let mut meta = new_level.meta.clone();
        meta.name = name.into();
        *new_level = Rc::new(LevelFull { meta, data: level });

        drop(inner);
        let group = self.update_group(group_index, new_group, None)?;
        let level = group.data.levels.get(level_index)?.clone();
        Some((group, level))
    }

    pub fn delete_group(&self, group: Index) {
        let mut inner = self.inner.borrow_mut();
        if let Some(group) = inner.groups.remove(group) {
            #[cfg(not(target_arch = "wasm32"))]
            {
                log::debug!("Deleting the group: {:?}", group.path);
                if let Err(err) = std::fs::remove_file(&group.path) {
                    log::error!("Failed to delete group: {:?}", err);
                }
            }
        }
    }

    pub fn delete_level(&self, group_index: Index, level: usize) {
        let inner = self.inner.borrow();
        if let Some(group) = inner.groups.get(group_index) {
            let mut new_group = group.data.clone();
            if level < new_group.levels.len() {
                let _level = new_group.levels.remove(level);

                drop(inner);
                self.update_group(group_index, new_group, None);
            }
        }
    }
}
