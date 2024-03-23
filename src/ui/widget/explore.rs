use ctl_client::core::types::{LevelInfo, MusicInfo};

use super::*;

use crate::{
    local::{CacheState, LevelCache},
    prelude::Assets,
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
    pub items_state: WidgetState,
    pub items: Vec<LevelItemWidget>,
}

pub struct ExploreMusicWidget {
    assets: Rc<Assets>,
    pub state: WidgetState,
    pub status: TextWidget,
    pub scroll: f32,
    pub target_scroll: f32,
    pub items_state: WidgetState,
    pub items: Vec<MusicItemWidget>,
}

pub struct LevelItemWidget {
    pub state: WidgetState,
    pub meta: LevelInfo,
}

pub struct MusicItemWidget {
    pub state: WidgetState,
    pub download: IconWidget,
    pub play: IconWidget,
    pub info: MusicInfo,
    pub name: TextWidget,
    pub author: TextWidget,
}

impl ExploreWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        let mut w = Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),

            tabs: WidgetState::new(),
            tab_music: TextWidget::new("Music"),
            tab_levels: TextWidget::new("Levels"),
            separator: WidgetState::new(),

            music: ExploreMusicWidget::new(assets),
            levels: ExploreLevelsWidget::new(),
        };
        w.music.hide();
        w
    }
}

impl StatefulWidget for ExploreWidget {
    type State = LevelCache;

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);
        self.window.update(context.delta_time);
        self.music.load(&state.music_list);

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
            state.fetch_music();
            self.music.show();
            self.levels.hide();
        } else if self.tab_levels.state.clicked {
            self.music.hide();
            self.levels.show();
        }

        let main = main.extend_uniform(-context.font_size * 0.5);
        self.music.update(main, context, state);
        self.levels.update(main, context);
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
            items_state: WidgetState::new(),
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
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            assets: assets.clone(),
            state: WidgetState::new(),
            status: TextWidget::new("Offline"),
            scroll: 0.0,
            target_scroll: 0.0,
            items_state: WidgetState::new(),
            items: Vec::new(),
        }
    }

    fn load(&mut self, music: &CacheState<Vec<MusicInfo>>) {
        self.items.clear();
        self.status.show();
        match music {
            CacheState::Offline => self.status.text = "Offline :(".into(),
            CacheState::Loading => self.status.text = "Loading...".into(),
            CacheState::Loaded(music) => {
                if music.is_empty() {
                    self.status.text = "Empty :(".into();
                } else {
                    self.status.hide();
                    self.items = music
                        .iter()
                        .map(|info| MusicItemWidget {
                            state: WidgetState::new(),
                            download: IconWidget::new(&self.assets.sprites.download),
                            play: IconWidget::new(&self.assets.sprites.button_next),
                            name: TextWidget::new(&info.name),
                            author: TextWidget::new(
                                itertools::Itertools::intersperse(
                                    info.authors.iter().map(|user| user.name.as_str()),
                                    ",",
                                )
                                .collect::<String>(),
                            ),
                            info: info.clone(),
                        })
                        .collect();
                }
            }
        }
    }
}

impl StatefulWidget for ExploreMusicWidget {
    type State = LevelCache;

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);

        let main = position;

        self.items_state.update(main, context);
        self.status.update(main, context);

        let main = main.translate(vec2(0.0, -self.scroll));
        let row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 2.0);
        let rows = row.stack(
            vec2(0.0, -row.height() - context.layout_size * 1.0),
            self.items.len(),
        );
        let height = rows.last().map_or(0.0, |row| main.max.y - row.min.y);
        for (row, position) in self.items.iter_mut().zip(rows) {
            row.update(position, context, state);
        }

        // TODO: extract to a function or smth
        {
            self.target_scroll += context.cursor.scroll;
            let overflow_up = self.target_scroll;
            let max_scroll = (height - main.height()).max(0.0);
            let overflow_down = -max_scroll - self.target_scroll;
            let overflow = if overflow_up > 0.0 {
                overflow_up
            } else if overflow_down > 0.0 {
                -overflow_down
            } else {
                0.0
            };
            self.target_scroll -= overflow * (context.delta_time / 0.2).min(1.0);
            self.scroll += (self.target_scroll - self.scroll) * (context.delta_time / 0.1).min(1.0);
        }
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

impl StatefulWidget for MusicItemWidget {
    type State = LevelCache;

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);

        let mut main = position.extend_uniform(-context.font_size * 0.2);

        let icons = main.cut_left(context.font_size);
        let rows = icons.split_rows(2);

        if !state.music.contains_key(&self.info.id) {
            // Not downloaded
            self.download.show();
            self.play.hide();

            self.download.update(rows[1], context);
            self.download.background = None;
            if self.download.state.hovered {
                self.download.color = context.theme.dark;
                self.download.background = Some(context.theme.light);
            }
            if self.download.state.clicked {
                state.download_music(self.info.id);
            }
        } else {
            self.download.hide();
            self.play.show();

            self.play.update(rows[1], context);
            self.play.background = None;
            if self.play.state.hovered {
                self.play.color = context.theme.dark;
                self.play.background = Some(context.theme.light);
            }
            if self.play.state.clicked {
                state.play_music(self.info.id);
            }
        }

        main.cut_left(context.layout_size);

        let mut author = main;
        let name = author.split_top(0.5);
        let margin = context.font_size * 0.2;
        author.cut_top(margin);

        self.name.update(name, context);
        self.name.align(vec2(0.0, 0.0));

        self.author.update(author, &mut context.scale_font(0.6)); // TODO: better
        self.author.align(vec2(0.0, 1.0));
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
