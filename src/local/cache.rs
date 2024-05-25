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
    pub groups: Arena<CachedGroup>,
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
                    log::debug!("downloaded group: {:?}", group.meta);
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
        };
        Self {
            geng: geng.clone(),
            inner: RefCell::new(inner),
        }
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
                    if !path.is_dir() {
                        log::error!("Unexpected file in levels dir: {:?}", path);
                        return Ok(None);
                    }
                    anyhow::Ok(Some(path))
                })
                .flatten()
                .collect();
            let group_loaders = paths.iter().map(|path| local.load_group_all(path));
            let groups = future::join_all(group_loaders).await;

            let mut inner = local.inner.borrow_mut();
            for (music, group) in groups.into_iter().flatten() {
                if let Some(music) = music {
                    inner.music.insert(music.meta.id, music);
                }
                inner.groups.insert(group);
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

    pub fn get_level(&self, group: Index, level: usize) -> Option<Rc<CachedLevel>> {
        let inner = self.inner.borrow();
        inner
            .groups
            .get(group)
            .and_then(|group| group.levels.get(level))
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

    pub async fn load_level(
        &self,
        level_path: impl AsRef<std::path::Path>,
    ) -> Result<(Rc<CachedMusic>, Rc<CachedLevel>)> {
        let level_path = level_path.as_ref();
        let (level_path, group_path) = if level_path.is_dir() {
            (
                level_path,
                level_path
                    .parent()
                    .ok_or(anyhow!("Level expected to be in a folder"))?,
            )
        } else {
            // Assume path to `level.json`
            let level_path = level_path
                .parent()
                .ok_or(anyhow!("Level expected to be in a folder"))?;
            (
                level_path,
                level_path
                    .parent()
                    .ok_or(anyhow!("Level expected to be in a folder"))?,
            )
        };

        // TODO: do not load all the group levels
        self.load_group_all(&group_path).await?;

        // If `load_group_all` succedes, the group is pushed to the end
        let inner = self.inner.borrow();
        let (_, group) = inner.groups.iter().last().unwrap();

        let music = group
            .music
            .clone()
            .ok_or(anyhow!("Group music not found"))?;

        let level = group
            .levels
            .iter()
            .find(|level| level.path == level_path)
            .ok_or(anyhow!("Specific level not found"))?
            .clone();

        Ok((music, level))
    }

    /// Load the group info at the given path without loading the levels.
    async fn load_group_empty(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(Option<Rc<CachedMusic>>, CachedGroup)> {
        let group_path = path.as_ref().to_path_buf();

        let meta_path = group_path.join("meta.toml"); // TODO: move to fs
        let meta: GroupMeta = file::load_detect(&meta_path).await?;

        let music = match self.get_music(meta.music) {
            Some(music) => Some(music),
            None => {
                let music_path = fs::music_path(meta.music);
                self.load_music(&music_path).await.ok()
            }
        };

        let group = CachedGroup {
            path: group_path,
            meta,
            music: music.clone(),
            levels: Vec::new(),
        };

        Ok((music, group))
    }

    /// Load the group and all levels from it.
    async fn load_group_all(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(Option<Rc<CachedMusic>>, CachedGroup)> {
        let group_path = path.as_ref();
        log::debug!("loading music at {:?}", group_path);
        let (music, mut group) = self.load_group_empty(group_path).await?;

        let mut levels = Vec::new();
        for entry in std::fs::read_dir(group_path)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let level = CachedLevel::load(self.geng.asset_manager(), &path).await?;
            levels.push(Rc::new(level));
        }

        group.levels.extend(levels);
        Ok((music, group))
    }

    pub fn new_group(&self, music_id: Id) {
        let mut inner = self.inner.borrow_mut();

        let music = inner.music.get(&music_id).cloned();
        let mut group = CachedGroup::new(GroupMeta {
            id: 0,
            music: music_id,
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

        inner.groups.insert(group);
    }

    pub fn new_level(&self, group: Index, meta: LevelInfo) {
        if meta.id != 0 {
            log::error!("Trying to create a new level with non-zero id");
            return;
        }

        let mut inner = self.inner.borrow_mut();
        if let Some(group) = inner.groups.get_mut(group) {
            let mut level = CachedLevel::new(meta);
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Write to fs
                level.path = fs::generate_level_path(&group.path, 0);
                if let Err(err) = fs::save_level(&level) {
                    log::error!("Failed to save level locally: {:?}", err);
                }
            }
            group.levels.push(Rc::new(level));
        }
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
            .any(|(_, group)| group.meta.id == group_id)
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
                    let info = client.get_group_info(group_id).await?;

                    // Music
                    let music = match music_list.get(&info.music.id) {
                        Some(music) => Rc::clone(music),
                        None => {
                            // Music is not local so we need to download it
                            let meta = client.get_music_info(info.music.id).await?;
                            let bytes = client.download_music(info.music.id).await?.to_vec();

                            log::debug!("Decoding downloaded music bytes");
                            let music = Rc::new(geng.audio().decode(bytes.clone()).await?);

                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                // Write to fs
                                if let Err(err) = fs::download_music(info.music.id, bytes, &meta) {
                                    log::error!("Failed to save music locally: {:?}", err);
                                } else {
                                    log::info!("Music saved successfully");
                                }
                            }

                            Rc::new(CachedMusic { meta, music })
                        }
                    };

                    // Levels
                    let group_path = fs::generate_group_path(info.id);
                    let meta = GroupMeta {
                        id: info.id,
                        music: info.music.id,
                    };
                    let mut levels = Vec::new();

                    for info in info.levels {
                        let bytes = client.download_level(info.id).await?.to_vec();

                        let hash = ctl_client::core::util::calculate_hash(&bytes);

                        let level: Level = bincode::deserialize(&bytes)?;
                        levels.push(Rc::new(CachedLevel {
                            path: fs::generate_level_path(&group_path, info.id),
                            meta: info,
                            data: level,
                            hash,
                        }));
                    }

                    let group = CachedGroup {
                        path: group_path,
                        meta,
                        music: Some(music),
                        levels,
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
                    let music = Rc::new(geng.audio().decode(bytes.clone()).await?);

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // Write to fs
                        if let Err(err) = fs::download_music(meta.id, bytes, &meta) {
                            log::error!("Failed to save music locally: {:?}", err);
                        } else {
                            log::info!("Music saved successfully");
                        }
                    }

                    let music = CachedMusic { meta, music };
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
                    if let Some(music) = &group.music {
                        inner.music.insert(music.meta.id, Rc::clone(music));
                    }
                    inner.groups.insert(group);
                }
            }
        }
    }

    pub fn synchronize(
        &self,
        group_index: Index,
        level_index: usize,
        group_id: Id,
        level_id: Id,
    ) -> Option<Rc<CachedLevel>> {
        let mut inner = self.inner.borrow_mut();
        if let Some(group) = inner.groups.get_mut(group_index) {
            group.meta.id = group_id;
            if let Some(level) = group.levels.get_mut(level_index) {
                let mut new_level: CachedLevel = (**level).clone();

                #[cfg(not(target_arch = "wasm32"))]
                let old_path = new_level.path.clone();

                new_level.meta.id = level_id;
                new_level.path = fs::generate_level_path(&group.path, level_id);

                #[cfg(not(target_arch = "wasm32"))]
                {
                    // Write to fs
                    if let Err(err) = fs::save_level(&new_level) {
                        log::error!("Failed to save the level: {:?}", err);
                    } else {
                        // Remove the level from the old path
                        log::debug!("Deleting the old level folder: {:?}", old_path);
                        let _ = std::fs::remove_dir_all(old_path);
                    }
                }

                *level = Rc::new(new_level);
                return Some(Rc::clone(level));
            }
        }
        None
    }

    pub fn update_level(&self, level_id: Id, level: Level) -> Option<Rc<CachedLevel>> {
        let mut inner = self.inner.borrow_mut();
        let cached = inner.groups.iter_mut().find_map(|(_, group)| {
            group
                .levels
                .iter_mut()
                .find(|level| level.meta.id == level_id)
        })?;
        let mut new_level: CachedLevel = (**cached).clone();
        new_level.hash = level.calculate_hash();
        new_level.data = level;

        let level = Rc::new(new_level);
        *cached = Rc::clone(&level);

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Write to fs
            if let Err(err) = fs::save_level(&level) {
                log::error!("Failed to save the level: {:?}", err);
            } else {
                log::debug!("Successfully saved the level at: {:?}", level.path);
            }
        }

        Some(level)
    }

    pub fn delete_group(&self, group: Index) {
        let mut inner = self.inner.borrow_mut();
        if let Some(group) = inner.groups.remove(group) {
            #[cfg(not(target_arch = "wasm32"))]
            {
                log::debug!("Deleting the group folder: {:?}", group.path);
                if let Err(err) = std::fs::remove_dir_all(&group.path) {
                    log::error!("Failed to delete group: {:?}", err);
                }
            }
        }
    }

    pub fn delete_level(&self, group: Index, level: usize) {
        let mut inner = self.inner.borrow_mut();
        if let Some(group) = inner.groups.get_mut(group) {
            if level < group.levels.len() {
                let level = group.levels.remove(level);
                #[cfg(not(target_arch = "wasm32"))]
                {
                    log::debug!("Deleting the level folder: {:?}", level.path);
                    if let Err(err) = std::fs::remove_dir_all(&level.path) {
                        log::error!("Failed to delete level: {:?}", err);
                    }
                }
            }
        }
    }
}
