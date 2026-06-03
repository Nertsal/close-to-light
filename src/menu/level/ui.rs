mod level;
mod modifiers;
mod select;

pub use self::{level::*, modifiers::*, select::*};

use super::*;

use crate::ui::{layout::AreaOps, widget::*};

use ctl_local::LocalMusic;
use ctl_ui::UiWindow;
use ctl_util::{Change, TimeInterpolation};
use itertools::Itertools;

pub struct MenuUI {
    context: Context,
    pub screen: WidgetState,

    // pub ctl_logo: IconWidget,
    // pub separator: WidgetState,
    pub exit: ButtonWidget,
    pub options: OptionsButtonWidget,

    pub confirm: Option<ConfirmWidget>,
    #[cfg(feature = "online")]
    pub sync: Option<SyncWidget>,
    pub notifications: NotificationsWidget,

    pub level_select: LevelSelectUI,
    pub play_level: PlayLevelWidget,
    pub modifiers: ModifiersWidget,
    pub practice_button: ButtonWidget,

    pub practice: PracticeWidget,
    pub explore: ExploreWidget,

    pub leaderboard_head: TextWidget,
    pub leaderboard: LeaderboardWidget,
}

pub struct PracticeWidget {
    pub state: WidgetState,
    pub window: UiWindow<()>,

    pub title: TextWidget,
    pub close: IconButtonWidget,
    pub confirm: IconButtonWidget,

    pub preview: WidgetState,
    pub preview_time: TimeInterpolation,
    pub level_duration: Time,
    pub cached_level: Option<Rc<Level>>,
    pub rendered: Option<LevelState>,

    pub timeline: WidgetState,
    pub timeline_interactive: WidgetState,
    pub timeline_current_time: TextWidget,
    pub timeline_selected_text: TextWidget,
    pub timeline_start: WidgetState,
    pub timeline_end: WidgetState,
    pub timeline_current: WidgetState,
    pub timeline_from: WidgetState,
    pub timeline_to: WidgetState,
    pub select_from: Time,
    pub select_to: Time,
}

impl PracticeWidget {
    pub fn new(assets: &Assets) -> Self {
        Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),

            title: TextWidget::new("Practice Section"),
            close: IconButtonWidget::new_close_button(assets.atlas.button_close()),
            confirm: IconButtonWidget::new(
                assets.atlas.button_confirm(),
                ThemeColor::Highlight,
                IconBackgroundKind::Circle,
            ),

            preview: WidgetState::new(),
            preview_time: TimeInterpolation::new(),
            level_duration: 0,
            cached_level: None,
            rendered: None,

