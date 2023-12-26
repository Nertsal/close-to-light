use crate::LeaderboardSecrets;

use geng::prelude::*;
use nertboard_client::ScoreEntry;

pub struct Leaderboard {
    pub my_position: Option<usize>,
    pub top10: Vec<ScoreEntry>,
}

impl Leaderboard {
    pub async fn submit(name: String, score: Option<i32>, secrets: LeaderboardSecrets) -> Self {
        let name = name.as_str();
        log::info!("Querying the leaderboard");

        let leaderboard = nertboard_client::Nertboard::new(secrets.url, Some(secrets.key));

        let player = if let Some(player) = preferences::load::<String>("player") {
            log::info!("Leaderboard: returning player");
            if player == name {
                // leaderboard.as_player(player.clone());
                player
            } else {
                log::info!("Leaderboard: name has changed");
                // let player = leaderboard.create_player(Some(name)).await.unwrap();
                preferences::save("player", &player);
                player.clone()
            }
        } else {
            log::info!("Leaderboard: new player");
            // let player = leaderboard.create_player(Some(name)).await.unwrap();
            let player = name.to_owned();
            preferences::save("player", &player);
            player.clone()
        };

        // let meta = serde_json::to_string(&diff).unwrap();
        let meta = "v1".to_string();
        if let Some(score) = score {
            leaderboard
                .submit_score(&ScoreEntry {
                    player,
                    score,
                    extra_info: Some(meta.clone()),
                })
                .await
                .unwrap();
        }

        let mut scores = leaderboard.fetch_scores().await.unwrap();
        scores.retain(|entry| {
            !entry.player.is_empty() && entry.extra_info.as_deref() == Some(meta.as_str())
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
