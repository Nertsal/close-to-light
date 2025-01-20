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

    fs: VecDeque<Task<anyhow::Result<()>>>,

    fetch_music: TaskRes<Vec<MusicInfo>>,
    // downloading_music: HashSet<Id>,
    download_music: VecDeque<(Id, Task<Result<CachedMusic>>)>,

    fetch_groups: TaskRes<Vec<GroupInfo>>,
    // downloading_groups: HashSet<Id>,
    download_group: VecDeque<(Id, Task<Result<CachedGroup>>)>,
    get_recommended: TaskRes<Vec<GroupInfo>>,

    notifications: Vec<String>,
}

#[derive(Debug)]
enum CacheAction {
    MusicList(Vec<MusicInfo>),
    Music(CachedMusic),
    GroupList(Vec<GroupInfo>),
    Group(CachedGroup),
    DownloadGroups(Vec<Id>),
}

impl CacheTasks {
    pub fn new(client: Option<&Arc<Nertboard>>) -> Self {
        Self {
            client: client.cloned(),

            fs: VecDeque::new(),

            fetch_music: None,
            // downloading_music: HashSet::new(),
            download_music: VecDeque::new(),

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
        } else if let Some(task) = self.fetch_music.take() {
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
        } else if let Some(task) = self.get_recommended.take() {
            match task.poll() {
                Err(task) => self.get_recommended = Some(task),
                Ok(Err(err)) => error!("Failed to get recommended levels: {:?}", err),
                Ok(Ok(groups)) => {
                    let group_ids = groups.into_iter().map(|group| group.id).collect();
                    return Some(CacheAction::DownloadGroups(group_ids));
                }
            }
        } else if let Some((music_id, task)) = self.download_music.pop_front() {
            match task.poll() {
                Err(task) => self.download_music.push_front((music_id, task)),
                Ok(Err(err)) => {
                    error!("Failed to download music {}: {:?}", music_id, err);
                }
                Ok(Ok(music)) => {
                    log::debug!("downloaded music {}: {:?}", music_id, music);
                    return Some(CacheAction::Music(music));
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

            music_list: CacheState::Offline,
            group_list: CacheState::Offline,

            music: HashMap::new(),
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
        let music = {
            let music = self.fs.load_music_all().await?;
            let mut inner = self.inner.borrow_mut();
            inner.music.extend(
                music
                    .into_iter()
                    .map(|music| (music.meta.id, Rc::new(music))),
            );
            log::debug!("loaded music: {:?}", inner.music);

            // NOTE: To not hold &inner.music across an await point for load_groups_all
            std::mem::take(&mut inner.music)
        };

        {
            let groups = self.fs.load_groups_all(&music).await?;
            let group_loaders = groups
                .into_iter()
                .map(|(path, group)| self.insert_group(path, group));
            let groups = future::join_all(group_loaders).await;
            let mut inner = self.inner.borrow_mut();
            inner.music.extend(music);
            for (music, group) in groups.into_iter().flatten() {
                if let Some(music) = music {
                    inner.music.insert(music.meta.id, music);
                }
                inner.groups.insert(Rc::new(group));
            }
            log::debug!("loaded groups: {}", inner.groups.len());
        }

        Ok(())
    }

    pub fn is_downloading_music(&self) -> Vec<Id> {
        let inner = self.inner.borrow();
        inner
            .tasks
            .download_music
            .iter()
            .map(|(id, _)| *id)
            .collect()
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

    async fn insert_group(
        &self,
        group_path: PathBuf,
        group: LevelSet,
    ) -> Result<(Option<Rc<CachedMusic>>, CachedGroup)> {
        let result = async {
            let hash = group.calculate_hash();

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
                if old_path != group.path {
                    fs.remove_group(old_path).await?;
                }
                Ok(())
            }
        };
        inner.tasks.fs.push_back(Task::new(&self.geng, future));
    }

    fn remove_music(&self, id: Id) {
        let mut inner = self.inner.borrow_mut();
        let future = {
            let fs = self.fs.clone();
            async move {
                fs.remove_music(id).await?;
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

    pub fn new_group(&self, music_id: Id) {
        let inner = self.inner.borrow();

        // Generate a non-occupied path
        let path = loop {
            let path = fs::generate_group_path(0);
            if !inner.groups.iter().any(|(_, group)| group.path == path) {
                break path;
            }
        };

        let music = inner.music.get(&music_id).cloned();
        let mut group = CachedGroup::new(
            path,
            LevelSet {
                id: 0,
                music: music_id,
                // TODO: set the logged in user
                owner: UserInfo {
                    id: 0,
                    name: "".into(),
                },
                levels: Vec::new(),
            },
        );
        group.music = music;

        // Write to fs
        drop(inner);
        let group = Rc::new(group);
        self.save_group(&group);

        self.inner.borrow_mut().groups.insert(group);
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
        let mut inner = self.inner.borrow_mut();
        if inner
            .groups
            .iter()
            .any(|(_, group)| group_id == group.data.id)
        {
            // Already downloaded
            // TODO: check version
            return;
        }

        if let Some(client) = inner.tasks.client.clone() {
            let geng = self.geng.clone();
            let fs = self.fs.clone();
            let music_list = inner.music.clone();

            let future = {
                async move {
                    log::debug!("Downloading group {}", group_id);

                    // Download group
                    let info = client.get_group_info(group_id).await?;
                    let bytes = client.download_group(group_id).await?.to_vec();
                    let hash = ctl_client::core::util::calculate_hash(&bytes);
                    // let data: LevelSet = bincode::deserialize(&bytes)?;
                    let data: LevelSet = cbor4ii::serde::from_slice(&bytes)?;

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
                            let music = CachedMusic::new(meta, music);

                            if let Err(err) = fs.save_music(&music, &bytes).await {
                                log::error!("Failed to save music locally: {:?}", err);
                            }

                            Rc::new(music)
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

                    // Write to fs
                    if let Err(err) = fs.save_group(&group).await {
                        log::error!("Failed to save group locally: {:?}", err);
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

        if inner.music.contains_key(&music_id) {
            // Already downloaded
            // TODO: check version
            return;
        }

        if let Some(client) = inner.tasks.client.clone() {
            log::debug!("Downloading music {}", music_id);
            let geng = self.geng.clone();
            let fs = self.fs.clone();
            let future = async move {
                let meta = client.get_music_info(music_id).await?;
                let bytes = client.download_music(music_id).await?.to_vec();

                log::debug!("Decoding downloaded music bytes");
                let music = geng.audio().decode(bytes.clone()).await?;

                let music = CachedMusic::new(meta, music);

                // Write to fs
                if let Err(err) = fs.save_music(&music, &bytes).await {
                    log::error!("Failed to save music locally: {:?}", err);
                } else {
                    log::info!("Music saved successfully");
                }

                Ok(music)
            };
            inner
                .tasks
                .download_music
                .push_back((music_id, Task::new(&self.geng, future)));
        }
    }

    pub fn poll(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(action) = inner.tasks.poll() {
            match action {
                CacheAction::MusicList(music) => inner.music_list = CacheState::Loaded(music),
                CacheAction::Music(music) => {
                    inner
                        .notifications
                        .push(format!("Downloaded music {}", music.meta.name));

                    inner.music.insert(music.meta.id, Rc::new(music));
                }
                CacheAction::GroupList(groups) => inner.group_list = CacheState::Loaded(groups),
                CacheAction::Group(mut group) => {
                    // Check music
                    if group.music.is_none() {
                        if let Some(music) = inner.music.get(&group.data.music) {
                            group.music = Some(music.clone());
                        }
                    }

                    let name = group
                        .music
                        .as_ref()
                        .map_or(&group.data.owner.name, |music| &music.meta.name);
                    inner.notifications.push(format!(
                        "Downloaded level {} - {}",
                        name, group.data.owner.name
                    ));

                    if let Some(music) = &group.music {
                        inner.music.insert(music.meta.id, Rc::clone(music));
                    }
                    inner.groups.insert(Rc::new(group));
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

        let inner = self.inner.borrow();
        let group = inner.groups.get(group_index)?;
        let mut new_group = group.data.clone();

        if new_group.levels.len() != info.levels.len() {
            log::error!("tried synchorinizing but groups have incompatible level counts");
            return None;
        }
        for (level, info) in new_group.levels.iter_mut().zip(&info.levels) {
            let mut lvl = (**level).clone();
            lvl.meta = info.clone();
            *level = Rc::new(lvl);
        }
        new_group.id = info.id;
        new_group.owner = info.owner.clone();
        new_group.music = info.music.id;

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

        // Write to fs
        drop(inner);
        self.move_group(&group, old_path);

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

    /// Delete the music and all associated groups.
    pub fn delete_music(&self, music_id: Id) {
        let mut inner = self.inner.borrow_mut();
        if let Some(_music) = inner.music.remove(&music_id) {
            let group_ids: Vec<_> = inner
                .groups
                .iter()
                .filter(|(_, group)| group.data.music == music_id)
                .map(|(idx, _)| idx)
                .collect();
            drop(inner);
            for idx in group_ids {
                self.delete_group(idx);
            }
            self.remove_music(music_id);
        }
    }

    pub fn delete_group(&self, group: Index) {
        let mut inner = self.inner.borrow_mut();
        if let Some(group) = inner.groups.remove(group) {
            drop(inner);
            self.remove_group(&group.path);
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
