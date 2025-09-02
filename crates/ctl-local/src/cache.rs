use super::*;

use ctl_client::Nertboard;
use ctl_util::Task;
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
    pub group_list: CacheState<Vec<LevelSetInfo>>,

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

    fetch_groups: TaskRes<Vec<LevelSetInfo>>,
    // downloading_groups: HashSet<Id>,
    download_group: VecDeque<(Id, Task<Result<CachedGroup>>)>,
    get_recommended: TaskRes<Vec<LevelSetInfo>>,

    notifications: Vec<String>,
}

enum CacheAction {
    GroupList(Vec<LevelSetInfo>),
    Group(Box<CachedGroup>),
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
                    log::debug!("downloaded group {group_id}");
                    return Some(CacheAction::Group(Box::new(group)));
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
            .find(|(_, group)| group.local.meta.id == group_id)
            .map(|(idx, group)| (idx, Rc::clone(group)))
    }

    pub fn get_level(&self, group: Index, level: usize) -> Option<LevelFull> {
        let inner = self.inner.borrow();
        let group = inner.groups.get(group)?;
        let data = group.local.data.levels.get(level).cloned()?;
        let meta = group.local.meta.levels.get(level).cloned()?;
        Some(LevelFull { meta, data })
    }

    async fn insert_group(&self, mut group: LocalGroup) -> Result<CachedGroup> {
        let result = async {
            group.meta.hash = group.data.calculate_hash();

            let origin = if group.meta.id == 0 {
                None
            } else if let Some(client) = self.client() {
                match client.get_group_info(group.meta.id).await {
                    Err(err) => {
                        log::error!("failed to check group info: {err:?}");
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
                .map(|level| level.calculate_hash())
                .collect();

            let group = CachedGroup {
                local: group,
                origin,
                level_hashes,
            };

            Ok(group)
        }
        .await;

        if let Err(err) = &result {
            log::error!("failed to load group: {err:?}");
        }
        result
    }

    fn save_group(&self, group: &Rc<CachedGroup>, save_music: bool) {
        let mut inner = self.inner.borrow_mut();
        let future = {
            let fs = self.fs.clone();
            let group = group.clone();
            async move {
                fs.save_group(&group, save_music).await?;
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
                fs.save_group(&group, true).await?;
                if old_path != group.local.path {
                    // Move the music file
                    let music_path = old_path.join("music.mp3");
                    if music_path.exists() {
                        fs.copy_music_from(music_path, group.local.path.join("music.mp3"))
                            .await?;
                    }

                    // Remove old data
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

    pub fn new_group(&self) -> Index {
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

        let data = LevelSet { levels: Vec::new() };
        let group = CachedGroup {
            origin: None,
            level_hashes: vec![],
            local: LocalGroup {
                path,
                meta: LevelSetInfo {
                    id: 0,
                    owner: UserInfo {
                        id: 0,
                        name: "".into(),
                    },
                    music: MusicInfo::default(),
                    levels: vec![],
                    hash: data.calculate_hash(),
                },
                music: None,
                data,
            },
        };

        // Write to fs
        drop(inner);
        let group = Rc::new(group);
        self.save_group(&group, true);

        self.inner.borrow_mut().groups.insert(group)
    }

    pub fn fetch_groups(&self) {
        let mut inner = self.inner.borrow_mut();
        if inner.tasks.fetch_groups.is_none() {
            if let Some(client) = inner.tasks.client.clone() {
                let future = async move {
                    let groups = client
                        .get_group_list(&LevelSetsQuery { recommended: false })
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
                        .get_group_list(&LevelSetsQuery { recommended: true })
                        .await?;
                    Ok(list)
                };
                inner.tasks.get_recommended = Some(Task::new(&self.geng, future));
            }
        }
    }

    pub fn download_group(&self, group_id: Id) {
        let mut inner = self.inner.borrow_mut();
        if inner
            .groups
            .iter()
            .any(|(_, group)| group_id == group.local.meta.id)
        {
            // Already downloaded
            // TODO: check version
            return;
        }

        if let Some(client) = inner.tasks.client.clone() {
            let geng = self.geng.clone();
            let fs = self.fs.clone();

            let future = {
                async move {
                    log::debug!("Downloading group {group_id}");

                    // Download group
                    let info = client.get_group_info(group_id).await?;
                    let bytes = client.download_group(group_id).await?.to_vec();
                    let data: LevelSet = cbor4ii::serde::from_slice(&bytes)?;

                    // Download music
                    let music = {
                        log::debug!("Downloading music for group {}", info.id);
                        // Music is not local so we need to download it
                        let bytes = client.download_music_for_group(info.id).await?.to_vec();

                        log::debug!("Decoding downloaded music bytes");
                        let music = geng.audio().decode(bytes.clone()).await?;
                        let music = LocalMusic::new(info.music.clone(), music, bytes.into());

                        Rc::new(music)
                    };

                    let level_hashes = data
                        .levels
                        .iter()
                        .map(|level| level.calculate_hash())
                        .collect();

                    let group = CachedGroup {
                        local: LocalGroup {
                            path: fs::generate_group_path(info.id),
                            meta: info.clone(),
                            music: Some(music),
                            data,
                        },
                        origin: Some(info),
                        level_hashes,
                    };

                    // Write to fs
                    if let Err(err) = fs.save_group(&group, true).await {
                        log::error!("Failed to save group locally: {err:?}");
                    }

                    Ok(group)
                }
            };
            inner
                .tasks
                .download_group
                .push_back((group_id, Task::new(&self.geng, future)));
        }
    }

    pub fn poll(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(action) = inner.tasks.poll() {
            match action {
                CacheAction::GroupList(groups) => inner.group_list = CacheState::Loaded(groups),
                CacheAction::Group(group) => {
                    let name = group
                        .local
                        .music
                        .as_ref()
                        .map_or(&group.local.meta.owner.name, |music| &music.meta.name);
                    inner.notifications.push(format!(
                        "Downloaded level {} - {}",
                        name, group.local.meta.owner.name
                    ));

                    inner.groups.insert(Rc::new(*group));
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

    pub fn synchronize_meta(
        &self,
        group_index: Index,
        info: LevelSetInfo,
    ) -> Option<Rc<CachedGroup>> {
        log::debug!("Synchronizing cached group {group_index:?}: {info:?}");

        let inner = self.inner.borrow();
        let group = inner.groups.get(group_index)?;

        if group.local.data.levels.len() != info.levels.len() {
            log::error!("tried synchorinizing but groups have incompatible level counts");
            return None;
        }

        drop(inner);

        self.update_group_meta(group_index, info)
    }

    fn update_group_local(
        &self,
        group_index: Index,
        new_local: LocalGroup,
        reset_origin: Option<LevelSetInfo>,
    ) -> Result<Rc<CachedGroup>> {
        let mut inner = self.inner.borrow_mut();
        let cached = inner
            .groups
            .get_mut(group_index)
            .ok_or(anyhow!("group {:?} not found", group_index))?;

        let mut new_group: CachedGroup = (**cached).clone();

        let old_path = new_group.local.path.clone();

        if let Some(info) = reset_origin {
            new_group.origin = Some(info);
        }
        new_group.local.meta.hash = new_local.data.calculate_hash();
        new_group.level_hashes = new_local
            .data
            .levels
            .iter()
            .map(|level| level.calculate_hash())
            .collect();
        new_group.local = new_local;

        // let move_from_assets = cached
        //     .local
        //     .path
        //     .parent()
        //     .is_some_and(|path| path != fs::all_groups_path());
        let move_to_local =
            new_group.local.meta.id != cached.local.meta.id && new_group.local.meta.id == 0;
        // || move_from_assets
        if move_to_local {
            new_group.local.path = fs::generate_group_path(new_group.local.meta.id);
        }

        let group = Rc::new(new_group);
        *cached = Rc::clone(&group);

        // Write to fs
        drop(inner);
        self.move_group(&group, old_path);

        Ok(group)
    }

    pub fn update_group_meta(
        &self,
        group_index: Index,
        group_meta: LevelSetInfo,
    ) -> Option<Rc<CachedGroup>> {
        let mut inner = self.inner.borrow_mut();
        let cached = inner.groups.get_mut(group_index)?;

        let mut new_group: LocalGroup = cached.local.clone();
        if let Some(music) = &new_group.music {
            let mut new_music: LocalMusic = (**music).clone();
            new_music.meta = group_meta.music.clone();
            new_group.music = Some(Rc::new(new_music));
        }
        new_group.meta = group_meta;

        drop(inner);
        self.update_group_local(group_index, new_group, None).ok()
    }

    pub fn update_group(
        &self,
        group_index: Index,
        group: LevelSet,
        reset_origin: Option<LevelSetInfo>,
    ) -> Option<Rc<CachedGroup>> {
        let mut inner = self.inner.borrow_mut();
        let cached = inner.groups.get_mut(group_index)?;

        let mut new_group: LocalGroup = cached.local.clone();
        new_group.data = group;

        drop(inner);
        self.update_group_local(group_index, new_group, reset_origin)
            .ok()
    }

    pub fn update_group_and_meta(
        &self,
        group_index: Index,
        group: LevelSet,
        group_meta: LevelSetInfo,
    ) -> Option<Rc<CachedGroup>> {
        let mut inner = self.inner.borrow_mut();
        let cached = inner.groups.get_mut(group_index)?;

        let mut new_group: LocalGroup = cached.local.clone();
        if let Some(music) = &new_group.music {
            let mut new_music: LocalMusic = (**music).clone();
            new_music.meta = group_meta.music.clone();
            new_group.music = Some(Rc::new(new_music));
        }
        new_group.meta = group_meta;
        new_group.data = group;

        drop(inner);
        self.update_group_local(group_index, new_group, None).ok()
    }

    pub fn update_level(
        &self,
        group_index: Index,
        level_index: usize,
        level: Level,
        name: String,
    ) -> Option<(Rc<CachedGroup>, LevelFull)> {
        let inner = self.inner.borrow();
        let group = inner.groups.get(group_index)?;
        let mut new_group = group.local.data.clone();
        let mut new_meta = group.local.meta.clone();
        let new_level = new_group.levels.get_mut(level_index)?;
        *new_level = Rc::new(level);

        let level_meta = new_meta.levels.get_mut(level_index)?;
        level_meta.name = name.into();

        drop(inner);
        let group = self.update_group_and_meta(group_index, new_group, new_meta)?;
        let level = group.local.data.levels.get(level_index)?.clone();
        let level_meta = group.local.meta.levels.get(level_index)?.clone();
        Some((
            group,
            LevelFull {
                meta: level_meta,
                data: level,
            },
        ))
    }

    pub fn select_music_file(
        &self,
        group_index: Index,
        _music_path: PathBuf,
    ) -> Result<Rc<CachedGroup>> {
        let inner = self.inner.borrow();
        let new_group = inner
            .groups
            .get(group_index)
            .ok_or(anyhow!("group {:?} not found", group_index))?
            .local
            .clone();
        drop(inner);

        #[cfg(not(target_arch = "wasm32"))]
        let new_group = {
            let mut new_group = new_group;

            let (music, music_bytes) = futures::executor::block_on({
                let path = _music_path.clone();
                let geng = self.geng.clone();
                async move {
                    let music_bytes = file::load_bytes(&path).await;
                    match music_bytes {
                        Ok(bytes) => {
                            let mut music: geng::Sound = geng.audio().decode(bytes.clone()).await?;
                            music.looped = true;
                            Ok((music, bytes))
                        }
                        Err(err) => Err(err),
                    }
                }
            })?;

            let music_meta = &mut new_group.meta.music;
            if let Some(name) = _music_path.file_name() {
                let name: Name = name.to_string_lossy().into();
                music_meta.name = name.clone();
                music_meta.romanized = name;
            }
            let music = LocalMusic::new(music_meta.clone(), music, music_bytes.into());
            new_group.music = Some(Rc::new(music));

            new_group
        };

        // TODO: should not move (change name) because we're only changing the music
        let group = self.update_group_local(group_index, new_group, None)?;

        // Copy music to the group path
        #[cfg(not(target_arch = "wasm32"))]
        {
            log::debug!("Copying music file into {:?}", group.local.path);
            // Create a dir because the rest of the group is saved asynchronously
            let _ = std::fs::create_dir_all(&group.local.path);
            if let Err(err) = std::fs::copy(&_music_path, group.local.path.join("music.mp3")) {
                log::error!("Copying music failed: {err:?}");
            }
        }

        Ok(group)
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
