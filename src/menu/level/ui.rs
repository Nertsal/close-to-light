mod level;
mod modifiers;
mod select;

pub use self::{level::*, modifiers::*, select::*};

use super::*;

use crate::ui::{layout::AreaOps, widget::*};

use itertools::Itertools;

pub struct MenuUI {
    context: Context,
    pub screen: WidgetState,
    pub ctl_logo: IconWidget,
    pub separator: WidgetState,

    pub options: OptionsButtonWidget,

    pub sync: Option<SyncWidget>,
    pub sync_offset: vec2<f32>,

    pub level_select: LevelSelectUI,
    pub play_level: PlayLevelWidget,
    pub modifiers: ModifiersWidget,

    // pub panels: PanelsUI,
    pub explore: ExploreWidget,

    pub leaderboard: LeaderboardWidget,
    pub level_config: LevelConfigWidget,
}

impl MenuUI {
    pub fn new(context: Context) -> Self {
        let geng = &context.geng;
        let assets = &context.assets;

        let mut explore = ExploreWidget::new(assets);
        explore.hide();

        Self {
            screen: WidgetState::new(),
            ctl_logo: IconWidget::new(&assets.sprites.title),
            separator: WidgetState::new(),

            options: OptionsButtonWidget::new(assets, 0.25),

            sync: None,
            sync_offset: vec2::ZERO,

            level_select: LevelSelectUI::new(geng, assets),
            play_level: PlayLevelWidget::new(),
            modifiers: ModifiersWidget::new(),

            // panels: PanelsUI::new(assets),
            explore,

            leaderboard: LeaderboardWidget::new(assets),
            level_config: LevelConfigWidget::new(assets),

            context,
        }
    }

    fn explore_music(&mut self) {
        self.explore.window.request = Some(WidgetRequest::Open);
        self.explore.select_tab(ExploreTab::Music);
        self.explore.show();
    }

    fn explore_groups(&mut self) {
        // TODO: with music filter
        self.explore.window.request = Some(WidgetRequest::Open);
        self.explore.select_tab(ExploreTab::Group);
        self.explore.show();
    }

