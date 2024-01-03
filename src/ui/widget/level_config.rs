use crate::{
    prelude::{Assets, HealthConfig, LevelConfig, LevelModifiers, Modifier},
    ui::layout,
};

use super::*;

pub struct LevelConfigWidget {
    pub state: WidgetState,
    pub close: ButtonWidget,
    pub tabs: WidgetState,
    pub tab_difficulty: TextWidget,
    pub tab_mods: TextWidget,
    pub separator: WidgetState,
    pub difficulty: LevelDifficultyWidget,
    pub mods: LevelModsWidget,
}

impl LevelConfigWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        let mut w = Self {
            state: WidgetState::new(),
            close: ButtonWidget::new_textured("", &assets.sprites.button_close),
            tabs: WidgetState::new(),
            tab_difficulty: TextWidget::new("Difficulty"),
            tab_mods: TextWidget::new("Modifiers"),
            separator: WidgetState::new(),
            difficulty: LevelDifficultyWidget::new(),
            mods: LevelModsWidget::new(),
        };
        w.difficulty.hide();
        w.mods.hide();
        w
    }

    pub fn set_config(&mut self, config: &LevelConfig) {
        self.difficulty.selected = config.health.clone();
        self.mods.selected = config.modifiers.clone();
    }

    pub fn update_config(&self, config: &mut LevelConfig) {
        config.health = self.difficulty.selected.clone();
        config.modifiers = self.mods.selected.clone();
    }
}

impl Widget for LevelConfigWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        let main = position;

        let close = layout::align_aabb(
            vec2::splat(1.0) * context.font_size,
            main.extend_uniform(-0.5 * context.layout_size),
            vec2(1.0, 1.0),
        );
        self.close.update(close, context);

        let main = main.extend_up(-context.layout_size * 1.0);
        let (bar, main) = layout::cut_top_down(main, context.font_size * 1.2);

        let bar = bar.extend_symmetric(-vec2(1.0, 0.0) * context.layout_size);

        let tab = Aabb2::point(bar.bottom_left())
            .extend_positive(vec2(4.0 * context.font_size, bar.height()));
        let tabs = layout::stack(tab, vec2(tab.width() + 2.0 * context.layout_size, 0.0), 2);

        let mut all_tabs = tab;
        if let Some(tab) = tabs.last() {
            all_tabs.max.x = tab.max.x;
        }
        let align = vec2(bar.center().x - all_tabs.center().x, 0.0);
        let all_tabs = all_tabs
            .translate(align)
            .extend_symmetric(vec2(1.0, 0.0) * context.layout_size);
        self.tabs.update(all_tabs, context);

        let tabs: Vec<_> = tabs.into_iter().map(|tab| tab.translate(align)).collect();

        for (tab, pos) in [&mut self.tab_difficulty, &mut self.tab_mods]
            .into_iter()
            .zip(tabs)
        {
            tab.update(pos, context);
        }

        let separator = bar.extend_up(context.font_size * 0.2 - bar.height());
        self.separator.update(separator, context);

        if self.tab_difficulty.state.clicked {
            self.difficulty.show();
            self.mods.hide();
        } else if self.tab_mods.state.clicked {
            self.difficulty.hide();
            self.mods.show();
        }

        let main = main.extend_uniform(-context.font_size * 0.5);
        self.difficulty.update(main, context);
        self.mods.update(main, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.tab_difficulty.walk_states_mut(f);
        self.tab_mods.walk_states_mut(f);
        self.difficulty.walk_states_mut(f);
        self.mods.walk_states_mut(f);
    }
}

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

pub struct LevelDifficultyWidget {
    pub state: WidgetState,
    pub selected: HealthConfig,
    pub presets: Vec<PresetWidget<HealthConfig>>,
}

impl LevelDifficultyWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            selected: HealthConfig::preset_normal(),
            presets: [
                ("Easy", HealthConfig::preset_easy()),
                ("Normal", HealthConfig::preset_normal()),
                ("Hard", HealthConfig::preset_hard()),
            ]
            .into_iter()
            .map(|(name, preset)| PresetWidget::new(name, preset))
            .collect(),
        }
    }
}

impl Widget for LevelDifficultyWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        for (pos, (_i, target)) in layout::split_columns(position, self.presets.len())
            .into_iter()
            .zip(self.presets.iter_mut().enumerate())
        {
            let pos = pos.extend_uniform(-context.font_size * 0.2);
            let pos = layout::fit_aabb(vec2(4.0, 2.0), pos, vec2(0.5, 1.0));
            target.update(pos, context);
            if target.button.text.state.clicked {
                self.selected = target.preset.clone();
            }
            target.selected = target.preset == self.selected;
        }
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        for preset in &mut self.presets {
            preset.walk_states_mut(f);
        }
    }
}

pub struct LevelModsWidget {
    pub state: WidgetState,
    pub selected: LevelModifiers,
    pub mods: Vec<PresetWidget<Modifier>>,
}

impl LevelModsWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            selected: LevelModifiers::default(),
            mods: enum_iterator::all::<Modifier>()
                .map(|preset| PresetWidget::new(format!("{}", preset), preset))
                .collect(),
        }
    }
}

impl Widget for LevelModsWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        for (pos, (_i, target)) in layout::split_columns(position, self.mods.len())
            .into_iter()
            .zip(self.mods.iter_mut().enumerate())
        {
            let pos = pos.extend_uniform(-context.font_size * 0.2);
            let pos = layout::fit_aabb(vec2(4.0, 2.0), pos, vec2(0.5, 1.0));
            target.update(pos, context);
            let mods = &mut self.selected;
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

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        for preset in &mut self.mods {
            preset.walk_states_mut(f);
        }
    }
}
