use self::prelude::Context;

use super::*;

use std::path::PathBuf;

use anyhow::Result;
use ctl_client::{
    Nertboard,
    core::{
        prelude::Uuid,
        types::{Id, NewMusician, UserLogin},
    },
};

#[derive(clap::Subcommand)]
pub enum Command {
    Play {
        level: String,
        diff: String,
        start_time: Option<String>,
    },
    Edit {
        level: String,
        diff: Option<String>,
    },
    /// Picture generation and similar.
    Media {
        #[clap(long)]
        text: Option<String>,
        #[clap(long)]
        picture: Option<PathBuf>,
    },
    Music(MusicArgs),
    Artist(ArtistArgs),
}

#[derive(clap::Args)]
pub struct MusicArgs {
    #[command(subcommand)]
    pub command: MusicCommand,
}

#[derive(clap::Args)]
pub struct ArtistArgs {
    #[command(subcommand)]
    pub command: ArtistCommand,
}

#[derive(clap::Subcommand)]
pub enum MusicCommand {
    Author(MusicAuthorArgs),
    /// Upload music to the server.
    Upload {
        path: PathBuf,
        #[clap(long)]
        name: String,
        #[clap(long)]
        romanized_name: Option<String>,
    },
    /// Update music info.
    Update {
        id: Id,
        #[clap(long)]
        name: Option<String>,
        #[clap(long)]
        original: Option<bool>,
        #[clap(long)]
        featured: Option<bool>,
    },
}

#[derive(clap::Args)]
pub struct MusicAuthorArgs {
    #[command(subcommand)]
    pub command: MusicAuthorCommand,
}

#[derive(clap::Subcommand)]
pub enum MusicAuthorCommand {
    Add {
        #[clap(long)]
        music: Id,
        #[clap(long)]
        artist: Id,
    },
    Remove {
        #[clap(long)]
        music: Id,
        #[clap(long)]
        artist: Id,
    },
}

#[derive(clap::Subcommand)]
pub enum ArtistCommand {
    Create {
        name: String,
        #[clap(long)]
        romanized: Option<String>,
        #[clap(long)]
        user: Option<Id>,
    },
}

