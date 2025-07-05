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
    /// Just display some dithered text on screen.
    Text {
        text: String,
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
        let client = if let Some(secrets) = &secrets {
            let client = ctl_client::Nertboard::new(&secrets.leaderboard.url)
                .context("Client initialization failed")?;
            login(&client).await?;
            Some(client)
        } else {
            None
        };

        match self {
            Command::Text { text } => {
                let state = media::MediaState::new(context.clone()).with_text(text);
                context.geng.run_state(state).await;
            }
            Command::Music(music) => {
                let client = client.expect("Cannot update music without secrets");
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
                            .upload_music(&path, &music)
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
                let client = client.expect("Cannot update artists without secrets");
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
