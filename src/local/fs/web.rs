use super::*;

use base64::prelude::*;
use rexie::{Result, *};
use serde_wasm_bindgen::Serializer;

#[derive(Serialize, Deserialize)]
struct GroupItem {
    meta: String,
    data: String,
    music: Option<String>,
}

pub async fn build_database() -> Result<Rexie> {
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
        let key: String = serde_wasm_bindgen::from_value(key).unwrap();
        let path = super::all_groups_path().join(key);
        let item: GroupItem = serde_wasm_bindgen::from_value(item).unwrap();

        let data = BASE64_STANDARD.decode(&item.data).unwrap(); // TODO dont panic
        let group: LevelSet = decode_group(&data).unwrap();

        let music_bytes = BASE64_STANDARD.decode(&item.meta).unwrap(); // TODO dont panic
        let meta: GroupMeta = ron::de::from_bytes(&music_bytes).unwrap(); // TODO dont panic

        let music = match &item.music {
            None => None,
            Some(music) => {
                let data = BASE64_STANDARD.decode(music).unwrap(); // TODO dont panic
                let music = geng.audio().decode(data).await.unwrap();
                Some(Rc::new(LocalMusic::new(
                    meta.music.clone().unwrap_or_default(),
                    music,
                    music_bytes,
                )))
            }
        };

        items.push(LocalGroup {
            path,
            meta,
            music,
            data: group,
        });
    }

    Ok(items)
}

pub async fn save_group(
    rexie: &Rexie,
    group: &CachedGroup,
    music: Option<&Vec<u8>>,
    id: &str,
) -> Result<()> {
    log::debug!("Storing group {:?} into browser storage", id);

    let transaction = rexie.transaction(&["groups"], TransactionMode::ReadWrite)?;

    let store = transaction.store("groups")?;

    let data = cbor4ii::serde::to_vec(Vec::new(), &group.local.data).unwrap(); // TODO: dont panic
    let data = BASE64_STANDARD.encode(&data);

    let music = music.map(|music| BASE64_STANDARD.encode(&music));

    let meta = ron::ser::to_string(&group.local.meta).unwrap(); // TODO: dont panic
    let meta = BASE64_STANDARD.encode(&meta);

    let item = GroupItem { data, music, meta };

    let serializer = Serializer::json_compatible();
    let item = item.serialize(&serializer).unwrap();
    let id = id.serialize(&serializer).unwrap();

    store.put(&item, Some(&id)).await?;

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
