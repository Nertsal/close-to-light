use crate::{prelude::*, task::Task};

use std::path::{Path, PathBuf};

use anyhow::Result;
use ctl_client::Nertboard;

pub struct LevelCache {
    geng: Geng,
    pub tasks: CacheTasks,

    /// List of downloadable music.
    pub music_list: Vec<MusicInfo>,

    pub music: HashMap<Id, Rc<CachedMusic>>,
    pub groups: Vec<CachedGroup>,
}

pub struct CacheTasks {
    geng: Geng,
    client: Option<Arc<Nertboard>>,
    fetch_music: Option<Task<anyhow::Result<Vec<MusicInfo>>>>,
    download_music: Option<Task<anyhow::Result<CachedMusic>>>,
}

#[derive(Debug)]
enum CacheAction {
    MusicList(Vec<MusicInfo>),
    Music(CachedMusic),
}

pub struct CachedMusic {
    pub meta: MusicInfo,
    pub music: Rc<geng::Sound>,
}

#[derive(Debug)]
pub struct CachedGroup {
    pub path: PathBuf,
    pub meta: GroupMeta,
    pub music: Option<Rc<CachedMusic>>,
    pub levels: Vec<Rc<CachedLevel>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupMeta {
    pub id: Id,
    pub music: Id,
}

#[derive(Debug)]
pub struct CachedLevel {
    pub path: PathBuf,
    pub meta: LevelInfo, // TODO: maybe Rc to reduce String allocations
    pub data: Level,     // TODO: Rc
    /// Hash code of the level.
    pub hash: String,
}

impl Debug for CachedMusic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CachedMusic")
            .field("metal", &self.meta)
            .field("music", &"<data>")
            .finish()
    }
}

impl CacheTasks {
    pub fn new(geng: &Geng, client: Option<&Arc<Nertboard>>) -> Self {
        Self {
            geng: geng.clone(),
            client: client.cloned(),
            fetch_music: None,
            download_music: None,
        }
    }

    pub fn fetch_music(&mut self) {
        if self.fetch_music.is_none() {
            if let Some(client) = self.client.clone() {
                let future = async move {
                    let music = client.get_music_list().await?;
                    Ok(music)
                };
                self.fetch_music = Some(Task::new(&self.geng, future));
            }
        }
    }