impl Command {
    pub async fn execute(self, context: Context, secrets: Option<Secrets>) -> Result<()> {
        async fn init_client(secrets: Option<&Secrets>) -> Result<Option<Arc<Nertboard>>> {
            if let Some(secrets) = &secrets {
                let client = ctl_client::Nertboard::new(&secrets.leaderboard.url)
                    .context("Client initialization failed")?;
                login(&client).await?;
                Ok(Some(Arc::new(client)))
            } else {
                Ok(None)
            }
        }

        let find_group = |local: &ctl_local::LevelCacheImpl, level: &str| {
            let Some((index, cached)) = local.groups.iter().find(|(_, group)| {
                &*group.local.meta.music.name == level
                    || &*group.local.meta.music.romanized == level
            }) else {
                log::error!("Level {:?} not found, available levels:\n", level);
                for (_, group) in local.groups.iter() {
                    log::error!("{}", group.local.meta.music.name);
                }
                anyhow::bail!("Level {:?} not found", level);
            };
            Ok((index, cached.clone()))
        };
        let find_diff = |group: &ctl_local::CachedGroup, diff: &str| {
            let diff_index = if let Ok(index) = diff.parse::<usize>() {
                index
            } else {
                let Some(index) = group
                    .local
                    .meta
                    .levels
                    .iter()
                    .position(|level| &*level.name == diff)
                else {
                    anyhow::bail!("Difficulty named {:?} not found", diff);
                };
                index
            };

            let Some(data) = group.local.data.levels.get(diff_index).cloned() else {
                anyhow::bail!("Difficulty indexed {} not found", diff_index);
            };
            let Some(meta) = group.local.meta.levels.get(diff_index).cloned() else {
                anyhow::bail!("Difficulty indexed {} lacks metadata", diff_index);
            };

            Ok((diff_index, ctl_logic::LevelFull { meta, data }))
        };

        match self {
            Command::Play {
                level,
                diff,
                start_time,
            } => {
                let start_time = match start_time {
                    None => None,
                    Some(time) => {
                        if let Some(end_of_number) =
                            time.find(|c: char| c != '.' && !c.is_ascii_digit())
                        {
                            let (number, unit) = time.split_at(end_of_number);
                            let number: f32 = number.parse()?;
                            let scale = match unit {
                                "ms" => 1e-3,
                                "s" => 1.0,
                                "m" => 60.0,
                                _ => anyhow::bail!("Unknown time unit: {:?}", unit),
                            };
                            Some(ctl_logic::seconds_to_time(r32(number * scale)))
                        } else {
                            let number: f32 = time.parse()?;
                            Some(ctl_logic::seconds_to_time(r32(number)))
                        }
                    }
                };

                let ((group_index, group), (diff_index, diff)) = {
                    let local = context.local.inner.borrow();
                    let group = find_group(&local, &level)?;
                    let diff = find_diff(&group.1, &diff)?;
                    (group, diff)
                };

                let level = ctl_logic::PlayLevel {
                    start_time: start_time.unwrap_or(0),
                    level: diff,
                    group: ctl_logic::PlayGroup {
                        group_index,
                        music: group.local.music.clone(),
                        cached: group,
                    },
                    level_index: diff_index,
                    config: ctl_logic::LevelConfig::default(),
                    transition_button: None,
                };

                let state = crate::game::Game::new(
                    context.clone(),
                    level,
                    ctl_local::Leaderboard::new(&context.geng, None, &context.local.fs),
                );
                context.geng.run_state(state).await;
            }
            Command::Edit { level, diff } => {
                let (group_index, group, level) = {
                    let local = context.local.inner.borrow();
                    let (group_index, group) = find_group(&local, &level)?;
                    let level = match diff {
                        None => None,
                        Some(diff) => Some(find_diff(&group, &diff)?),
                    };
                    (group_index, group, level)
                };

                let group = ctl_logic::PlayGroup {
                    group_index,
                    music: group.local.music.clone(),
                    cached: group,
                };

                let config: crate::editor::EditorConfig = geng::asset::Load::load(
                    context.geng.asset_manager(),
                    &run_dir().join("assets").join("editor.ron"),
                    &(),
                )
                .await
                .expect("failed to load editor config");

                let state = if let Some((level_index, level)) = level {
                    let level = ctl_logic::PlayLevel {
                        group,
                        level_index,
                        level,
                        config: ctl_logic::LevelConfig::default(),
                        start_time: 0,
                        transition_button: None,
                    };
                    crate::editor::EditorState::new_level(context.clone(), config, level)
                } else {
                    crate::editor::EditorState::new_group(context.clone(), config, group)
                };
                context.geng.run_state(state).await;
            }
            Command::Media { text, picture } => {
                let mut state = media::MediaState::new(context.clone());
                if let Some(text) = text {
                    state.set_text(text);
                }
                if let Some(path) = picture {
                    let picture: ugli::Texture = geng::asset::Load::load(
                        context.geng.asset_manager(),
                        &path,
                        &geng::asset::TextureOptions {
                            filter: ugli::Filter::Nearest,
                            ..Default::default()
                        },
                    )
                    .await?;
                    state.set_picture(picture);
                }
                context.geng.run_state(state).await;
            }
            Command::Music(music) => {
                let client = init_client(secrets.as_ref())
                    .await?
                    .expect("Cannot update music without secrets");
                match music.command {
                    MusicCommand::Upload {
                        path,
                        name,
                        romanized_name,
                    } => {
                        let music = ctl_client::core::types::NewMusic {
                            romanized_name: romanized_name.unwrap_or(name.clone()),
                            name,
                        };
                        log::info!("Uploading music from {path:?}: {music:?}");

                        let music_id = client
                            .upload_music_file(&path, &music)
                            .await
                            .context("failed to upload music")?;
                        log::info!("Music uploaded successfully, id: {music_id}");
                    }
                    MusicCommand::Update {
                        id,
                        name,
                        original,
                        featured,
                    } => {
                        let update = ctl_client::core::types::MusicUpdate {
                            name,
                            original,
                            featured,
                        };
                        log::info!("Updating music {id}: {update:#?}");

                        client
                            .update_music(id, &update)
                            .await
                            .context("failed to update music")?;
                        log::info!("Music updated successfully");
                    }
                    MusicCommand::Author(author) => match author.command {
                        MusicAuthorCommand::Add { music, artist } => {
                            log::info!("Adding artist {artist} as author of music {music}");
                            client
                                .music_author_add(music, artist)
                                .await
                                .context("when adding artist as author")?;
                        }
                        MusicAuthorCommand::Remove { music, artist } => {
                            log::info!("Removing artist {artist} as author of music {music}");
                            client
                                .music_author_remove(music, artist)
                                .await
                                .context("when adding artist as author")?;
                        }
                    },
                }
            }
            Command::Artist(artist) => {
                let client = init_client(secrets.as_ref())
                    .await?
                    .expect("Cannot update artists without secrets");
                match artist.command {
                    ArtistCommand::Create {
                        name,
                        romanized,
                        user,
                    } => {
                        log::info!("Creating a new artist {name} (user: {user:?})");
                        client
                            .create_artist(NewMusician {
                                romanized_name: romanized.unwrap_or(name.clone()),
                                name,
                                user,
                            })
                            .await
                            .context("when creating a new artist")?;
                    }
                }
            }
        }

        Ok(())
    }
}

async fn login(client: &Nertboard) -> Result<()> {
    let user: Option<UserLogin> = preferences::load(ctl_local::PLAYER_LOGIN_STORAGE);

    if let Some(user) = user {
        let user = client
            .login_token(user.id, &user.token)
            .await?
            .map_err(|err| anyhow!(err))?;
        log::debug!("logged in as {}", user.name);
    } else {
        let state = Uuid::new_v4().to_string();
        webbrowser::open(&format!("{}&state={}", ctl_core::DISCORD_LOGIN_URL, state))?;
        let user = client
            .login_external(state)
            .await?
            .map_err(|err| anyhow!(err))?;
        preferences::save(ctl_local::PLAYER_LOGIN_STORAGE, &user);
    }

    Ok(())
}
