pub use ctl_core as core;
use ctl_core::{prelude::Uuid, Player, ScoreEntry};

use reqwest::{Client, Result, Url};

pub struct Nertboard {
    url: Url,
    api_key: Option<String>,
    client: Client,
}

impl Nertboard {
    pub fn new(url: impl reqwest::IntoUrl, api_key: Option<String>) -> Result<Self> {
        Ok(Self {
            url: url.into_url()?,
            api_key,
            client: Client::new(),
        })
    }

    pub async fn create_player(&self, name: &str) -> Result<Player> {
        let url = self.url.join("player/create/").unwrap();
        let req = self.client.post(url).json(&name);
        let response = req.send().await?;
        response.json().await
    }

    pub async fn fetch_scores(&self, level: Uuid) -> Result<Vec<ScoreEntry>> {
        let url = self
            .url
            .join(&format!("levels/{}/scores/", level.simple()))
            .unwrap();
        let mut req = self.client.get(url);
        if let Some(key) = &self.api_key {
            req = req.header("api-key", key);
        }

        let response = req.send().await?;
        response.json().await
    }

    pub async fn submit_score(
        &self,
        level: Uuid,
        player: &Player,
        entry: &ScoreEntry,
    ) -> Result<()> {
        let mut req = self
            .client
            .post(
                self.url
                    .join(&format!("levels/{}/scores/", level.simple()))
                    .unwrap(),
            )
            .header("player-key", &player.key);
        if let Some(key) = &self.api_key {
            req = req.header("api-key", key);
        }

        let req = req.json(entry);

        let _response = req.send().await?;
        // TODO: check returned error
        Ok(())
    }
}
