mod auth;
#[cfg(not(target_arch = "wasm32"))]
mod native;

use core::{
    model::Level,
    types::{GroupInfo, LevelInfo, NewLevel},
};

pub use ctl_core as core;
use ctl_core::{
    prelude::{
        anyhow::{Context, Result},
        log, serde_json, DeserializeOwned, Id, MusicInfo, MusicUpdate,
    },
    ScoreEntry, SubmitScore,
};

use reqwest::{Client, Response, Url};
use tokio_util::bytes::Bytes;

pub struct Nertboard {
    url: Url,
    client: Client,
}

impl Nertboard {
    pub fn new(url: impl reqwest::IntoUrl) -> Result<Self> {
        let client = Client::builder();
        #[cfg(not(target_arch = "wasm32"))]
        let client = client.cookie_store(true); // NOTE: cookie_store does not work on wasm
        let client = client.build().context("when building the client")?;

        Ok(Self {
            url: url.into_url()?,
            client,
        })
    }

    /// Helper function to send simple get requests expecting json response.
    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let url = self.url.join(url).unwrap();
        let req = self.client.get(url);

        let response = req.send().await.context("when sending request")?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn fetch_scores(&self, level: Id) -> Result<Vec<ScoreEntry>> {
        let url = self.url.join(&format!("level/{}/scores/", level)).unwrap();
        let req = self.client.get(url);

        let response = req.send().await?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn submit_score(&self, level: Id, entry: &SubmitScore) -> Result<()> {
        let req = self
            .client
            .post(self.url.join(&format!("level/{}/scores/", level)).unwrap())
            .json(entry);

        let response = req.send().await?;
        get_body(response).await?;
        // TODO: check returned error
        Ok(())
    }

    pub async fn get_level_info(&self, level: Id) -> Result<LevelInfo> {
        let url = self.url.join(&format!("level/{}", level)).unwrap();
        let req = self.client.get(url);

        let response = req.send().await.context("when sending request")?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn upload_level(&self, query: NewLevel, level: &Level) -> Result<Id> {
        let url = self.url.join("level/create").unwrap();
        let body = bincode::serialize(level)?;
        let req = self.client.post(url).query(&query).body(body);

        let response = req.send().await.context("when sending request")?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn get_group_list(&self) -> Result<Vec<GroupInfo>> {
        self.get_json("groups").await
    }

    pub async fn get_group_info(&self, group: Id) -> Result<GroupInfo> {
        self.get_json(&format!("group/{}", group)).await
    }

    pub async fn create_group(&self, music: Id) -> Result<Id> {
        let url = self.url.join("group/create").unwrap();
        let req = self.client.post(url).query(&[("id", music)]);

        let response = req.send().await.context("when sending request")?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn get_music_list(&self) -> Result<Vec<MusicInfo>> {
        self.get_json("music").await
    }

    pub async fn get_music_info(&self, music: Id) -> Result<MusicInfo> {
        self.get_json(&format!("music/{}", music)).await
    }

    pub async fn download_music(&self, music: Id) -> Result<Bytes> {
        let url = self.url.join(&format!("music/{}/download", music)).unwrap();
        let req = self.client.get(url);

        let response = req.send().await?;
        Ok(response.bytes().await?)
    }

    pub async fn download_level(&self, level: Id) -> Result<Bytes> {
        let url = self.url.join(&format!("level/{}/download", level)).unwrap();
        let req = self.client.get(url);

        let response = req.send().await?;
        Ok(response.bytes().await?)
    }

    pub async fn update_music(&self, music: Id, update: &MusicUpdate) -> Result<()> {
        let url = self.url.join(&format!("music/{}", music)).unwrap();

        let req = self.client.patch(url).json(update);

        let response = req.send().await.context("when sending request")?;
        get_body(response).await?;
        Ok(())
    }

    pub async fn music_author_add(&self, music: Id, artist: Id) -> Result<()> {
        let url = self.url.join(&format!("music/{}/authors", music)).unwrap();

        let req = self.client.post(url).query(&[("id", artist)]);

        let response = req.send().await.context("when sending request")?;
        get_body(response).await?;
        Ok(())
    }

    pub async fn music_author_remove(&self, music: Id, artist: Id) -> Result<()> {
        let url = self.url.join(&format!("music/{}/authors", music)).unwrap();

        let req = self.client.delete(url).query(&[("id", artist)]);

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
