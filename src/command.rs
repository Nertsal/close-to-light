use super::*;

use anyhow::Result;
use assets::Assets;
use ctl_client::core::types::Id;

#[derive(clap::Subcommand)]
pub enum Command {
    /// Just display some dithered text on screen.
    Text { text: String },
    /// Upload music to the server.
    MusicUpload {
        path: PathBuf,
        #[clap(long)]
        name: String,
        #[clap(long)]
        original: bool,
        #[clap(long)]
        bpm: f32,
    },
    /// Update music info.
    MusicUpdate {
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

impl Command {
    pub async fn execute(
        self,
        geng: Geng,
        assets: Rc<Assets>,
        secrets: Option<Secrets>,
    ) -> Result<()> {
        match self {
            Command::Text { text } => {
                let state = media::MediaState::new(&geng, &assets).with_text(text);
                geng.run_state(state).await;
            }
            Command::MusicUpload {
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
                let secrets = secrets.context("Cannot upload music without secrets")?;

                let future = async move {
                    let client = ctl_client::Nertboard::new(
                        &secrets.leaderboard.url,
                        Some(secrets.leaderboard.key),
                    )
                    .context("Client initialization failed")?;
                    let music_id = client
                        .upload_music(&path, &music)
                        .await
                        .context("failed to upload music")?;
                    log::info!("Music uploaded successfully, id: {}", music_id);
                    anyhow::Ok(())
                };
                execute_task(future)??;
            }
            Command::MusicUpdate {
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
                let secrets = secrets.expect("Cannot update music without secrets");

                let future = async move {
                    let client = ctl_client::Nertboard::new(
                        &secrets.leaderboard.url,
                        Some(secrets.leaderboard.key),
                    )
                    .context("Client initialization failed")?;
                    client
                        .update_music(id, &update)
                        .await
                        .context("failed to update music")?;
                    log::info!("Music updated successfully");
                    anyhow::Ok(())
                };
                execute_task(future)??;
            }
        }

        Ok(())
    }
}

fn execute_task<T: Send + Sync + 'static>(
    future: impl Future<Output = T> + Send + Sync + 'static,
) -> Result<T> {
    let mut task = task::Task::new(future);
    loop {
        if let Some(res) = task.poll() {
            return res.context("when executing a task");
        }
    }
}