            timeline: WidgetState::new(),
            timeline_interactive: WidgetState::new(),
            timeline_current_time: TextWidget::new("0:00"),
            timeline_selected_text: TextWidget::new("0:00 - 0:00"),
            timeline_start: WidgetState::new(),
            timeline_end: WidgetState::new(),
            timeline_current: WidgetState::new(),
            timeline_from: WidgetState::new(),
            timeline_to: WidgetState::new(),
            select_from: 0,
            select_to: 0,
        }
    }

    pub fn reload_level(&mut self, _music: &LocalMusic, level: &LevelFull) {
        // TODO: music waveform maybe
        self.cached_level = Some(level.data.clone());
        self.level_duration = level.data.last_time();
        self.preview_time.snap_to(0);
        self.select_from = 0;
        self.select_to = 0;
    }

    pub fn selected_range(&self) -> (Time, Time) {
        (
            std::cmp::min(self.select_from, self.select_to),
            std::cmp::max(self.select_from, self.select_to),
        )
    }

    pub fn update(&mut self, position: Aabb2<f32>, state: &mut MenuState, context: &UiContext) {
        let spacing = context.font_size * 0.2;
        let title_height = context.font_size * 1.2;
        let preview_res = crate::render::PREVIEW_RESOLUTION.as_f32();
        let preview_height = position.width() / preview_res.aspect();
        let timeline_height = context.font_size * 1.8;
        let timeline_space = context.font_size * 0.3;
        let position = position.with_height(
            spacing + title_height + preview_height + timeline_height + timeline_space,
            0.0,
        );

        self.window.update(context.delta_time);
        let t = 1.0 - self.window.show.time.get_ratio();
        let t = crate::util::smoothstep(t);
        let mut position = position.translate(vec2(0.0, context.screen.min.y - position.max.y) * t);
        self.state.update(position, context);

        position.cut_top(spacing);
        let title = position
            .cut_top(title_height)
            .extend_symmetric(-vec2(spacing, 0.0));
        self.title.update(title, context);
        self.close.update(
            title.align_aabb(vec2::splat(title.height()), vec2(0.0, 0.5)),
            context,
        );
        self.confirm.update(
            title.align_aabb(vec2::splat(title.height()), vec2(1.0, 0.5)),
            context,
        );
        if self.close.icon.state.mouse_left.clicked {
            self.window.request = Some(WidgetRequest::Close);
        }
        if self.confirm.icon.state.mouse_left.clicked {
            state.practice_section = Some(self.selected_range());
        }

        // Selected area text
        self.timeline_selected_text.update(
            position.align_aabb(vec2(10.0, 0.8) * context.font_size, vec2(0.5, 1.0)),
            context,
        );
        self.timeline_selected_text.text = format!(
            "{} - {}",
            ctl_util::display_time(self.select_from, false),
            ctl_util::display_time(self.select_to, false)
        )
        .into();

        // Timeline
        position.cut_top(timeline_space);
        let timeline_pos = position.cut_top(timeline_height);
        self.timeline_interactive.update(timeline_pos, context);
        self.timeline.update(
            timeline_pos.extend_symmetric(-vec2(spacing * 2.0, 0.0)),
            context,
        );

        // Cursor time text
        self.timeline_current_time
            .state
            .set_visibility(self.timeline_interactive.hovered);
        self.timeline_current_time.update(
            timeline_pos.align_aabb(vec2(10.0, 0.8) * context.font_size, vec2(0.5, 0.0)),
            context,
        );
        self.timeline_current_time.text =
            ctl_util::display_time(self.preview_time.target, false).into();

        let tick = |time| {
            Aabb2::point(
                self.timeline
                    .position
                    .align_pos(vec2(time as f32 / self.level_duration as f32, 0.5)),
            )
            .extend_symmetric(vec2(0.1, 0.3) * context.font_size / 2.0)
        };
        self.timeline_start.update(tick(0), context);
        self.timeline_end.update(tick(self.level_duration), context);
        self.timeline_current
            .update(tick(self.preview_time.value), context);
        self.timeline_from.update(tick(self.select_from), context);
        self.timeline_to.update(tick(self.select_to), context);

        let local = &state.context.local;
        if let Some(show_group) = &state.selected_level
            && let Some(group) = local.get_group(show_group.data)
            && let Some(music) = &group.local.music
            && let Some(show_level) = &state.selected_diff
            && let Some(level) = local.get_level(show_group.data, show_level.data)
        {
            if self
                .cached_level
                .as_ref()
                .is_none_or(|cached| !Rc::ptr_eq(cached, &level.data))
            {
                self.reload_level(music, &level);
            }

            let t = (context.cursor.position.x - self.timeline.position.min.x)
                / self.timeline.position.width();
            let t = t.clamp(0.0, 1.0);
            let cursor_time = (self.level_duration as f32 * t) as Time;
            let cursor_time = level.data.timing.snap_to_beat(cursor_time, BeatTime::WHOLE);
            if self.timeline_interactive.mouse_left.just_pressed {
                self.select_from = cursor_time;
                self.select_to = cursor_time;
            }
            if self.timeline_interactive.mouse_left.pressed.is_some() {
                self.select_to = cursor_time;
            }
            if self.timeline_interactive.hovered {
                self.preview_time.scroll_time(Change::Set(cursor_time));
            }

            if let Some(level) = &self.cached_level {
                let mut vfx = Vfx::new();
                self.rendered = Some(LevelState::render(
                    level,
                    self.preview_time.value,
                    None,
                    Some(&mut vfx),
                ));
            }
        }

        let preview_pos = position.cut_top(preview_height);
        self.preview.update(preview_pos, context);
        self.preview_time.update(r32(context.delta_time));

        context.update_focus(self.state.hovered);
    }
}

