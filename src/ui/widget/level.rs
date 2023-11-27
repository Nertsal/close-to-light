use super::*;

use crate::{
    menu::GroupEntry,
    prelude::{Assets, HealthConfig, LevelConfig, Modifier, Theme},
    ui::layout,
};

// use geng_utils::bounded::Bounded;

pub struct PresetWidget<T> {
    pub button: ButtonWidget,
    pub preset: T,
    pub selected: bool,
}

impl<T> PresetWidget<T> {
    pub fn new(name: impl Into<String>, preset: T) -> Self {
        Self {
            button: ButtonWidget::new(name),
            preset,
            selected: false,
        }
    }
}

impl<T> Widget for PresetWidget<T> {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.button.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.button.walk_states_mut(f);
    }
}

pub struct ConfigWidget {
    pub state: WidgetState,
    pub configuring: Configuring,
}

pub enum Configuring {
    Palette {
        presets: Vec<PresetWidget<Theme>>,
    },
    Health {
        presets: Vec<PresetWidget<HealthConfig>>,
    },
    Modifiers {
        presets: Vec<PresetWidget<Modifier>>,
    },
}

impl ConfigWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext, config: &mut LevelConfig) {
        self.state.update(position, context);
        match &mut self.configuring {
            Configuring::Palette { presets } => {
                for (pos, (_i, target)) in layout::split_columns(position, presets.len())
                    .into_iter()
                    .zip(presets.iter_mut().enumerate())
                {
                    let pos = pos.extend_uniform(-context.font_size * 0.2);
                    let pos = layout::fit_aabb(vec2(4.0, 2.0), pos, vec2(0.5, 1.0));
                    target.update(pos, context);
                    if target.button.text.state.clicked {
                        config.theme = target.preset;
                    }
                    target.selected = target.preset == config.theme;
                }
            }
            Configuring::Health { presets } => {
                for (pos, (_i, target)) in layout::split_columns(position, presets.len())
                    .into_iter()
                    .zip(presets.iter_mut().enumerate())
                {
                    let pos = pos.extend_uniform(-context.font_size * 0.2);
                    let pos = layout::fit_aabb(vec2(4.0, 2.0), pos, vec2(0.5, 1.0));
                    target.update(pos, context);
                    if target.button.text.state.clicked {
                        config.health = target.preset.clone();
                    }
                    target.selected = target.preset == config.health;
                }
            }
            Configuring::Modifiers { presets } => {
                for (pos, (_i, target)) in layout::split_columns(position, presets.len())
                    .into_iter()
                    .zip(presets.iter_mut().enumerate())
                {
                    let pos = pos.extend_uniform(-context.font_size * 0.2);
                    let pos = layout::fit_aabb(vec2(4.0, 2.0), pos, vec2(0.5, 1.0));
                    target.update(pos, context);
                    let mods = &mut config.modifiers;
                    let value = match target.preset {
                        Modifier::NoFail => &mut mods.nofail,
                        Modifier::Sudden => &mut mods.sudden,
                        Modifier::Hidden => &mut mods.hidden,
                    };
                    if target.button.text.state.clicked {
                        *value = !*value;
                    }
                    target.selected = *value;
                }
            }
        }
    }
}

pub struct LevelWidget {
    pub play: ButtonWidget,
    pub credits: TextWidget,
}

pub struct LevelGroupWidget {
    pub state: WidgetState,
    pub levels: Vec<LevelWidget>,

    /// What we are currently configuring.
    /// Float for animating.
    pub config_current: f32,
    pub config_target: f32,
    pub config_titles: Vec<TextWidget>,
    pub configs_area: Aabb2<f32>,
    pub configs: Vec<ConfigWidget>,
    pub prev_config: ButtonWidget,
    pub next_config: ButtonWidget,
    pub level_config: LevelConfig,
}

impl LevelWidget {
    pub fn new() -> Self {
        Self {
            play: ButtonWidget::new("<level name>"),
            credits: TextWidget::new("by <author>"),
        }
    }
}

impl Widget for LevelWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        let pos = layout::fit_aabb_height(vec2(4.0, 2.0), position, 0.5);
        let (button_pos, credits_pos) = layout::cut_top_down(pos, context.font_size * 1.2);
        self.play.update(button_pos, context);
        self.credits.update(credits_pos, &context.scale_font(0.75));
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.play.walk_states_mut(f);
        self.credits.walk_states_mut(f);
    }
}

