use super::*;

use crate::{
    prelude::{LevelMeta, MusicMeta},
    ui::layout::AreaOps,
};

pub struct ExploreWidget {
    pub state: WidgetState,
    pub window: UiWindow<()>,

    pub tabs: WidgetState,
    pub tab_music: TextWidget,
    pub tab_levels: TextWidget,
    pub separator: WidgetState,

    pub music: ExploreMusicWidget,
    pub levels: ExploreLevelsWidget,
}

pub struct ExploreLevelsWidget {
    pub state: WidgetState,
    pub items: Vec<LevelItemWidget>,
}

pub struct ExploreMusicWidget {
    pub state: WidgetState,
    pub items: Vec<MusicItemWidget>,
}

pub struct LevelItemWidget {
    pub state: WidgetState,
    pub meta: LevelMeta,
}

pub struct MusicItemWidget {
    pub state: WidgetState,
    pub meta: MusicMeta,
}

impl ExploreWidget {
    pub fn new() -> Self {
        let mut w = Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),

            tabs: WidgetState::new(),
            tab_music: TextWidget::new("Music"),
            tab_levels: TextWidget::new("Levels"),
            separator: WidgetState::new(),

            music: ExploreMusicWidget::new(),
            levels: ExploreLevelsWidget::new(),
        };
        w.music.hide();
        w
    }
}

impl Widget for ExploreWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.window.update(context.delta_time);

        let mut main = position;
        main.cut_top(context.layout_size * 1.0);
        let bar = main.cut_top(context.font_size * 1.2);

        let bar = bar.extend_symmetric(-vec2(1.0, 0.0) * context.layout_size);

        // TODO: extract to a function or smth
        {
            let tab_refs = [&mut self.tab_music, &mut self.tab_levels];

            let tab = Aabb2::point(bar.bottom_left())
                .extend_positive(vec2(4.0 * context.font_size, bar.height()));
            let tabs = tab.stack(
                vec2(tab.width() + 2.0 * context.layout_size, 0.0),
                tab_refs.len(),
            );

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

            for (tab, pos) in tab_refs.into_iter().zip(tabs) {
                tab.update(pos, context);
            }

            let separator = bar.extend_up(context.font_size * 0.2 - bar.height());
            self.separator.update(separator, context);
        }

        if self.tab_music.state.clicked {
            self.music.show();
            self.levels.hide();
        } else if self.tab_levels.state.clicked {
            self.music.hide();
            self.levels.show();
        }

        let main = main.extend_uniform(-context.font_size * 0.5);
        // self.mods.update(main, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);

        self.tabs.walk_states_mut(f);
        self.tab_music.walk_states_mut(f);
        self.tab_levels.walk_states_mut(f);

        self.music.walk_states_mut(f);
        self.levels.walk_states_mut(f);
    }
}

impl ExploreLevelsWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            items: Vec::new(),
        }
    }
}

impl Widget for ExploreLevelsWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        for w in &mut self.items {
            w.walk_states_mut(f);
        }
    }
}

impl ExploreMusicWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            items: Vec::new(),
        }
    }
}

impl Widget for ExploreMusicWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        for w in &mut self.items {
            w.walk_states_mut(f);
        }
    }
}

impl Widget for LevelItemWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}

impl Widget for MusicItemWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
