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

    pub confirm: Option<ConfirmWidget>,
    pub sync: Option<SyncWidget>,
    pub notifications: NotificationsWidget,

    pub level_select: LevelSelectUI,
    pub play_level: PlayLevelWidget,
    pub modifiers: ModifiersWidget,

    pub explore: ExploreWidget,

    pub leaderboard_head: TextWidget,
    pub leaderboard: LeaderboardWidget,
}

impl MenuUI {
    pub fn new(context: Context) -> Self {
        let geng = &context.geng;
        let assets = &context.assets;

        let mut explore = ExploreWidget::new(assets);
        explore.hide();

        Self {
            screen: WidgetState::new(),
            ctl_logo: IconWidget::new(assets.atlas.title()),
            separator: WidgetState::new(),

            options: OptionsButtonWidget::new(assets, 0.25),

            confirm: None,
            sync: None,
            notifications: NotificationsWidget::new(assets),

            level_select: LevelSelectUI::new(geng, assets),
            play_level: PlayLevelWidget::new(),
            modifiers: ModifiersWidget::new(assets),

            explore,

            leaderboard_head: TextWidget::new("Leaderboard")
                .rotated(Angle::from_degrees(90.0))
                .aligned(vec2(0.5, 0.5)),
            leaderboard: LeaderboardWidget::new(assets, true),

            context,
        }
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

        if let Some(confirm) = &mut self.confirm {
            let size = vec2(20.0, 10.0) * layout_size;
            let window = screen.align_aabb(size, vec2(0.5, 0.5));
            confirm.update(window, context);
            if confirm.confirm.state.mouse_left.clicked {
                confirm.window.show.going_up = false;
                state.confirm_action(self);
            } else if confirm.discard.state.mouse_left.clicked {
                confirm.window.show.going_up = false;
                state.confirm_popup = None;
            } else if confirm.window.show.time.is_min() {
                self.confirm = None;
            }

            // NOTE: When confirm is active, you cant interact with other widgets
            context.update_focus(true);
        } else if let Some(popup) = &state.confirm_popup {
            let mut confirm = ConfirmWidget::new(
                &self.context.assets,
                popup.title.clone(),
                popup.message.clone(),
            );
            confirm.window.show.going_up = true;
            self.confirm = Some(confirm);
        }

        for message in state.notifications.drain(..) {
            self.notifications.notify(message);
        }
        self.notifications.update(right, context);

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
                    ExploreAction::PlayMusic(group_id) => {
                        if let Some((_index, group)) = self.context.local.get_group_id(group_id) {
                            if let Some(music) = &group.local.music {
                                self.context.music.switch(music);
                            }
                        }
                    }
                    ExploreAction::PauseMusic => {
                        self.context.music.stop();
                    }
                    ExploreAction::GotoGroup(group_id) => {
                        if let Some((index, _group)) = self.context.local.get_group_id(group_id) {
                            self.explore.window.request = Some(WidgetRequest::Close);
                            self.level_select.select_tab(LevelSelectTab::Difficulty);
                            state.switch_group = Some(index);
                        }
                    }
                }
            }

            // NOTE: Everything below `explore` cannot get focused
            context.update_focus(true);
        }

        let action = self.level_select.update(left, state, context);
        if let Some(action) = action {
            match action {
                LevelSelectAction::SyncGroup(group_index) => {
                    let local = self.context.local.inner.borrow();
                    if let Some(group) = local.groups.get(group_index) {
                        self.sync = Some(SyncWidget::new(
                            &self.context.geng,
                            &self.context.assets,
                            group.clone(),
                            group_index,
                        ));
                    }
                }
                LevelSelectAction::EditLevel(group, level) => {
                    state.edit_level(group, Some(level));
                }
                LevelSelectAction::DeleteLevel(group, level) => {
                    state.popup_confirm(
                        ConfirmAction::DeleteLevel(group, level),
                        "delete this difficulty",
                    );
                }
                LevelSelectAction::EditGroup(group) => {
                    state.edit_level(group, None);
                }
                LevelSelectAction::DeleteGroup(group) => {
                    state.popup_confirm(ConfirmAction::DeleteGroup(group), "delete the group");
                }
            }
        } else if self
            .level_select
            .add_group
            .menu
            .browse
            .state
            .mouse_left
            .clicked
        {
            self.explore_groups();
        } else if self
            .level_select
            .add_group
            .menu
            .create
            .state
            .mouse_left
            .clicked
        {
            state.new_group();
        }

        let options = right.extend_positive(-vec2(2.0, 2.0) * layout_size);

        right.cut_left(5.0 * layout_size);
        right.cut_right(5.0 * layout_size);
        right.cut_top(3.5 * layout_size);
        right.cut_bottom(2.0 * layout_size);
        self.play_level.update(right, state, context);
        self.modifiers.update(right, state, context);
        state.update_board_meta();

        {
            // Leaderboard
            let main = screen;

            let size = vec2(layout_size * 22.0, main.height() - layout_size * 6.0);
            let head_size = vec2(font_size, layout_size * 8.0);
            let pos = main.align_pos(vec2(1.0, 0.5));

            let base_t = state
                .selected_level
                .as_ref()
                .map_or(0.0, |show| show.time.get_ratio());
            let base_t = crate::util::smoothstep(base_t);

            let hover_t = self.leaderboard.window.show.time.get_ratio();
            let hover_t = crate::util::smoothstep(hover_t);

            let slide =
                vec2(-1.0, 0.0) * (hover_t * (size.x + layout_size * 2.0) + base_t * head_size.x);

            let up = 0.6;
            let leaderboard = Aabb2::point(pos + vec2(head_size.x, 0.0) + slide)
                .extend_right(size.x)
                .extend_up(size.y * up)
                .extend_down(size.y * (1.0 - up));
            let leaderboard_head = Aabb2::point(pos + slide)
                .extend_right(head_size.x)
                .extend_symmetric(vec2(0.0, head_size.y) / 2.0);

            self.leaderboard.update_state(&state.leaderboard);
            self.leaderboard.update(leaderboard, context);
            self.leaderboard_head.update(leaderboard_head, context);
            context.update_focus(self.leaderboard.state.hovered);

            let hover = base_t > 0.0
                && (self.leaderboard.state.hovered || self.leaderboard_head.state.hovered);
            self.leaderboard.window.layout(
                hover,
                context.cursor.position.x < leaderboard.min.x && !hover,
            );

            context.update_focus(self.leaderboard.state.hovered);
        }

        self.options.update(options, context, state);
        context.update_focus(self.options.options.state.hovered);

        if let Some(sync) = &mut self.sync {
            let size = vec2(20.0, 17.0) * layout_size;
            let pos = screen.align_aabb(size, vec2(0.5, 0.5));
            sync.update(pos, context, state);
            context.update_focus(sync.state.hovered);
            if !sync.window.show.going_up && sync.window.show.time.is_min() {
                // Close window
                self.sync = None;
            }
        }

        !context.can_focus()
    }
}
