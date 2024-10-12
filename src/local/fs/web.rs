use super::*;

use base64::prelude::*;
use rexie::{Result, *};
use serde_wasm_bindgen::Serializer;

#[derive(Serialize, Deserialize)]
struct MusicItem {
    info: String,
    data: String,
}

#[derive(Serialize, Deserialize)]
struct GroupItem {
    data: String,
}

pub async fn build_database() -> Result<Rexie> {
    // Create a new database
    let rexie = Rexie::builder("close-to-light")
        .version(1)
        .add_object_store(ObjectStore::new("music"))
        .add_object_store(ObjectStore::new("groups"))
        .build()
        .await?;

    Ok(rexie)
}

pub async fn load_music_all(rexie: &Rexie, geng: &Geng) -> Result<Vec<CachedMusic>> {
    let transaction = rexie.transaction(&["music"], TransactionMode::ReadOnly)?;

    let music = transaction.store("music")?;

    let raw_items = music.get_all(None, None, None, None).await?;
    let mut items = Vec::with_capacity(raw_items.len());
    for (_key, item) in raw_items {
        let item: MusicItem = serde_wasm_bindgen::from_value(item).unwrap();

        let meta: MusicInfo = serde_json::from_str(&item.info).unwrap();

        let data = BASE64_STANDARD.decode(&item.data).unwrap(); // TODO dont panic
        let music = geng.audio().decode(data).await.unwrap();

        items.push(CachedMusic {
            meta,
            music: Rc::new(music),
        });
    }

    Ok(items)
}

pub async fn load_groups_all(rexie: &Rexie) -> Result<Vec<(PathBuf, LevelSet)>> {
    let transaction = rexie.transaction(&["groups"], TransactionMode::ReadOnly)?;

    let groups = transaction.store("groups")?;

    let raw_items = groups.get_all(None, None, None, None).await?;
    let mut items = Vec::with_capacity(raw_items.len());
    for (key, item) in raw_items {
        let key: String = serde_wasm_bindgen::from_value(key).unwrap();
        let path = super::all_groups_path().join(key);
        let item: GroupItem = serde_wasm_bindgen::from_value(item).unwrap();

        let data = BASE64_STANDARD.decode(&item.data).unwrap(); // TODO dont panic

        let group: LevelSet = decode_group(&data).unwrap();

        items.push((path, group));
    }

    Ok(items)
}

pub async fn save_music(rexie: &Rexie, id: Id, data: &[u8], info: &MusicInfo) -> Result<()> {
    log::debug!("Storing music {:?} into browser storage", id);

    let transaction = rexie.transaction(&["music"], TransactionMode::ReadWrite)?;

    let music = transaction.store("music")?;

    let data = BASE64_STANDARD.encode(&data);
    let info = serde_json::to_string(&info).unwrap();

    let item = MusicItem { info, data };

    let serializer = Serializer::json_compatible();
    let item = item.serialize(&serializer).unwrap();
    let id = id.serialize(&serializer).unwrap();

    music.put(&item, Some(&id)).await?;

    transaction.done().await?;

    Ok(())
}

pub async fn save_group(rexie: &Rexie, group: &CachedGroup, id: &str) -> Result<()> {
    log::debug!("Storing group {:?} into browser storage", id);

    let transaction = rexie.transaction(&["groups"], TransactionMode::ReadWrite)?;

    let store = transaction.store("groups")?;

    let data = cbor4ii::serde::to_vec(Vec::new(), &group.data).unwrap();
    let data = data.as_bytes();
    let data = BASE64_STANDARD.encode(data);
    let item = GroupItem { data };

    let serializer = Serializer::json_compatible();
    let item = item.serialize(&serializer).unwrap();
    let id = id.serialize(&serializer).unwrap();

    store.put(&item, Some(&id)).await?;

    transaction.done().await?;

    Ok(())
}

pub async fn remove_music(rexie: &Rexie, id: Id) -> Result<()> {
    log::debug!("Deleting music {:?} from browser storage", id);

    let transaction = rexie.transaction(&["music"], TransactionMode::ReadWrite)?;

    let store = transaction.store("music")?;

    let serializer = Serializer::json_compatible();
    let id = id.serialize(&serializer).unwrap();

    store.delete(&id).await?;

    transaction.done().await?;

    Ok(())
}

pub async fn remove_group(rexie: &Rexie, id: &str) -> Result<()> {
    log::debug!("Deleting group {:?} from browser storage", id);

    let transaction = rexie.transaction(&["groups"], TransactionMode::ReadWrite)?;

    let store = transaction.store("groups")?;

    let serializer = Serializer::json_compatible();
    let id = id.serialize(&serializer).unwrap();

    store.delete(&id).await?;

    transaction.done().await?;

    Ok(())
}
