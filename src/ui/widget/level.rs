use super::*;

use crate::{menu::GroupEntry, prelude::LevelConfig, ui::layout};

// use geng_utils::bounded::Bounded;

pub struct PresetWidget {
    pub button: ButtonWidget,
    pub preset: LevelConfig,
    pub selected: bool,
}

impl PresetWidget {
    pub fn new(name: impl Into<String>, preset: LevelConfig) -> Self {
        Self {
            button: ButtonWidget::new(name),
            preset,
            selected: false,
        }
    }
}

impl Widget for PresetWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.button.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.button.walk_states_mut(f);
    }
}

pub struct PlayLevelWidget {
    pub state: WidgetState,
    pub level_normal: ButtonWidget,
    pub credits_normal: TextWidget,
    pub level_hard: ButtonWidget,
    pub credits_hard: TextWidget,

    /// What we are currently configuring.
    pub config_title: TextWidget,
    pub presets: Vec<PresetWidget>,
    pub level_config: LevelConfig,
}

impl PlayLevelWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            level_normal: ButtonWidget::new("Normal"),
            credits_normal: TextWidget::new("<normal credits>"),
            level_hard: ButtonWidget::new("Hard"),
            credits_hard: TextWidget::new("<hard credits>"),

            config_title: TextWidget::new("Presets"),
            presets: [
                ("Easy", LevelConfig::preset_easy()),
                ("Normal", LevelConfig::preset_normal()),
                ("Hard", LevelConfig::preset_hard()),
            ]
            .into_iter()
            .map(|(name, preset)| PresetWidget::new(name, preset))
            .collect(),
            level_config: LevelConfig::default(),
        }
    }

    pub fn set_group(&mut self, group: &GroupEntry) {
        self.credits_normal.text = format!("by {}", group.meta.normal.author);
        self.credits_hard.text = format!("by {}", group.meta.hard.author);
    }
}

impl Widget for PlayLevelWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);

        let main = position.extend_uniform(-context.font_size * 0.5);

        // Levels
        let (levels_pos, main) = layout::cut_top_down(main, context.font_size * 2.5);
        let levels = [
            (&mut self.level_normal, &mut self.credits_normal),
            (&mut self.level_hard, &mut self.credits_hard),
        ];
        for (pos, (button, credits)) in layout::split_columns(levels_pos, levels.len())
            .into_iter()
            .zip(levels)
        {
            let pos = layout::fit_aabb_height(vec2(4.0, 3.0), pos, 0.5);
            let (button_pos, credits_pos) = layout::cut_top_down(pos, context.font_size * 1.2);
            button.update(button_pos, context);
            credits.update(credits_pos, &context.scale_font(0.75));
        }

        let main = main.extend_up(-context.font_size * 1.0);
        let (title, main) = layout::cut_top_down(main, context.font_size * 1.5);
        self.config_title.update(title, context);

        // Presets
        let mut selected = None;
        for (pos, (i, target)) in layout::split_columns(main, self.presets.len())
            .into_iter()
            .zip(self.presets.iter_mut().enumerate())
        {
            let pos = pos.extend_uniform(-context.font_size * 0.2);
            let pos = layout::fit_aabb(vec2(4.0, 2.0), pos, vec2(0.5, 1.0));
            target.update(pos, context);
            if target.button.text.state.clicked {
                selected = Some(i);
                self.level_config = target.preset.clone();
            }
        }
        if let Some(selected) = selected {
            for (i, preset) in self.presets.iter_mut().enumerate() {
                preset.selected = i == selected;
            }
        }
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.level_normal.walk_states_mut(f);
        self.credits_normal.walk_states_mut(f);
        self.level_hard.walk_states_mut(f);
        self.credits_hard.walk_states_mut(f);
    }
}
