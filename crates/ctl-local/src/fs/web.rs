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
    #[error("UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
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
    id: String,
    meta: String,
    data: String,
    music: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ScoresItem {
    level_hash: String,
    scores: Vec<SavedScore>,
}

pub async fn build_database() -> rexie::Result<Rexie> {
    // Create a new database
    let rexie = Rexie::builder("close-to-light")
        .version(2)
        .add_object_store(ObjectStore::new("groups"))
        .add_object_store(ObjectStore::new("scores"))
        .build()
        .await?;

    Ok(rexie)
}

//
// NOTE
// Transactions cannot be held across an await point
// (because it returns the control to the browser which then commits the transaction)
// so all database operations must be done in a single call.
//

pub async fn load_groups_all(geng: &Geng, rexie: &Rexie) -> Result<Vec<LocalGroup>> {
    let transaction = rexie.transaction(&["groups"], TransactionMode::ReadOnly)?;
    let groups = transaction.store("groups")?;
    let raw_items = groups.get_all(None, None).await?;
    // transaction.done().await?;

    let mut items = Vec::with_capacity(raw_items.len());
    for item in raw_items {
        let process_item = async |item| -> Result<LocalGroup> {
            let item: GroupItem = serde_wasm_bindgen::from_value(item)?;
            let path = super::all_groups_path().join(item.id);

            let data = BASE64_STANDARD.decode(&item.data)?;
            let meta_bytes = BASE64_STANDARD.decode(&item.meta)?;
            let meta_str = String::from_utf8(meta_bytes)?;
            let (group, meta) = decode_group(&data, &meta_str)?;

            let music = match &item.music {
                None => None,
                Some(music) => {
                    let data = BASE64_STANDARD.decode(music)?;
                    let music = geng.audio().decode(data.clone()).await?;
                    Some(Rc::new(LocalMusic::new(
                        meta.music.clone(),
                        music,
                        data.into(),
                    )))
                }
            };

            Ok(LocalGroup {
                path,
                loaded_from_assets: true,
                meta,
                music,
                data: group,
            })
        };

        match process_item(item).await {
            Ok(group) => items.push(group),
            Err(err) => log::error!("failed to load level item: {:?}", err),
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

    let data = cbor4ii::serde::to_vec(Vec::new(), &group.local.data)?;
    let data = BASE64_STANDARD.encode(&data);

    let music = music.map(|music| BASE64_STANDARD.encode(&music));

    let meta = ron::ser::to_string(&group.local.meta)?;
    let meta = BASE64_STANDARD.encode(&meta);

    let item = GroupItem {
        id: id.to_string(),
        data,
        music,
        meta,
    };

    let serializer = Serializer::json_compatible();
    let item = item.serialize(&serializer)?;
    let id = id.serialize(&serializer)?;

    let transaction = rexie.transaction(&["groups"], TransactionMode::ReadWrite)?;
    let store = transaction.store("groups")?;
    store.put(&item, Some(&id)).await?;
    // transaction.commit().await?;

    Ok(())
}

pub async fn remove_group(rexie: &Rexie, id: &str) -> Result<()> {
    log::debug!("Deleting group {:?} from browser storage", id);

    let serializer = Serializer::json_compatible();
    let id = id.serialize(&serializer)?;

    let transaction = rexie.transaction(&["groups"], TransactionMode::ReadWrite)?;
    let store = transaction.store("groups")?;
    store.delete(id).await?;
    // transaction.commit().await?;

    Ok(())
}

pub async fn load_local_highscores(rexie: &Rexie) -> Result<HashMap<String, SavedScore>> {
    log::debug!("Loading all local highscores from browser storage");

    let transaction = rexie.transaction(&["scores"], TransactionMode::ReadOnly)?;
    let store = transaction.store("scores")?;
    let all_scores = store.get_all(None, None).await?;
    // transaction.done().await?;

    let mut result = HashMap::new();
    for scores in all_scores {
        let process = || -> Result<()> {
            let item: ScoresItem = serde_wasm_bindgen::from_value(scores)?;
            if let Some(score) = item.scores.into_iter().max_by_key(|score| score.score) {
                result.insert(item.level_hash, score);
            }
            Ok(())
        };
        if let Err(err) = process() {
            log::error!("score file error: {err}");
        }
    }

    Ok(result)
}

pub async fn load_local_scores(rexie: &Rexie, level_hash: &str) -> Result<Vec<SavedScore>> {
    log::debug!(
        "Loading local scores for level {:?} from browser storage",
        level_hash
    );

    let serializer = Serializer::json_compatible();
    let level_hash = level_hash.serialize(&serializer)?;

    let transaction = rexie.transaction(&["scores"], TransactionMode::ReadOnly)?;
    let store = transaction.store("scores")?;
    let Some(scores) = store.get(level_hash).await? else {
        return Ok(vec![]);
    };
    // transaction.done().await?;
    let scores: ScoresItem = serde_wasm_bindgen::from_value(scores)?;

    Ok(scores.scores)
}

pub async fn save_local_scores(
    rexie: &Rexie,
    level_hash: &str,
    scores: &[SavedScore],
) -> Result<()> {
    log::debug!(
        "Saving local scores for level {:?} into browser storage",
        level_hash
    );

    let serializer = Serializer::json_compatible();
    let scores = ScoresItem {
        level_hash: level_hash.to_string(),
        scores: scores.to_vec(),
    };
    let scores = scores.serialize(&serializer)?;
    let level_hash = level_hash.serialize(&serializer)?;

    let transaction = rexie.transaction(&["scores"], TransactionMode::ReadWrite)?;
    let store = transaction.store("scores")?;
    store.put(&scores, Some(&level_hash)).await?;
    // transaction.commit().await?;

    Ok(())
}
