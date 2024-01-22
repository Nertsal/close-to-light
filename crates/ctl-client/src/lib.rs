pub use ctl_core as core;
use ctl_core::{
    prelude::{
        anyhow::{Context, Result},
        log, serde_json, DeserializeOwned, Id, MusicInfo, MusicUpdate, NewMusic,
    },
    Player, ScoreEntry,
};

use reqwest::{Body, Client, Response, Url};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

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
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn fetch_scores(&self, level: Id) -> Result<Vec<ScoreEntry>> {
        let url = self.url.join(&format!("levels/{}/scores/", level)).unwrap();
        let mut req = self.client.get(url);
        if let Some(key) = &self.api_key {
            req = req.header("api-key", key);
        }

        let response = req.send().await?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn submit_score(&self, level: Id, player: &Player, entry: &ScoreEntry) -> Result<()> {
        let mut req = self
            .client
            .post(self.url.join(&format!("levels/{}/scores/", level)).unwrap())
            .header("player-key", &player.key);
        if let Some(key) = &self.api_key {
            req = req.header("api-key", key);
        }

        let req = req.json(entry);

        let response = req.send().await?;
        get_body(response).await?;
        // TODO: check returned error
        Ok(())
    }

    pub async fn get_music_list(&self) -> Result<Vec<MusicInfo>> {
        let url = self.url.join("music").unwrap();
        let req = self.client.get(url);

        let response = req.send().await.context("when sending request")?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn upload_music(
        &self,
        path: impl AsRef<std::path::Path>,
        music: &NewMusic,
    ) -> Result<Id> {
        let path = path.as_ref();
        let url = self.url.join("music/create").unwrap();

        let file = File::open(path)
            .await
            .context("when opening the music file")?;
        let mut req = self.client.post(url).body(file_to_body(file)).query(&music);
        if let Some(key) = &self.api_key {
            req = req.header("api-key", key);
        }

        let response = req.send().await.context("when sending request")?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn update_music(&self, music: Id, update: &MusicUpdate) -> Result<()> {
        let url = self.url.join(&format!("music/{}", music)).unwrap();

        let mut req = self.client.patch(url).json(update);
        if let Some(key) = &self.api_key {
            req = req.header("api-key", key);
        }

        let response = req.send().await.context("when sending request")?;
        get_body(response).await?;
        Ok(())
    }

    pub async fn music_author_add(&self, music: Id, artist: Id) -> Result<()> {
        let url = self.url.join(&format!("music/{}/authors", music)).unwrap();

        let mut req = self.client.post(url).query(&[("id", artist)]);
        if let Some(key) = &self.api_key {
            req = req.header("api-key", key);
        }

        let response = req.send().await.context("when sending request")?;
        get_body(response).await?;
        Ok(())
    }

    pub async fn music_author_remove(&self, music: Id, artist: Id) -> Result<()> {
        let url = self.url.join(&format!("music/{}/authors", music)).unwrap();

        let mut req = self.client.delete(url).query(&[("id", artist)]);
        if let Some(key) = &self.api_key {
            req = req.header("api-key", key);
        }

        let response = req.send().await.context("when sending request")?;
        get_body(response).await?;
        Ok(())
    }
}

async fn get_body(response: Response) -> Result<String> {
    log::debug!("Response: {:?}", response);
    let body = response
        .text()
        .await
        .context("when reading response body")?;
    log::debug!("Response body: {:?}", body);
    Ok(body)
}

async fn read_json<T: DeserializeOwned>(response: Response) -> Result<T> {
    let body = get_body(response).await?;
    let value = serde_json::from_str(&body).context("when parsing response as json")?;
    Ok(value)
}

fn file_to_body(file: File) -> Body {
    let stream = FramedRead::new(file, BytesCodec::new());
    Body::wrap_stream(stream)
}
