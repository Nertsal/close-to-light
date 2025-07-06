use super::*;

use base64::prelude::*;
use rexie::*;
use serde_wasm_bindgen::Serializer;

#[derive(thiserror::Error, Debug)]
pub enum WebError {
    #[error("rexie: {0}")]
    Rexie(#[from] rexie::Error),
    #[error("serde_wasm_bindgen: {0}")]
    SerdeWasm(#[from] serde_wasm_bindgen::Error),
    #[error("base64: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("ron: {0}")]
    RonSpanned(#[from] ron::error::SpannedError),
    #[error("ron: {0}")]
    Ron(#[from] ron::Error),
    #[error("cbor4ii: {0}")]
    CborEncode(#[from] cbor4ii::serde::EncodeError<std::collections::TryReserveError>),
    #[error("anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, WebError>;

#[derive(Serialize, Deserialize)]
struct GroupItem {
    meta: String,
    data: String,
    music: Option<String>,
}

pub async fn build_database() -> rexie::Result<Rexie> {
    // Create a new database
    let rexie = Rexie::builder("close-to-light")
        .version(1)
        .add_object_store(ObjectStore::new("groups"))
        .build()
        .await?;

    Ok(rexie)
}

pub async fn load_groups_all(geng: &Geng, rexie: &Rexie) -> Result<Vec<LocalGroup>> {
    let transaction = rexie.transaction(&["groups"], TransactionMode::ReadOnly)?;

    let groups = transaction.store("groups")?;

    let raw_items = groups.get_all(None, None, None, None).await?;
    let mut items = Vec::with_capacity(raw_items.len());
    for (key, item) in raw_items {
        let process_item = async |key, item| -> Result<LocalGroup> {
            let key: String = serde_wasm_bindgen::from_value(key)?;
            let path = super::all_groups_path().join(key);
            let item: GroupItem = serde_wasm_bindgen::from_value(item)?;

            let data = BASE64_STANDARD.decode(&item.data)?;
            let group: LevelSet = decode_group(&data)?;

            let music_bytes = BASE64_STANDARD.decode(&item.meta)?;
            let meta: GroupMeta = ron::de::from_bytes(&music_bytes)?;

            let music = match &item.music {
                None => None,
                Some(music) => {
                    let data = BASE64_STANDARD.decode(music)?;
                    let music = geng.audio().decode(data).await?;
                    Some(Rc::new(LocalMusic::new(
                        meta.music.clone().unwrap_or_default(),
                        music,
                        music_bytes.into(),
                    )))
                }
            };

            Ok(LocalGroup {
                path,
                meta,
                music,
                data: group,
            })
        };

        if let Ok(group) = process_item(key, item).await {
            items.push(group);
        }
    }

    Ok(items)
}

pub async fn save_group(
    rexie: &Rexie,
    group: &CachedGroup,
    music: Option<&[u8]>,
    id: &str,
) -> Result<()> {
    log::debug!("Storing group {:?} into browser storage", id);

    let transaction = rexie.transaction(&["groups"], TransactionMode::ReadWrite)?;

    let store = transaction.store("groups")?;

    let data = cbor4ii::serde::to_vec(Vec::new(), &group.local.data)?;
    let data = BASE64_STANDARD.encode(&data);

    let music = music.map(|music| BASE64_STANDARD.encode(&music));

    let meta = ron::ser::to_string(&group.local.meta)?;
    let meta = BASE64_STANDARD.encode(&meta);

    let item = GroupItem { data, music, meta };

    let serializer = Serializer::json_compatible();
    let item = item.serialize(&serializer)?;
    let id = id.serialize(&serializer)?;

    store.put(&item, Some(&id)).await?;

    transaction.done().await?;

    Ok(())
}

pub async fn remove_group(rexie: &Rexie, id: &str) -> Result<()> {
    log::debug!("Deleting group {:?} from browser storage", id);

    let transaction = rexie.transaction(&["groups"], TransactionMode::ReadWrite)?;

    let store = transaction.store("groups")?;

    let serializer = Serializer::json_compatible();
    let id = id.serialize(&serializer)?;

    store.delete(&id).await?;

    transaction.done().await?;

    Ok(())
}
