use super::*;

use std::path::PathBuf;

use anyhow::Result;
use assets::Assets;
use ctl_client::{
    core::types::{Id, NewArtist},
    Nertboard,
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
        original: bool,
        #[clap(long)]
        bpm: f32,
    },
    /// Update music info.
    Update {
        id: Id,
        #[clap(long)]
        name: Option<String>,
        #[clap(long)]
        public: Option<bool>,
        #[clap(long)]
        original: Option<bool>,
        #[clap(long)]
        bpm: Option<f32>,
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
        user: Option<Id>,
    },
}

impl Command {
    pub async fn execute(
        self,
        geng: Geng,
        assets: Rc<Assets>,
        secrets: Option<Secrets>,
    ) -> Result<()> {
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
                let state = media::MediaState::new(&geng, &assets).with_text(text);
                geng.run_state(state).await;
            }
            Command::Music(music) => {
                let client = client.expect("Cannot update music without secrets");
                match music.command {
                    MusicCommand::Upload {
                        path,
                        name,
                        original,
                        bpm,
                    } => {
                        let music = ctl_client::core::types::NewMusic {
                            name,
                            original,
                            bpm,
                        };
                        log::info!("Uploading music from {:?}: {:?}", path, music);

                        let music_id = client
                            .upload_music(&path, &music)
                            .await
                            .context("failed to upload music")?;
                        log::info!("Music uploaded successfully, id: {}", music_id);
                    }
                    MusicCommand::Update {
                        id,
                        name,
                        public,
                        original,
                        bpm,
                    } => {
                        let update = ctl_client::core::types::MusicUpdate {
                            name,
                            public,
                            original,
                            bpm,
                        };
                        log::info!("Updating music {}: {:#?}", id, update);

                        client
                            .update_music(id, &update)
                            .await
                            .context("failed to update music")?;
                        log::info!("Music updated successfully");
                    }
                    MusicCommand::Author(author) => match author.command {
                        MusicAuthorCommand::Add { music, artist } => {
                            log::info!("Adding artist {} as author of music {}", artist, music);
                            client
                                .music_author_add(music, artist)
                                .await
                                .context("when adding artist as author")?;
                        }
                        MusicAuthorCommand::Remove { music, artist } => {
                            log::info!("Removing artist {} as author of music {}", artist, music);
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
                    ArtistCommand::Create { name, user } => {
                        log::info!("Creating a new artist {} (user: {:?})", name, user);
                        client
                            .create_artist(NewArtist { name, user })
                            .await
                            .context("when creating a new artist")?;
                    }
                }
            }
        }

        Ok(())
    }
}

async fn login(_client: &Nertboard) -> Result<()> {
    // let stdin = std::io::stdin();

    // println!("Logging in...");

    // println!("Username:");
    // let mut username = String::new();
    // stdin.read_line(&mut username)?;
    // username.pop(); // Pop new line
    // if username.is_empty() {
    //     // Skip login
    //     return Ok(());
    // }

    // println!("Password:");
    // let mut password = String::new();
    // stdin.read_line(&mut password)?;
    // password.pop(); // Pop new line

    // client
    //     .login(&ctl_client::core::auth::Credentials { username, password })
    //     .await?
    //     .map_err(|err| anyhow!(err))?;

    // Ok(())
    todo!()
}
