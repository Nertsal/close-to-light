use crate::{
    prelude::{HealthConfig, LevelModifiers},
    LeaderboardSecrets,
};

use geng::prelude::*;
use nertboard_client::{Player, ScoreEntry};

pub struct Leaderboard {
    pub my_position: Option<usize>,
    pub top10: Vec<ScoreEntry>,
}

/// Meta information saved together with the score.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ScoreMeta {
    pub version: u32,
    pub group: String,
    pub level: String,
    pub mods: LevelModifiers,
    pub health: HealthConfig,
}

impl Leaderboard {
    pub async fn submit(
        name: String,
        score: Option<i32>,
        meta: &ScoreMeta,
        secrets: LeaderboardSecrets,
    ) -> Self {
        let name = name.as_str();
        log::debug!("Querying the leaderboard");
        log::debug!("Meta info:\n{:#?}", meta);

        let leaderboard = nertboard_client::Nertboard::new(secrets.url, Some(secrets.key)).unwrap();

        let player = if let Some(mut player) = preferences::load::<Player>("player") {
            log::debug!("Leaderboard: returning player");
            if player.name == name {
                // leaderboard.as_player(player.clone());
                player
            } else {
                log::debug!("Leaderboard: name has changed");
                player.name = name.to_owned();
                preferences::save("player", &player);
                player.clone()
            }
        } else {
            log::debug!("Leaderboard: new player");
            let player = leaderboard.create_player(name).await.unwrap();
            preferences::save("player", &player);
            player.clone()
        };

        let meta_str = serde_json::to_string(meta).unwrap(); // TODO: more compact?
        if let Some(score) = score {
            leaderboard
                .submit_score(
                    &player,
                    &ScoreEntry {
                        player: player.name.clone(),
                        score,
                        extra_info: Some(meta_str),
                    },
                )
                .await
                .unwrap();
        }

        let mut scores = leaderboard.fetch_scores().await.unwrap();
        scores.retain(|entry| {
            !entry.player.is_empty()
                && entry.extra_info.as_ref().map_or(false, |info| {
                    serde_json::from_str::<ScoreMeta>(info)
                        .map_or(false, |entry_meta| entry_meta == *meta)
                })
        });
        scores.sort_by_key(|entry| -entry.score);

        {
            // Only leave unique names
            let mut i = 0;
            let mut names_seen = HashSet::new();
            while i < scores.len() {
                if !names_seen.contains(&scores[i].player) {
                    names_seen.insert(scores[i].player.clone());
                    i += 1;
                } else if Some(scores[i].score) == score {
                    i += 1;
                } else {
                    scores.remove(i);
                }
            }
        }

        let my_pos = score.map(|score| scores.iter().position(|this| this.score == score).unwrap());

        {
            // Only leave unique names
            let mut i = 0;
            let mut names_seen = HashSet::new();
            while i < scores.len() {
                if !names_seen.contains(&scores[i].player) {
                    names_seen.insert(scores[i].player.clone());
                    i += 1;
                } else {
                    scores.remove(i);
                }
            }
        }
        scores.truncate(10);

        Self {
            my_position: my_pos,
            top10: scores,
        }
    }
}
