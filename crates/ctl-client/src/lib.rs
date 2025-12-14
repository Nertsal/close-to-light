mod auth;
mod error;
#[cfg(not(target_arch = "wasm32"))]
mod native;

pub use self::error::*;

pub use ctl_core as core;
use ctl_core::{
    ScoreEntry, SubmitScore,
    prelude::{DeserializeOwned, Id, MusicInfo, MusicUpdate, log, serde_json},
    types::{LevelInfo, LevelSetFull, LevelSetInfo, NewMusician},
};

use core::types::LevelSetsQuery;
use std::sync::atomic::AtomicBool;

use reqwest::{Client, RequestBuilder, Response, StatusCode, Url};
use tokio::sync::RwLock;
use tokio_util::bytes::Bytes;

pub type Result<T, E = ClientError> = std::result::Result<T, E>;

pub struct Nertboard {
    #[cfg(feature = "steam")]
    steam: Option<steamworks::Client>,
    pub url: Url,
    client: Client,
    online: AtomicBool,
    auth: RwLock<Option<(String, String)>>,
}

impl Nertboard {
    pub fn new(url: impl reqwest::IntoUrl) -> Result<Self> {
        let client = Client::builder();
        let client = client.build()?;

        Ok(Self {
            #[cfg(feature = "steam")]
            steam: None,
            url: url.into_url()?,
            client,
            online: AtomicBool::new(false),
            auth: RwLock::new(None),
        })
    }

    #[cfg(feature = "steam")]
    pub fn connect_steam(&mut self, steam: steamworks::Client) {
        self.steam = Some(steam);
    }

    /// Whether the server is currently online.
    pub fn is_online(&self) -> bool {
        self.online.load(std::sync::atomic::Ordering::Relaxed)
    }

    async fn send(&self, mut request: RequestBuilder) -> Result<Response> {
        if let Some((user, pass)) = &*self.auth.read().await {
            request = request.basic_auth(user, Some(pass));
        }
        self.check(request.send().await).await
    }

    async fn check(&self, response: Result<Response, reqwest::Error>) -> Result<Response> {
        let online = response.is_ok();
        self.online
            .swap(online, std::sync::atomic::Ordering::Relaxed);
        Ok(response?)
    }

    /// Helper function to send simple get requests expecting json response.
    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let url = self.url.join(url).unwrap();
        let req = self.client.get(url);

        let response = self.send(req).await?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn ping(&self) -> Result<()> {
        let url = self.url.clone();
        let req = self.client.get(url);
        let response = self.send(req).await?;
        get_body(response).await?;
        Ok(())
    }

    pub async fn fetch_scores(&self, level: Id) -> Result<Vec<ScoreEntry>> {
        let url = self.url.join(&format!("level/{level}/scores")).unwrap();
        let req = self.client.get(url);

        let response = self.send(req).await?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn submit_score(&self, level: Id, entry: &SubmitScore) -> Result<()> {
        let req = self
            .client
            .post(self.url.join(&format!("level/{level}/scores")).unwrap())
            .json(entry);

        let response = self.send(req).await?;
        get_body(response).await?;
        // TODO: check returned error
        Ok(())
    }

    pub async fn get_level_info(&self, level: Id) -> Result<LevelInfo> {
        let url = self.url.join(&format!("level/{level}")).unwrap();
        let req = self.client.get(url);

        let response = self.send(req).await?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn upload_group(&self, group: &LevelSetFull, music_id: Id) -> Result<LevelSetInfo> {
        let url = self.url.join("level_set/create").unwrap();
        let body = bincode::serialize(group)?;
        let req = self
            .client
            .post(url)
            .query(&[("music_id", music_id)])
            .body(body);

        let response = self.send(req).await?;
        let group_id: Id = read_json(response).await?;

        self.get_group_info(group_id).await
    }

    pub async fn get_group_list(&self, query: &LevelSetsQuery) -> Result<Vec<LevelSetInfo>> {
        let url = self.url.join("level_sets").unwrap();
        let req = self.client.get(url).query(&query);
        let response = self.send(req).await?;
        let res = read_json(response).await?;
        Ok(res)
    }

    pub async fn get_group_info(&self, group: Id) -> Result<LevelSetInfo> {
        self.get_json(&format!("level_set/{group}")).await
    }

    pub async fn get_music_list(&self) -> Result<Vec<MusicInfo>> {
        self.get_json("music").await
    }

    pub async fn get_music_info(&self, music: Id) -> Result<MusicInfo> {
        self.get_json(&format!("music/{music}")).await
    }

    pub async fn get_music_info_for_group(&self, group: Id) -> Result<MusicInfo> {
        let url = self.url.join("music").unwrap();
        let req = self.client.get(url).query(&[("level_set_id", group)]);

        let response = self.send(req).await?;
        let info: Vec<MusicInfo> = read_json(response).await?;
        let info = info
            .into_iter()
            .next()
            .ok_or(ClientError::UnexpectedFormat(
                "expected a single element".into(),
            ))?;
        Ok(info)
    }

    pub async fn download_music(&self, music: Id) -> Result<Bytes> {
        let url = self.url.join(&format!("music/{music}/download")).unwrap();
        let req = self.client.get(url);

        let response = self.send(req).await?;
        let response = error_for_status(response).await?;
        Ok(response.bytes().await?)
    }

    pub async fn download_music_for_group(&self, group: Id) -> Result<Bytes> {
        let url = self.url.join("music/download").unwrap();
        let req = self.client.get(url).query(&[("level_set_id", group)]);

        let response = self.send(req).await?;
        let response = error_for_status(response).await?;
        Ok(response.bytes().await?)
    }

    pub async fn download_group(&self, group: Id) -> Result<Bytes> {
        let url = self
            .url
            .join(&format!("level_set/{group}/download"))
            .unwrap();
        let req = self.client.get(url);

        let response = self.send(req).await?;
        let response = error_for_status(response).await?;
        Ok(response.bytes().await?)
    }

    pub async fn update_music(&self, music: Id, update: &MusicUpdate) -> Result<()> {
        let url = self.url.join(&format!("music/{music}")).unwrap();

        let req = self.client.patch(url).json(update);

        let response = self.send(req).await?;
        get_body(response).await?;
        Ok(())
    }

    pub async fn music_author_add(&self, music: Id, artist: Id) -> Result<()> {
        let url = self.url.join(&format!("music/{music}/authors")).unwrap();

        let req = self.client.post(url).query(&[("id", artist)]);

        let response = self.send(req).await?;
        get_body(response).await?;
        Ok(())
    }

    pub async fn music_author_remove(&self, music: Id, artist: Id) -> Result<()> {
        let url = self.url.join(&format!("music/{music}/authors")).unwrap();

        let req = self.client.delete(url).query(&[("id", artist)]);

        let response = self.send(req).await?;
        get_body(response).await?;
        Ok(())
    }

    pub async fn create_artist(&self, artist: NewMusician) -> Result<Id> {
        let url = self.url.join("artists").unwrap();

        let req = self.client.post(url).form(&artist);

        let response = self.send(req).await?;
        let res = read_json(response).await?;
        Ok(res)
    }
}

async fn get_body(response: Response) -> Result<String> {
    log::debug!("Response: {response:?}");
    let response = error_for_status(response).await?;
    let body = response.text().await?;
    log::debug!("Response body: {body:?}");
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
