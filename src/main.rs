mod assets;
mod editor;
mod game;
mod leaderboard;
mod menu;
mod model;
mod prelude;
mod render;
mod ui;
mod util;

use geng::prelude::*;

#[derive(clap::Parser)]
struct Opts {
    /// Open a level in the editor.
    #[clap(long)]
    edit: bool,
    #[clap(flatten)]
    geng: geng::CliArgs,
}

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "toml")]
struct Secrets {
    leaderboard: LeaderboardSecrets,
}

#[derive(Deserialize, Clone)]
pub struct LeaderboardSecrets {
    id: String,
    key: String,
}

fn main() {
    logger::init();
    geng::setup_panic_handler();

    let opts: Opts = clap::Parser::parse();

    let mut options = geng::ContextOptions::default();
    options.window.title = "Geng Game".to_string();
    options.with_cli(&opts.geng);

    Geng::run_with(&options, move |geng| async move {
        let manager = geng.asset_manager();
        let assets_path = run_dir().join("assets");

        // let level_path = assets_path.join("levels").join("level.json");
        // let level: migrate::old::Level = geng::asset::Load::load(manager, &level_path, &())
        //     .await
        //     .expect("failed to load level");
        // let level = migrate::migrate(level);
        // (|| -> anyhow::Result<()> {
        //     // TODO: switch back to ron
        //     // https://github.com/geng-engine/geng/issues/71
        //     let level = serde_json::to_string_pretty(&level)?;
        //     let mut writer = std::io::BufWriter::new(std::fs::File::create(&level_path)?);
        //     write!(writer, "{}", level)?;
        //     Ok(())
        // })()
        // .unwrap();
        // return;

        let assets = assets::Assets::load(manager).await.unwrap();
        let assets = Rc::new(assets);
        let config: model::Config =
            geng::asset::Load::load(manager, &assets_path.join("config.ron"), &())
                .await
                .expect("failed to load config");

        if opts.edit {
            // Editor
            let level_path = assets_path.join("levels").join("level.json");
            let level: model::Level = geng::asset::Load::load(manager, &level_path, &())
                .await
                .expect("failed to load level");

            let editor_config: editor::EditorConfig =
                geng::asset::Load::load(manager, &assets_path.join("editor.ron"), &())
                    .await
                    .expect("failed to load editor config");
            let state = editor::EditorState::new(
                geng.clone(),
                assets,
                editor_config,
                config,
                level,
                level_path,
            );
            geng.run_state(state).await;
        } else {
            // Main menu
            let state = menu::MainMenu::new(&geng, &assets, config);
            geng.run_state(state).await;
        }
    });
}

mod migrate {
    use crate::prelude::*;

    pub fn migrate(old_level: old::Level) -> Level {
        Level {
            config: old_level.config,
            events: old_level
                .events
                .into_iter()
                .map(|event| {
                    let e = match event.event {
                        old::Event::Light(light) => {
                            let mut key_frames = light.light.movement.key_frames;

                            let fade = key_frames.pop_front().unwrap();
                            assert_eq!(fade.transform.translation, vec2::ZERO);
                            assert_eq!(fade.transform.rotation, Angle::ZERO);
                            assert_eq!(fade.transform.scale, Coord::ZERO);
                            let fade = key_frames.pop_front().unwrap();
                            assert_eq!(fade.transform.translation, vec2::ZERO);
                            assert_eq!(fade.transform.rotation, Angle::ZERO);
                            let fade_in = fade.lerp_time;
                            let initial = Transform {
                                translation: light.light.position,
                                rotation: Angle::from_degrees(light.light.rotation),
                                scale: fade.transform.scale,
                            };

                            let fade = key_frames.pop_back().unwrap();
                            assert_eq!(fade.transform.translation, vec2::ZERO);
                            assert_eq!(fade.transform.rotation, Angle::ZERO);
                            assert_eq!(fade.transform.scale, Coord::ZERO);
                            let fade_out = fade.lerp_time;

                            let light = LightEvent {
                                light: LightSerde {
                                    danger: light.light.danger,
                                    shape: light.light.shape,
                                    movement: Movement {
                                        fade_in,
                                        fade_out,
                                        initial,
                                        key_frames,
                                    },
                                },
                                telegraph: light.telegraph,
                            };
                            Event::Light(light)
                        }
                        old::Event::Theme(theme) => Event::Theme(theme),
                    };
                    TimedEvent {
                        beat: event.beat,
                        event: e,
                    }
                })
                .collect(),
        }
    }

    pub mod old {
        use super::*;

        #[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize, PartialEq)]
        #[load(serde = "json")]
        pub struct Level {
            pub config: LevelConfig,
            // /// Whether to start rng after the predefined level is finished.
            // #[serde(default)]
            // pub rng_end: bool,
            pub events: Vec<TimedEvent>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub struct TimedEvent {
            /// The beat on which the event should happen.
            pub beat: Time,
            pub event: Event,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub enum Event {
            Light(LightEvent),
            Theme(LevelTheme),
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        pub struct LightSerde {
            pub position: vec2<Coord>,
            /// Whether the light is dangerous.
            #[serde(default)]
            pub danger: bool,
            /// Rotation (in degrees).
            #[serde(default = "LightSerde::default_rotation")]
            pub rotation: Coord,
            pub shape: Shape,
            /// Movement with timings in beats.
            #[serde(default)]
            pub movement: Movement,
            // /// Lifetime (in beats).
            // pub lifetime: Time,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        pub struct LightEvent {
            pub light: LightSerde,
            pub telegraph: Telegraph,
        }

        impl LightSerde {
            fn default_rotation() -> Coord {
                Coord::ZERO
            }

            pub fn instantiate(self, event_id: Option<usize>) -> Light {
                let collider = Collider {
                    position: self.position,
                    rotation: Angle::from_degrees(self.rotation),
                    shape: self.shape,
                };
                Light {
                    base_collider: collider.clone(),
                    collider,
                    // movement: self.movement,
                    lifetime: Time::ZERO,
                    // lifetime: Lifetime::new_max(self.lifetime * beat_time),
                    danger: self.danger,
                    event_id,
                }
            }
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        pub struct Movement {
            pub key_frames: VecDeque<MoveFrame>,
        }

        impl Default for Movement {
            fn default() -> Self {
                Self {
                    key_frames: default(),
                }
            }
        }
    }
}