impl MenuUI {
    pub fn new(context: Context) -> Self {
        let geng = &context.geng;
        let assets = &context.assets;

        let mut explore = ExploreWidget::new(assets);
        explore.hide();

        Self {
            screen: WidgetState::new(),

            // ctl_logo: IconWidget::new(assets.atlas.title()),
            // separator: WidgetState::new(),
            exit: ButtonWidget::new("Back"),
            options: OptionsButtonWidget::new(assets, 0.25),

            confirm: None,
            #[cfg(feature = "online")]
            sync: None,
            notifications: NotificationsWidget::new(assets),

            level_select: LevelSelectUI::new(geng, assets),
            play_level: PlayLevelWidget::new(),
            modifiers: ModifiersWidget::new(assets),
            practice_button: ButtonWidget::new("Practice"),

            practice: PracticeWidget::new(assets),
            explore,

            leaderboard_head: TextWidget::new("Leaderboard")
                .rotated(Angle::from_degrees(90.0))
                .aligned(vec2(0.5, 0.5)),
            leaderboard: LeaderboardWidget::new(assets, true),

            context,
        }
    }

    #[cfg(feature = "online")]
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

        let exit = screen
            .align_aabb(vec2(2.2, 1.0) * context.font_size, vec2(0.0, 1.0))
            .translate(vec2(1.0, -0.5) * context.layout_size);

        let mut right = self.screen.position;
        let left = right.split_left(0.55);

        // let separator = Aabb2::point(vec2(right.min.x, right.center().y))
        //     .extend_symmetric(vec2(0.1 * layout_size, screen.height() - 10.0 * layout_size) / 2.0);
        // self.separator.update(separator, context);

        let left = left.extend_symmetric(-vec2(2.0, 3.0) * layout_size);
        // let logo = left.cut_top(2.5 * layout_size);
        // self.ctl_logo.update(logo, context);

        {
            // Practice
            let position = left;
            self.practice.update(position, state, context);
        }

        if let Some(confirm) = &mut self.confirm {
            let size = vec2(20.0, 10.0) * layout_size;
            let window = screen.align_aabb(size, vec2(0.5, 0.5));
            confirm.update(window, context);
            if confirm.confirm_icon.icon.state.mouse_left.clicked
                || confirm.confirm_text.text.state.mouse_left.clicked
            {
                confirm.window.show.going_up = false;
                state.confirm_action(self);
            } else if confirm.discard_icon.icon.state.mouse_left.clicked
                || confirm.discard_text.text.state.mouse_left.clicked
            {
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
                popup.confirm_text.clone(),
                popup.confirm_color,
                popup.discard_text.clone(),
            );
            confirm.window.show.going_up = true;
            self.confirm = Some(confirm);
        }

