mod auth;
mod error;
#[cfg(not(target_arch = "wasm32"))]
mod native;

pub use self::error::*;

pub use ctl_core as core;
use ctl_core::{
    prelude::{log, serde_json, DeserializeOwned, Id, MusicInfo, MusicUpdate},
    types::{GroupInfo, LevelInfo, LevelSet, NewArtist},
    ScoreEntry, SubmitScore,
};

use std::sync::atomic::AtomicBool;

use reqwest::{Client, Response, StatusCode, Url};
use tokio_util::bytes::Bytes;

pub type Result<T, E = ClientError> = std::result::Result<T, E>;

pub struct Nertboard {
    pub url: Url,
    client: Client,
    online: AtomicBool,
}

impl Nertboard {
    pub fn new(url: impl reqwest::IntoUrl) -> Result<Self> {
        let client = Client::builder();
        #[cfg(not(target_arch = "wasm32"))]
        let client = client.cookie_store(true); // NOTE: cookie_store does not work on wasm
        let client = client.build()?;

        Ok(Self {
            url: url.into_url()?,
            client,
            online: AtomicBool::new(false),
        })
    }

    /// Whether the server is currently online.
    pub fn is_online(&self) -> bool {
        self.online.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn check(&self, response: Result<Response, reqwest::Error>) -> Result<Response> {
        // let online = !matches!(&response, Err(err) if err.is_connect()); // TODO: fix web
        let online = response.is_ok();
        self.online
            .swap(online, std::sync::atomic::Ordering::Relaxed);
        Ok(response?)
    }

    /// Helper function to send simple get requests expecting json response.
    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let url = self.url.join(url).unwrap();
        let req = self.client.get(url);

        let response = self.check(req.send().await)?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn fetch_scores(&self, level: Id) -> Result<Vec<ScoreEntry>> {
        let url = self.url.join(&format!("level/{}/scores", level)).unwrap();
        let req = self.client.get(url);

        let response = self.check(req.send().await)?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn submit_score(&self, level: Id, entry: &SubmitScore) -> Result<()> {
        let req = self
            .client
            .post(self.url.join(&format!("level/{}/scores", level)).unwrap())
            .json(entry);

        let response = self.check(req.send().await)?;
        get_body(response).await?;
        // TODO: check returned error
        Ok(())
    }

    pub async fn get_level_info(&self, level: Id) -> Result<LevelInfo> {
        let url = self.url.join(&format!("level/{}", level)).unwrap();
        let req = self.client.get(url);

        let response = self.check(req.send().await)?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn upload_group(&self, group: &LevelSet) -> Result<GroupInfo> {
        let url = self.url.join("group/create").unwrap();
        let body = bincode::serialize(group)?;
        let req = self.client.post(url).body(body);

        let response = self.check(req.send().await)?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn get_group_list(&self) -> Result<Vec<GroupInfo>> {
        self.get_json("groups").await
    }

    pub async fn get_group_info(&self, group: Id) -> Result<GroupInfo> {
        self.get_json(&format!("group/{}", group)).await
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

        let response = self.check(req.send().await)?;
        let response = error_for_status(response).await?;
        Ok(response.bytes().await?)
    }

    pub async fn download_group(&self, group: Id) -> Result<Bytes> {
        let url = self.url.join(&format!("group/{}/download", group)).unwrap();
        let req = self.client.get(url);

        let response = self.check(req.send().await)?;
        let response = error_for_status(response).await?;
        Ok(response.bytes().await?)
    }

    pub async fn update_music(&self, music: Id, update: &MusicUpdate) -> Result<()> {
        let url = self.url.join(&format!("music/{}", music)).unwrap();

        let req = self.client.patch(url).json(update);

        let response = self.check(req.send().await)?;
        get_body(response).await?;
        Ok(())
    }

    pub async fn music_author_add(&self, music: Id, artist: Id) -> Result<()> {
        let url = self.url.join(&format!("music/{}/authors", music)).unwrap();

        let req = self.client.post(url).query(&[("id", artist)]);

        let response = self.check(req.send().await)?;
        get_body(response).await?;
        Ok(())
    }

    pub async fn music_author_remove(&self, music: Id, artist: Id) -> Result<()> {
        let url = self.url.join(&format!("music/{}/authors", music)).unwrap();

        let req = self.client.delete(url).query(&[("id", artist)]);

        let response = self.check(req.send().await)?;
        get_body(response).await?;
        Ok(())
    }

    pub async fn create_artist(&self, artist: NewArtist) -> Result<Id> {
        let url = self.url.join("artists").unwrap();

        let req = self.client.post(url).form(&artist);

        let response = self.check(req.send().await)?;
        let res = read_json(response).await?;
        Ok(res)
    }
}

async fn get_body(response: Response) -> Result<String> {
    log::debug!("Response: {:?}", response);
    let response = error_for_status(response).await?;
    let body = response.text().await?;
    log::debug!("Response body: {:?}", body);
    Ok(body)
}

async fn read_json<T: DeserializeOwned>(response: Response) -> Result<T> {
    let response = error_for_status(response).await?;
    let body = get_body(response).await?;
    let value = serde_json::from_str(&body)?;
    Ok(value)
}

async fn error_for_status(response: Response) -> Result<Response> {
    let status = response.status();
    if let StatusCode::NOT_FOUND = status {
        Err(ClientError::NotFound)
    } else if status.is_server_error() {
        let body = response.text().await?;
        Err(ClientError::Server(body))
    } else if status.is_client_error() {
        let body = response.text().await?;
        Err(ClientError::Client(body))
    } else {
        Ok(response)
    }
}
