use crate::LeaderboardSecrets;

use geng::prelude::*;

pub struct Leaderboard {
    pub my_position: Option<usize>,
    pub top10: Vec<jornet::Score>,
}

impl Leaderboard {
    pub async fn submit(name: String, score: Option<f32>, secrets: LeaderboardSecrets) -> Self {
        let name = name.as_str();
        log::info!("Querying the leaderboard");

        let mut leaderboard = jornet::Leaderboard::with_host_and_leaderboard(
            None,
            secrets.id.parse().unwrap(),
            secrets.key.parse().unwrap(),
        );

        let _player = if let Some(player) = preferences::load::<jornet::Player>("player") {
            log::info!("Returning player");
            if player.name == name {
                leaderboard.as_player(player.clone());
                player
            } else {
                log::info!("Name has changed");
                let player = leaderboard.create_player(Some(name)).await.unwrap();
                preferences::save("player", player);
                player.clone()
            }
        } else {
            log::info!("New player");
            let player = leaderboard.create_player(Some(name)).await.unwrap();
            preferences::save("player", &player);
            player.clone()
        };

        // let meta = serde_json::to_string(&diff).unwrap();
        let meta = "v0".to_string();
        if let Some(score) = score {
            leaderboard
                .send_score_with_meta(score, &meta)
                .await
                .unwrap();
        }

        let mut scores = leaderboard.get_leaderboard().await.unwrap();
        scores.retain(|score| {
            !score.player.is_empty() && score.meta.as_deref() == Some(meta.as_str())
        });
        scores.sort_by_key(|score| -r32(score.score));

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
                    dbg!(&scores[i]);
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