    /// Layout all the ui elements and return whether any of them is focused.
    pub fn layout(
        &mut self,
        state: &mut MenuState,
        screen: Aabb2<f32>,
        context: &mut UiContext,
    ) -> bool {
        // Fix aspect
        let screen = screen.fit_aabb(vec2(16.0, 9.0), vec2::splat(0.5));

        let layout_size = screen.height() * 0.03;
        let font_size = screen.height() * 0.06;

        context.screen = screen;
        context.layout_size = layout_size;
        context.font_size = font_size;

        self.screen.update(screen, context);

        let mut right = self.screen.position;
        let left = right.split_left(0.55);

        let separator = Aabb2::point(vec2(right.min.x, right.center().y))
            .extend_symmetric(vec2(0.1 * layout_size, screen.height() - 10.0 * layout_size) / 2.0);
        self.separator.update(separator, context);

        let mut left = left.extend_symmetric(-vec2(2.0, 3.0) * layout_size);
        let logo = left.cut_top(2.5 * layout_size);
        self.ctl_logo.update(logo, context);

        if self.explore.state.visible {
            let size = vec2(50.0, 30.0) * layout_size;
            let window = screen.align_aabb(size, vec2(0.5, 0.5));

            let slide_t = 1.0 - self.explore.window.show.time.get_ratio();
            let slide_t = crate::util::smoothstep(slide_t);
            let slide = vec2(0.0, screen.min.y - window.max.y) * slide_t;

            let mut temp_state = (self.context.local.clone(), None);
            self.explore
                .update(window.translate(slide), context, &mut temp_state);
            if let Some(action) = temp_state.1 {
                match action {
                    ExploreAction::PlayMusic(music_id) => {
                        if let Some(music) = self.context.local.get_music(music_id) {
                            self.context.music.switch(&music);
                        }
                    }
                    ExploreAction::GotoMusic(music_id) => {
                        self.explore.window.request = Some(WidgetRequest::Close);
                        self.level_select.select_tab(LevelSelectTab::Group);
                        state.switch_music = Some(music_id);
                    }
                    ExploreAction::GotoGroup(group_id) => {
                        if let Some((index, group)) = self
                            .context
                            .local
                            .inner
                            .borrow()
                            .groups
                            .iter()
                            .find(|(_, group)| group.meta.id == group_id)
                        {
                            self.explore.window.request = Some(WidgetRequest::Close);
                            self.level_select.select_tab(LevelSelectTab::Difficulty);
                            state.switch_music = Some(group.meta.music);
                            state.switch_group = Some(index);
                        }
                    }
                }
            }

            // NOTE: Everything below `explore` cannot get focused
            context.can_focus = false;
        }

        self.level_select.update(left, state, context);
        if self.level_select.add_music.state.clicked {
            self.explore_music();
        } else if self.level_select.add_group.state.clicked {
            self.explore_groups();
        }

        let options = right.extend_positive(-vec2(1.5, 1.5) * layout_size);
        let old_options = state.options.clone();
        self.options.update(options, context, state);
        if state.options != old_options {
            preferences::save(OPTIONS_STORAGE, &state.options);
        }

        right.cut_left(5.0 * layout_size);
        right.cut_right(5.0 * layout_size);
        right.cut_top(3.5 * layout_size);
        right.cut_bottom(2.0 * layout_size);
        self.play_level.update(right, state, context);
        self.modifiers.update(right, state, context);

        // // Margin
        // let mut main = screen
        //     .extend_uniform(-layout_size * 2.0)
        //     .extend_up(-layout_size * 2.0);

        // // Logo
        // let ctl_logo = main.cut_top(layout_size * 4.0);
        // self.ctl_logo.update(ctl_logo, context);
        // main.cut_top(layout_size * 3.0);

        // if let Some(sync) = &mut self.sync {
        //     let size = vec2(20.0, 17.0) * layout_size;
        //     let pos = Aabb2::point(screen.center() + self.sync_offset).extend_symmetric(size / 2.0);
        //     sync.update(pos, context, &mut state.local.borrow_mut());
        //     context.update_focus(sync.state.hovered);
        //     if !sync.window.show.going_up && sync.window.show.time.is_min() {
        //         // Close window
        //         self.sync = None;
        //         self.sync_offset = vec2::ZERO;
        //     }
        // }

        // let base_t = if state.level_up {
        //     1.0
        // } else {
        //     state
        //         .show_level
        //         .as_ref()
        //         .map_or(0.0, |show| show.time.get_ratio())
        // };
        // let base_t = crate::util::smoothstep(base_t) * 2.0 - 1.0;

        // self.panels.update(state, context);

        // let cursor_high = context.cursor.position.y > main.max.y;

        // {
        //     // Leaderboard
        //     let width = layout_size * 22.0;
        //     let height = main.height() + layout_size * 2.0;

        //     let leaderboard =
        //         Aabb2::point(main.bottom_right() + vec2(0.0, 2.0) * base_t * layout_size)
        //             .extend_left(width)
        //             .extend_down(height);

        //     let t = self.leaderboard.window.show.time.get_ratio();
        //     let t = crate::util::smoothstep(t);
        //     let offset = main.height() * t;

        //     let leaderboard = leaderboard.translate(vec2(0.0, offset));

        //     self.leaderboard.update_state(
        //         &state.leaderboard.status,
        //         &state.leaderboard.loaded,
        //         &state.player.info,
        //     );
        //     update!(self.leaderboard, leaderboard);
        //     context.update_focus(self.leaderboard.state.hovered);

        //     self.leaderboard.window.layout(
        //         self.leaderboard.state.hovered,
        //         self.leaderboard.close.state.clicked
        //             || cursor_high && !self.leaderboard.state.hovered,
        //     );
        // }

        // {
        //     // Mods
        //     let width = layout_size * 30.0;
        //     let height = layout_size * 20.0;

        //     let t = self.level_config.window.show.time.get_ratio();
        //     let t = crate::util::smoothstep(t);
        //     let offset = height * t;
        //     let config = Aabb2::point(main.bottom_left() + vec2(0.0, 2.0) * base_t * layout_size)
        //         .extend_right(width)
        //         .extend_down(height)
        //         .translate(vec2(0.0, offset));

        //     self.level_config.set_config(&state.config);
        //     update!(self.level_config, config);
        //     context.update_focus(self.level_config.state.hovered);
        //     let old_config = state.config.clone();
        //     self.level_config.update_config(&mut state.config);
        //     if old_config != state.config && self.leaderboard.window.show.going_up {
        //         self.leaderboard.window.request = Some(WidgetRequest::Reload);
        //     }

        //     self.level_config.window.layout(
        //         self.level_config.state.hovered,
        //         self.level_config.close.state.clicked
        //             || cursor_high && !self.level_config.state.hovered,
        //     );
        // }

        // // Margin
        // main.cut_left(layout_size * 0.5);
        // if let Some(sync) = self.level_select.update(main, state, context) {
        //     self.sync = Some(sync);
        // }

        !context.can_focus
    }
}
