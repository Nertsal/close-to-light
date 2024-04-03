use super::*;

use crate::task::Task;

use ctl_client::Nertboard;
use generational_arena::Index;

pub struct LevelCache {
    geng: Geng,
    tasks: CacheTasks,
    pub playing_music: Option<Music>,

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

    fetch_music: Option<Task<anyhow::Result<Vec<MusicInfo>>>>,
    download_music: Option<Task<anyhow::Result<CachedMusic>>>,

    fetch_groups: Option<Task<anyhow::Result<Vec<GroupInfo>>>>,
    download_group: Option<Task<anyhow::Result<CachedGroup>>>,
}

#[derive(Debug)]
enum CacheAction {
    MusicList(Vec<MusicInfo>),
    Music(CachedMusic),
    GroupList(Vec<GroupInfo>),
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
                Ok(Err(_)) => {}
                Ok(Ok(music)) => {
                    return Some(CacheAction::Music(music));
                }
            }
        }

        None
    }
}

impl LevelCache {
    pub fn new(client: Option<&Arc<Nertboard>>, geng: &Geng) -> Self {
        Self {
            geng: geng.clone(),
            tasks: CacheTasks::new(client),
            playing_music: None,

            music_list: CacheState::Offline,
            group_list: CacheState::Offline,

            music: HashMap::new(),
            groups: Arena::new(),
        }
    }

    pub fn client(&self) -> Option<&Arc<Nertboard>> {
        self.tasks.client.as_ref()
    }

    /// Load from the local storage.
    pub async fn load(client: Option<&Arc<Nertboard>>, geng: &Geng) -> Result<Self> {
        // TODO: report failures but continue working

        #[cfg(target_arch = "wasm32")]
        {
            return Ok(Self::new(client, geng));
        }

        log::info!("Loading local storage");
        let base_path = fs::base_path();
        std::fs::create_dir_all(&base_path)?;

        let mut local = Self::new(client, geng);

        for entry in std::fs::read_dir(fs::all_music_path())? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                log::error!("Unexpected file in music dir: {:?}", path);
                continue;
            }

