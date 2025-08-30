use super::*;

use ctl_core::prelude::NewMusic;

use reqwest::Body;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

fn file_to_body(file: File) -> Body {
    let stream = FramedRead::new(file, BytesCodec::new());
    Body::wrap_stream(stream)
}

impl Nertboard {
    pub async fn upload_music_file(
        &self,
        path: impl AsRef<std::path::Path>,
        music: &NewMusic,
    ) -> Result<Id> {
        let file = File::open(path.as_ref()).await?;
        let body = file_to_body(file);
        self.upload_music(body, music).await
    }

    pub async fn upload_music_bytes(&self, bytes: &[u8], music: &NewMusic) -> Result<Id> {
        let body = Body::from(bytes.to_vec());
        self.upload_music(body, music).await
    }

    async fn upload_music(&self, body: Body, music: &NewMusic) -> Result<Id> {
        let url = self.url.join("music/create").unwrap();

        let req = self.client.post(url).body(body).query(&music);

        let response = self.send(req).await?;
        let res = read_json(response).await?;
        Ok(res)
    }
}