        #[cfg(feature = "online")]
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
                        if let Some((_index, group)) = self.context.local.get_group_id(group_id)
                            && let Some(music) = &group.local.music
                        {
                            self.context.music.switch(music, true);
                        }
                    }
                    ExploreAction::PauseMusic => {
                        self.context.music.stop();
                    }
                    ExploreAction::GotoGroup(group_id) => {
                        if let Some((index, _group)) = self.context.local.get_group_id(group_id) {
                            self.explore.window.request = Some(WidgetRequest::Close);
                            state.switch_level = Some(index);
                        }
                    }
                }
            }

            // NOTE: Everything below `explore` cannot get focused
            context.update_focus(true);
        }

        let level_select_t = state
            .selected_level
            .as_ref()
            .filter(|show| {
                !show.going_up && state.switch_level.is_none()
                    || show.going_up && state.last_selected_level.is_none()
            })
            .map_or(
                if state.last_selected_level.is_some() {
                    1.0
                } else {
                    0.0
                },
                |show| show.time.get_ratio(),
            );
        let level_select_t = crate::util::smoothstep(1.0 - level_select_t);
        let level_select = left.translate(vec2(
            (screen.center().x - left.center().x) * level_select_t,
            0.0,
        ));

        let action = self.level_select.update(level_select, state, context);
        if let Some(action) = action {
            match action {
                #[cfg(feature = "online")]
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
                LevelSelectAction::EditDifficulty(group, level) => {
                    #[cfg(not(feature = "editor"))]
                    {
                        let _ = (group, level);
                        state.editor_not_available();
                    }
                    #[cfg(feature = "editor")]
                    state.edit_level(group, Some(level));
                }
                LevelSelectAction::DeleteDifficulty(group, level) => {
                    state.popup_confirm(
                        ConfirmAction::DeleteLevel(group, level),
                        "delete difficulty",
                        "delete",
                        ThemeColor::Danger,
                        "cancel",
                    );
                }
                LevelSelectAction::EditGroup(group) => {
                    #[cfg(not(feature = "editor"))]
                    {
                        let _ = group;
                        state.editor_not_available();
                    }
                    #[cfg(feature = "editor")]
                    state.edit_level(group, None);
                }
                LevelSelectAction::DeleteGroup(group) => {
                    state.popup_confirm(
                        ConfirmAction::DeleteGroup(group),
                        "delete group",
                        "delete",
                        ThemeColor::Danger,
                        "cancel",
                    );
                }
            }
        }

        let options = right.extend_positive(-vec2(2.0, 0.5) * layout_size);

        right.cut_left(2.0 * layout_size);
        right.cut_right(5.0 * layout_size);
        right.cut_top(3.5 * layout_size);
        right.cut_bottom(2.0 * layout_size);
        self.play_level.update(right, state, context);
        self.modifiers.update(right, state, context);
        {
            // Practice button
            // Slide in when a level is selected
            let pos = right
                .align_aabb(
                    vec2(7.0 * context.layout_size, 1.1 * context.font_size),
                    vec2(0.5, 0.0),
                )
                .translate(vec2(8.0 * context.layout_size, 0.0));
            let t = self.modifiers.t;
            let t = crate::util::smoothstep(t);
            let slide = vec2(0.0, context.screen.min.y - pos.max.y);
            let pos = pos.translate(slide * (1.0 - t));
            self.practice_button.update(pos, context);
            if self.practice_button.text.state.mouse_left.clicked {
                if self.practice.window.show.going_up {
                    self.practice.window.request = Some(WidgetRequest::Close);
                } else {
                    self.practice.window.request = Some(WidgetRequest::Open);
                }
            }
        }
        state.update_board_meta();

        {
            // Leaderboard
            let main = screen;

            let size = vec2(layout_size * 22.0, main.height() - layout_size * 6.0);
            let head_size = vec2(font_size, layout_size * 8.0);
            let pos = main.align_pos(vec2(1.0, 0.5));

            let hover_t = self.leaderboard.window.show.time.get_ratio();
            let hover_t = crate::util::smoothstep(hover_t);

            let base_t = state
                .selected_diff
                .as_ref()
                .map_or(0.0, |show| show.time.get_ratio());
            let base_t = crate::util::smoothstep(base_t).min(1.0 - hover_t);

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
            self.leaderboard
                .update(leaderboard, &mut context.scale_font(0.84));
            self.leaderboard_head
                .update(leaderboard_head, &context.scale_font(0.9));
            context.update_focus(self.leaderboard.state.hovered);

            let hover = base_t > 0.0
                && (self.leaderboard.state.hovered || self.leaderboard_head.state.hovered);
            self.leaderboard.window.layout(
                hover,
                context.cursor.position.x < leaderboard.min.x && !hover,
            );

            context.update_focus(self.leaderboard.state.hovered);
        }

        self.exit.update(exit, &context.scale_font(0.8));
        if self.exit.text.state.mouse_left.clicked {
            state.exit = true;
        }

        self.options.update(options, context, &mut state.options);
        context.update_focus(self.options.options.state.hovered);

        !context.can_focus()
    }
}
