use super::*;

use crate::{layout::AreaOps, util::ScrollState};

use ctl_assets::Assets;
use ctl_core::types::{Id, LevelSetInfo};
use ctl_local::{CacheState, LevelCache};

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

    pub reload: IconButtonWidget,
    pub close: IconButtonWidget,

    pub levels: ExploreLevelsWidget,
    refetch: bool,
}

pub struct ExploreLevelsWidget {
    assets: Rc<Assets>,
    pub state: WidgetState,
    pub status: TextWidget,
    pub scroll: ScrollState,
    pub items_state: WidgetState,
    pub items: Vec<LevelItemWidget>,
}

pub struct LevelItemWidget {
    pub state: WidgetState,
    pub download: IconButtonWidget,
    pub downloading: IconWidget,
    pub play_music: IconButtonWidget,
    pub pause_music: IconButtonWidget,
    pub goto: IconButtonWidget,
    pub info: LevelSetInfo,
    pub name: TextWidget,
    pub author: TextWidget,
}

impl ExploreWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),

            reload: IconButtonWidget::new_normal(assets.atlas.reset()),
            close: IconButtonWidget::new_close_button(assets.atlas.button_close()),

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
        if self.reload.icon.state.mouse_left.clicked {
            self.refetch = true;
        }

        let close = vec2::splat(2.0) * context.layout_size;
        let close = bar.align_aabb(close, vec2(1.0, 1.0));
        self.close.update(close, context);
        if self.close.icon.state.mouse_left.clicked {
            self.window.request = Some(WidgetRequest::Close);
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
            scroll: ScrollState::new(),
            items_state: WidgetState::new(),
            items: Vec::new(),
        }
    }

    fn load(&mut self, groups: &CacheState<Vec<LevelSetInfo>>) {
        self.status.show();
        match groups {
            CacheState::Offline => self.status.text = "Offline :(".into(),
            CacheState::Loading => self.status.text = "Loading...".into(),
            CacheState::Loaded(groups) => {
                if groups.is_empty() {
                    self.status.text = "Empty :(".into();
                } else {
                    self.status.hide();
                    if self.items.len() > groups.len() {
                        self.items.drain(groups.len() + 1..self.items.len());
                    }
                    self.items.extend((self.items.len()..groups.len()).map(|_| {
                        let mut widget = LevelItemWidget {
                            state: WidgetState::new(),
                            download: IconButtonWidget::new_normal(self.assets.atlas.download()),
                            downloading: IconWidget::new(self.assets.atlas.loading()),
                            play_music: IconButtonWidget::new_normal(self.assets.atlas.play()),
                            pause_music: IconButtonWidget::new_normal(self.assets.atlas.pause()),
                            goto: IconButtonWidget::new_normal(self.assets.atlas.goto()),
                            name: TextWidget::new(""),
                            author: TextWidget::new(""),
                            info: LevelSetInfo::default(),
                        };
                        widget.downloading.hide();
                        widget
                    }));
                    for (widget, info) in self.items.iter_mut().zip(groups) {
                        let artists = info.music.authors();
                        let authors = info.mappers();
                        widget.name.text = info.music.name.clone();
                        widget.author.text = format!("by {artists} mapped by {authors}").into();
                        widget.info = info.clone();
                    }
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

        // Scroll
        self.scroll.drag(context, &self.state);

        let main = position;

        self.items_state.update(main, context);
        self.status.update(main, context);

        let main = main.translate(vec2(0.0, -self.scroll.state.current));
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

        self.scroll
            .overflow(context.delta_time, height, main.height());
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
            .any(|(_, group)| group.local.meta.id == self.info.id)
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
            self.play_music.hide();
            self.pause_music.hide();
            self.download.update(rows[1], context);
            self.downloading.update(rows[1], context);
            if self.download.icon.state.mouse_left.clicked {
                state.download_group(self.info.id);
            }
        } else {
            self.download.hide();
            self.downloading.hide();

            if context.context.music.is_playing() == Some(self.info.music.id) {
                self.play_music.hide();
                self.pause_music.show();
                self.pause_music.update(rows[0], context);
                if self.pause_music.icon.state.mouse_left.clicked {
                    *action = Some(ExploreAction::PauseMusic);
                }
            } else {
                self.pause_music.hide();
                self.play_music.show();
                self.play_music.update(rows[0], context);
                if self.play_music.icon.state.mouse_left.clicked {
                    *action = Some(ExploreAction::PlayMusic(self.info.id));
                }
            }

            self.goto.show();
            self.goto.update(rows[1], context);
            if self.goto.icon.state.mouse_left.clicked {
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