    pub fn download_music(&mut self, music_id: Id) {
        if self.download_music.is_none() {
            if let Some(client) = self.client.clone() {
                let geng = self.geng.clone();
                let future = async move {
                    let meta = client.get_music_info(music_id).await?;
                    let bytes = client.download_music(music_id).await?;

                    let music = Rc::new(geng.audio().decode(bytes.to_vec()).await?);

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let bytes = bytes.clone();
                        let save = || -> Result<()> {
                            // Write to fs
                            let music_path =
                                preferences::base_path().join(format!("music/{}", meta.id));
                            std::fs::create_dir_all(&music_path)?;

                            // let mut file = std::fs::File::create(&music_path.join("music.mp3"))?;
                            // let mut cursor = std::io::Cursor::new(bytes);
                            // std::io::copy(&mut cursor, &mut file)?;
                            std::fs::write(music_path.join("music.mp3"), bytes)?;
                            std::fs::write(
                                music_path.join("meta.toml"),
                                toml::to_string_pretty(&meta)?,
                            )?;

                            Ok(())
                        };
                        if let Err(err) = save() {
                            log::error!("Failed to save music locally: {:?}", err);
                        }
                    }

                    let music = CachedMusic { meta, music };
                    Ok(music)
                };
                self.download_music = Some(Task::new(&self.geng, future));
            }
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
            tasks: CacheTasks::new(geng, client),

            music_list: Vec::new(),

            music: HashMap::new(),
            groups: Vec::new(),
        }
    }

    /// Load from the local storage.
    pub async fn load(client: Option<&Arc<Nertboard>>, geng: &Geng) -> Result<Self> {
        // TODO: report failures but continue working

        #[cfg(target_arch = "wasm32")]
        {
            return Ok(Self::new(client, geng));
        }

        log::info!("Loading local storage");
        let base_path = preferences::base_path();

        // let mut music = HashMap::new();
        // for entry in std::fs::read_dir(base_path.join("music"))? {
        //     let entry = entry?;
        //     let path = entry.path();
        //     if !path.is_dir() {
        //         log::error!("Unexpected file in music dir: {:?}", path);
        //         continue;
        //     }

        //     let id: Id = entry
        //         .file_name()
        //         .to_str()
        //         .ok_or(anyhow!("Directory name is not valid UTF-8"))?
        //         .parse()?;

        //     let m = CachedMusic::load(manager, &path).await?;
        //     music.insert(id, Rc::new(m));
        // }

        let mut local = Self::new(client, geng);

        for entry in std::fs::read_dir(base_path.join("levels"))? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                log::error!("Unexpected file in levels dir: {:?}", path);
                continue;
            }

            local.load_group_all(&path).await?;
        }

        Ok(local)
    }

    pub async fn load_level(
        &mut self,
        level_path: impl AsRef<std::path::Path>,
    ) -> Result<(Rc<CachedMusic>, Rc<CachedLevel>)> {
        let level_path = level_path.as_ref();
        let (level_path, group_path) = if level_path.is_dir() {
            (
                level_path.join("level.json"),
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
        let group = self.groups.last().unwrap();

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

        let meta_path = group_path.join("meta.toml");
        let meta: GroupMeta = file::load_detect(&meta_path).await?;

        let music = match self.music.get(&meta.music) {
            Some(music) => Some(music.clone()),
            None => {
                let music_path = preferences::base_path().join(format!("music/{}", meta.music));
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
        self.groups.push(group);

        Ok(())
    }

    /// Load the group and all levels from it.
    async fn load_group_all(&mut self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let group_path = path.as_ref();
        self.load_group_empty(group_path).await?;

        // If `load_group_empty` succedes, the group is pushed to the end
        let group = self.groups.last_mut().unwrap();

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
        self.groups.push(group);
        // TODO: write to fs
    }

    pub fn new_level(&mut self, group: usize, meta: LevelInfo) {
        if let Some(group) = self.groups.get_mut(group) {
            let level = CachedLevel::new(meta);
            group.levels.push(Rc::new(level));
            // TODO: write to fs
        }
    }

    pub fn poll(&mut self) {
        if let Some(action) = self.tasks.poll() {
            match action {
                CacheAction::MusicList(music) => self.music_list = music,
                CacheAction::Music(music) => {
                    self.music.insert(music.meta.id, Rc::new(music));
                }
            }
        }
    }
}

impl CachedMusic {
    pub async fn load(manager: &geng::asset::Manager, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let meta_path = path.join("meta.toml");
        let meta: MusicInfo = file::load_detect(&meta_path).await?;

        let file_path = path.join("music.mp3");
        let file: geng::Sound = geng::asset::Load::load(
            manager,
            &file_path,
            &geng::asset::SoundOptions { looped: false },
        )
        .await?;

        Ok(Self {
            meta,
            music: Rc::new(file),
        })
    }
}

impl CachedGroup {
    pub fn new(meta: GroupMeta) -> Self {
        Self {
            path: PathBuf::new(), // TODO
            meta,
            music: None,
            levels: Vec::new(),
        }
    }
}

impl CachedLevel {
    pub fn new(meta: LevelInfo) -> Self {
        Self {
            path: PathBuf::new(), // TODO
            meta,
            data: Level::new(),
            hash: String::new(),
        }
    }

    pub async fn load(_manager: &geng::asset::Manager, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let meta_path = path.join("meta.toml");
        let meta: LevelInfo = file::load_detect(&meta_path).await?;

        let level_path = path.join("level.json");
        let level: Level = file::load_detect(&level_path).await?;

        let hash = {
            use data_encoding::HEXLOWER;
            use sha2::{Digest, Sha256};

            let mut hasher = Sha256::new();
            let mut reader = std::io::BufReader::new(std::fs::File::open(&level_path)?);
            let mut buffer = [0; 1024];
            loop {
                let count = reader.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                hasher.update(&buffer[..count]);
            }
            HEXLOWER.encode(hasher.finalize().as_ref())
        };

        Ok(Self {
            path: level_path.to_path_buf(),
            meta,
            data: level,
            hash,
        })
    }
}
