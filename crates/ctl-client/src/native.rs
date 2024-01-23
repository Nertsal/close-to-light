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
}
