use ctl_client::core::types::{GroupInfo, Id, MusicInfo};

use super::*;

use crate::{
    local::{CacheState, LevelCache},
    prelude::Assets,
    ui::layout::AreaOps,
};

#[derive(Debug, Clone, Copy)]
pub enum ExploreAction {
    PlayMusic(Id),
    PauseMusic,
    GotoGroup(Id),
}

#[derive(Debug, Clone, Copy)]
pub enum ExploreTab {
    Group,
}

pub struct ExploreWidget {
    pub state: WidgetState,
    pub window: UiWindow<()>,

    pub tabs: WidgetState,
    pub tab_music: TextWidget,
    pub tab_levels: TextWidget,

    pub reload: IconButtonWidget,
    pub close: IconButtonWidget,
    pub separator: WidgetState,

    pub levels: ExploreLevelsWidget,
    refetch: bool,
}

pub struct ExploreLevelsWidget {
    assets: Rc<Assets>,
    pub state: WidgetState,
    pub status: TextWidget,
    pub scroll: f32,
    pub target_scroll: f32,
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
    pub download: IconButtonWidget,
    pub downloading: IconWidget,
    pub goto: IconButtonWidget,
    pub info: GroupInfo,
    pub name: TextWidget,
    pub author: TextWidget,
}

pub struct MusicItemWidget {
    pub state: WidgetState,
    pub download: IconButtonWidget,
    pub downloading: IconWidget,
    pub play: IconButtonWidget,
    pub pause: IconButtonWidget,
    pub goto: IconButtonWidget,
    pub info: MusicInfo,
    pub name: TextWidget,
    pub author: TextWidget,
}

impl ExploreWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),

            tabs: WidgetState::new(),
            tab_music: TextWidget::new("Music"),
            tab_levels: TextWidget::new("Levels"),

            reload: IconButtonWidget::new_normal(assets.atlas.reset()),
            close: IconButtonWidget::new_close_button(assets.atlas.button_close()),
            separator: WidgetState::new(),

            levels: ExploreLevelsWidget::new(assets),

            refetch: true,
        }
    }

    pub fn select_tab(&mut self, tab: ExploreTab) {
        self.levels.hide();

        match tab {
            ExploreTab::Group => self.levels.show(),
        }
    }
}

impl StatefulWidget for ExploreWidget {
    type State<'a> = (Rc<LevelCache>, Option<ExploreAction>);

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
        (state, action): &mut Self::State<'_>,
    ) {
        // TODO: better
        if !self.state.visible {
            return;
        }

        if std::mem::take(&mut self.refetch) {
            state.fetch_groups();
        }

        self.state.update(position, context);
        self.window.update(context.delta_time);
        if self.window.show.time.is_min() {
            self.hide();
        }

        self.levels.load(&state.inner.borrow().group_list);

        let mut main = position;
        main.cut_top(context.layout_size * 1.0);
        let bar = main.cut_top(context.font_size * 1.2);

        let bar = bar.extend_symmetric(-vec2(1.0, 0.0) * context.layout_size);

        let reload = vec2::splat(2.0) * context.layout_size;
        let reload = bar.align_aabb(reload, vec2(0.0, 1.0));
        self.reload.update(reload, context);
        if self.reload.state.clicked {
            self.refetch = true;
        }

        let close = vec2::splat(2.0) * context.layout_size;
        let close = bar.align_aabb(close, vec2(1.0, 1.0));
        self.close.update(close, context);
        if self.close.state.clicked {
            self.window.request = Some(WidgetRequest::Close);
        }

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

        if self.tab_levels.state.clicked {
            self.select_tab(ExploreTab::Group);
        }

        let main = main.extend_uniform(-context.font_size * 0.5);
        let mut state = (state.clone(), None);
        self.levels.update(main, context, &mut state);
        *action = state.1;
    }
}

impl ExploreLevelsWidget {
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

    fn load(&mut self, groups: &CacheState<Vec<GroupInfo>>) {
        self.items.clear();
        self.status.show();
        match groups {
            CacheState::Offline => self.status.text = "Offline :(".into(),
            CacheState::Loading => self.status.text = "Loading...".into(),
            CacheState::Loaded(groups) => {
                if groups.is_empty() {
                    self.status.text = "Empty :(".into();
                } else {
                    self.status.hide();
                    self.items = groups
                        .iter()
                        .map(|info| {
                            let artists = info.music.authors();
                            let authors = info.mappers();

                            let mut widget = LevelItemWidget {
                                state: WidgetState::new(),
                                download: IconButtonWidget::new_normal(
                                    self.assets.atlas.download(),
                                ),
                                downloading: IconWidget::new(self.assets.atlas.loading()),
                                goto: IconButtonWidget::new_normal(self.assets.atlas.goto()),
                                name: TextWidget::new(info.music.name.clone()),
                                author: TextWidget::new(format!(
                                    "by {} mapped by {}",
                                    artists, authors
                                )),
                                info: info.clone(),
                            };
                            widget.downloading.hide();
                            widget
                        })
                        .collect();
                }
            }
        }
    }
}

impl StatefulWidget for ExploreLevelsWidget {
    type State<'a> = (Rc<LevelCache>, Option<ExploreAction>);

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
        (state, action): &mut Self::State<'_>,
    ) {
        // TODO: better
        if !self.state.visible {
            return;
        }
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
            let mut state = (state.clone(), None);
            row.update(position, context, &mut state);
            if let Some(act) = state.1 {
                *action = Some(act);
            }
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
}

impl StatefulWidget for LevelItemWidget {
    type State<'a> = (Rc<LevelCache>, Option<ExploreAction>);

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
        (state, action): &mut Self::State<'_>,
    ) {
        // TODO: better
        if !self.state.visible {
            return;
        }
        self.state.update(position, context);

        let mut main = position.extend_uniform(-context.font_size * 0.2);

        let icons = main.cut_left(context.font_size);
        let rows = icons.split_rows(2);

        if !state
            .inner
            .borrow()
            .groups
            .iter()
            .any(|(_, group)| group.local.data.id == self.info.id)
        {
            // Not downloaded
            if state.is_downloading_group().contains(&self.info.id) {
                self.downloading.show();
                self.download.hide();
            } else {
                self.downloading.hide();
                self.download.show();
            }

            self.goto.hide();
            self.download.update(rows[1], context);
            self.downloading.update(rows[1], context);
            if self.download.state.clicked {
                state.download_group(self.info.id);
            }
        } else {
            self.download.hide();
            self.goto.show();
            self.goto.update(rows[1], context);
            if self.goto.state.clicked {
                *action = Some(ExploreAction::GotoGroup(self.info.id));
            }
        }

        main.cut_left(context.layout_size);

        let mut author = main;
        let name = author.split_top(0.5);
        let margin = context.font_size * 0.2;
        author.cut_top(margin);

        self.name.update(name, context);
        self.name.align(vec2(0.0, 0.0));

        self.author.update(author, &context.scale_font(0.6)); // TODO: better
        self.author.align(vec2(0.0, 1.0));
    }
}