impl LevelGroupWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            state: WidgetState::new(),
            levels: vec![],

            config_current: 0.0,
            config_target: 0.0,
            config_titles: ["Palette", "Difficulty", "Modifiers"]
                .into_iter()
                .map(TextWidget::new)
                .collect(),
            prev_config: ButtonWidget::new_textured("", &assets.sprites.button_prev),
            next_config: ButtonWidget::new_textured("", &assets.sprites.button_next),
            configs_area: Aabb2::ZERO,
            configs: [
                Configuring::Palette {
                    presets: [("Classic", Theme::default()), ("Test", Theme::test())]
                        .into_iter()
                        .map(|(name, preset)| PresetWidget::new(name, preset))
                        .collect(),
                },
                Configuring::Health {
                    presets: [
                        ("Easy", HealthConfig::preset_easy()),
                        ("Normal", HealthConfig::preset_normal()),
                        ("Hard", HealthConfig::preset_hard()),
                    ]
                    .into_iter()
                    .map(|(name, preset)| PresetWidget::new(name, preset))
                    .collect(),
                },
                Configuring::Modifiers {
                    presets: enum_iterator::all::<Modifier>()
                        .map(|preset| PresetWidget::new(format!("{}", preset), preset))
                        .collect(),
                },
            ]
            .into_iter()
            .map(|configuring| ConfigWidget {
                state: WidgetState::new(),
                configuring,
            })
            .collect(),
            level_config: LevelConfig::default(),
        }
    }

    pub fn set_group(&mut self, group: &GroupEntry) {
        self.levels.truncate(group.levels.len());
        for _ in 0..group.levels.len() - self.levels.len() {
            self.levels.push(LevelWidget::new());
        }

        for (level, (_, meta)) in self.levels.iter_mut().zip(&group.levels) {
            level.play.text.text = meta.name.to_string();
            level.credits.text = format!("by {}", meta.author);
        }
    }

    // TODO: move to Widget
    pub fn update_time(&mut self, delta_time: f32) {
        let lerp_time = 0.2;
        self.config_current += (self.config_target - self.config_current) / lerp_time * delta_time;
    }
}

impl Widget for LevelGroupWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);

        let main = position.extend_uniform(-context.font_size * 0.5);

        // Levels
        let (levels_pos, main) = layout::cut_top_down(main, context.font_size * 2.5);
        let levels = &mut self.levels;
        for (pos, level) in layout::split_columns(levels_pos, levels.len())
            .into_iter()
            .zip(levels)
        {
            level.update(pos, context);
        }

        // Config
        let main = main.extend_up(-context.font_size * 1.0);
        let (title, main) = layout::cut_top_down(main, context.font_size * 1.5);
        {
            self.configs_area = main;
            let title = Aabb2::point(title.center())
                .extend_symmetric(vec2(context.font_size * 5.0, title.height()) / 2.0);
            for (i, (config_title, config)) in self
                .config_titles
                .iter_mut()
                .zip(&mut self.configs)
                .enumerate()
            {
                let offset = i as f32 - self.config_current;
                if offset.abs() > 1.1 {
                    config_title.hide();
                } else {
                    config_title.show();
                    let offset = offset * title.width();
                    let title = title.translate(vec2(offset, 0.0));
                    config_title.update(title, context);
                }

                if offset.abs() > 0.5 {
                    config.state.hide();
                } else {
                    config.state.show();
                    let offset = offset * main.width();
                    let main = main.translate(vec2(offset, 0.0));
                    config.update(main, context, &mut self.level_config);
                }
            }

            let title = title.extend_symmetric(-vec2(0.0, context.font_size * 0.4) / 2.0);
            let prev = Aabb2::point(title.bottom_left())
                .extend_left(title.height())
                .extend_up(title.height());
            let next =
                Aabb2::point(title.bottom_right()).extend_positive(vec2::splat(title.height()));
            self.prev_config.update(prev, context);
            self.next_config.update(next, context);

            if self.prev_config.text.state.clicked {
                self.config_target -= 1.0;
            } else if self.next_config.text.state.clicked {
                self.config_target += 1.0;
            }

            // Wrap
            let max = self.config_titles.len() as f32 - 1.0;
            let wrap = |value: &mut f32| {
                if *value < -0.5 {
                    *value += max + 1.0;
                } else if *value > max + 0.5 {
                    *value -= max + 1.0;
                }
            };
            wrap(&mut self.config_target);
            wrap(&mut self.config_current);
        }
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        for level in &mut self.levels {
            level.walk_states_mut(f);
        }
        self.prev_config.walk_states_mut(f);
        self.next_config.walk_states_mut(f);
    }
}