            local.load_music(&path).await?;
        }

        let levels_path = fs::all_levels_path();
        if levels_path.exists() {
            for entry in std::fs::read_dir(levels_path)? {
                let entry = entry?;
                let path = entry.path();
                if !path.is_dir() {
                    log::error!("Unexpected file in levels dir: {:?}", path);
                    continue;
                }

                local.load_group_all(&path).await?;
            }
        }

        Ok(local)
    }

    pub async fn load_music(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Rc<CachedMusic>> {
        let path = path.as_ref();
        let music = Rc::new(CachedMusic::load(self.geng.asset_manager(), path).await?);
        self.music.insert(music.meta.id, Rc::clone(&music));
        Ok(music)
    }

    pub async fn load_level(
        &mut self,
        level_path: impl AsRef<std::path::Path>,
    ) -> Result<(Rc<CachedMusic>, Rc<CachedLevel>)> {
        let level_path = level_path.as_ref();
        let (level_path, group_path) = if level_path.is_dir() {
            (
                level_path.join("level.json"), // TODO: move to fs
                level_path
                    .parent()
                    .ok_or(anyhow!("Level expected to be in a folder"))?,
            )
        } else {
            // Assume path to `level.json`
            (
                level_path.to_path_buf(),
                level_path
                    .parent()
                    .ok_or(anyhow!("Level expected to be in a folder"))?
                    .parent()
                    .ok_or(anyhow!("Level expected to be in a folder"))?,
            )
        };

        // TODO: do not load all the group levels
        self.load_group_all(&group_path).await?;

        // If `load_group_all` succedes, the group is pushed to the end
        let (_, group) = self.groups.iter().last().unwrap();

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
    async fn load_group_empty(&mut self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let group_path = path.as_ref().to_path_buf();

        let meta_path = group_path.join("meta.toml"); // TODO: move to fs
        let meta: GroupMeta = file::load_detect(&meta_path).await?;

        let music = match self.music.get(&meta.music) {
            Some(music) => Some(music.clone()),
            None => {
                let music_path = fs::music_path(meta.music);
                CachedMusic::load(self.geng.asset_manager(), &music_path)
                    .await
                    .ok()
                    .map(|music| {
                        let music = Rc::new(music);
                        self.music.insert(meta.music, music.clone());
                        music
                    })
            }
        };

        let group = CachedGroup {
            path: group_path,
            meta,
            music,
            levels: Vec::new(),
        };
        self.groups.insert(group);

        Ok(())
    }

    /// Load the group and all levels from it.
    async fn load_group_all(&mut self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let group_path = path.as_ref();
        self.load_group_empty(group_path).await?;

        // If `load_group_empty` succedes, the group is pushed to the end
        let (_, group) = self.groups.iter_mut().last().unwrap();

        let mut levels = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let level = CachedLevel::load(self.geng.asset_manager(), &path).await?;
            levels.push(Rc::new(level));
        }

        group.levels.extend(levels);
        Ok(())
    }

    pub fn new_group(&mut self, music_id: Id) {
        let music = self.music.get(&music_id).cloned();
        let mut group = CachedGroup::new(GroupMeta {
            id: 0,
            music: music_id,
        });
        group.music = music;
        self.groups.insert(group);
        // TODO: write to fs
    }

    pub fn new_level(&mut self, group: Index, meta: LevelInfo) {
        if let Some(group) = self.groups.get_mut(group) {
            let level = CachedLevel::new(meta);
            group.levels.push(Rc::new(level));
            // TODO: write to fs
        }
    }

    pub fn fetch_groups(&mut self) {
        if self.tasks.fetch_groups.is_none() {
            if let Some(client) = self.tasks.client.clone() {
                let future = async move {
                    let groups = client.get_group_list().await?;
                    Ok(groups)
                };
                self.tasks.fetch_groups = Some(Task::new(&self.geng, future));
                self.group_list = CacheState::Loading;
            }
        }
    }

    pub fn download_group(&mut self, group_id: Id) {
        if self.tasks.download_group.is_none() {
            if let Some(client) = self.tasks.client.clone() {
                // TODO
                // let geng = self.geng.clone();
                // let future = async move {
                //     // let meta = client.get_group_info(group_id).await?;
                //     // let bytes = client.download_level(group_id).await?;

                //     // log::debug!("Decoding downloaded music bytes");
                //     // let music = Rc::new(geng.audio().decode(bytes.to_vec()).await?);

                //     // #[cfg(not(target_arch = "wasm32"))]
                //     // {
                //     //     let bytes = bytes.clone();
                //     //     let save = || -> Result<()> {
                //     //         // Write to fs
                //     //         let music_path =
                //     //             fs::music_path(meta.id);
                //     //         std::fs::create_dir_all(&music_path)?;

                //     //         // let mut file = std::fs::File::create(&music_path.join("music.mp3"))?;
                //     //         // let mut cursor = std::io::Cursor::new(bytes);
                //     //         // std::io::copy(&mut cursor, &mut file)?;
                //     //         std::fs::write(music_path.join("music.mp3"), bytes)?;
                //     //         std::fs::write(
                //     //             music_path.join("meta.toml"),
                //     //             toml::to_string_pretty(&meta)?,
                //     //         )?;

                //     //         log::info!("Music saved successfully");
                //     //         Ok(())
                //     //     };
                //     //     if let Err(err) = save() {
                //     //         log::error!("Failed to save music locally: {:?}", err);
                //     //     }
                //     // }

                //     let group = CachedGroup {};
                //     Ok(group)
                // };
                // self.tasks.download_group = Some(Task::new(&self.geng, future));
            }
        }
    }

    pub fn fetch_music(&mut self) {
        if self.tasks.fetch_music.is_none() {
            if let Some(client) = self.tasks.client.clone() {
                let future = async move {
                    let music = client.get_music_list().await?;
                    Ok(music)
                };
                self.tasks.fetch_music = Some(Task::new(&self.geng, future));
                self.music_list = CacheState::Loading;
            }
        }
    }

    pub fn play_music(&mut self, music_id: Id) {
        if let Some(music) = self.music.get(&music_id) {
            let mut music = Music::from_cache(music);
            music.play();
            self.playing_music = Some(music);
        }
    }

    pub fn download_music(&mut self, music_id: Id) {
        if self.tasks.download_music.is_none() {
            if let Some(client) = self.tasks.client.clone() {
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
                self.tasks.download_music = Some(Task::new(&self.geng, future));
            }
        }
    }

    pub fn poll(&mut self) {
        if let Some(action) = self.tasks.poll() {
            match action {
                CacheAction::MusicList(music) => self.music_list = CacheState::Loaded(music),
                CacheAction::Music(music) => {
                    self.music.insert(music.meta.id, Rc::new(music));
                }
                CacheAction::GroupList(groups) => self.group_list = CacheState::Loaded(groups),
            }
        }
    }

    pub fn synchronize(
        &mut self,
        group_index: Index,
        level_index: usize,
        group_id: Id,
        level_id: Id,
    ) -> Option<Rc<CachedLevel>> {
        if let Some(group) = self.groups.get_mut(group_index) {
            group.meta.id = group_id;
            if let Some(level) = group.levels.get_mut(level_index) {
                let mut new_level: CachedLevel = (**level).clone();
                new_level.meta.id = level_id;
                *level = Rc::new(new_level);
                // TODO: write to fs
                return Some(Rc::clone(level));
            }
        }
        None
    }
}