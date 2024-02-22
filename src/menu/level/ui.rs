use super::*;

use crate::ui::{layout::AreaOps, widget::*};

pub struct MenuUI {
    pub screen: WidgetState,
    pub ctl_logo: WidgetState,
    pub groups_state: WidgetState,
    pub groups: Vec<GroupWidget>,
    pub levels_state: WidgetState,
    pub levels: Vec<LevelWidget>,
    pub options_head: TextWidget,
    pub options: OptionsWidget,
    pub profile_head: IconWidget,
    pub profile: ProfileWidget,
    pub leaderboard: LeaderboardWidget,
    pub level_config: LevelConfigWidget,
}

impl MenuUI {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            screen: WidgetState::new(),
            ctl_logo: default(),
            groups_state: default(),
            groups: Vec::new(),
            levels_state: default(),
            levels: Vec::new(),
            options_head: TextWidget::new("Options"),
            options: OptionsWidget::new(
                Options::default(),
                vec![
                    // TODO: custom
                    PaletteWidget::new("Classic", Theme::classic()),
                    PaletteWidget::new("Test", Theme::test()),
                ],
            ),
            profile_head: IconWidget::new(&assets.sprites.head),
            profile: ProfileWidget::new(),
            leaderboard: LeaderboardWidget::new(assets),
            level_config: LevelConfigWidget::new(assets),
        }
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

        context.layout_size = layout_size;
        context.font_size = screen.height() * 0.06;

        macro_rules! update {
            ($widget:expr, $position:expr) => {{
                $widget.update($position, context);
            }};
            ($widget:expr, $position:expr, $state:expr) => {{
                $widget.update($position, context, $state);
            }};
        }

        update!(self.screen, screen);

        // Margin
        let mut main = screen
            .extend_uniform(-layout_size * 2.0)
            .extend_up(-layout_size * 2.0);

        // Logo
        let ctl_logo = main.cut_top(layout_size * 4.0);
        update!(self.ctl_logo, ctl_logo);
        main.cut_top(layout_size * 3.0);

        let base_t = if state.level_up {
            1.0
        } else {
            state
                .show_level
                .as_ref()
                .map_or(0.0, |show| show.time.get_ratio())
        };
        let base_t = crate::util::smoothstep(base_t) * 2.0 - 1.0;

        let mut top_bar = screen.clone().cut_top(context.font_size * 1.2);
        top_bar.cut_right(context.layout_size * 7.0);

        let profile_head = top_bar.cut_right(context.font_size * 1.2);
        top_bar.cut_right(context.layout_size * 3.0);

        let options_head = top_bar.cut_right(context.font_size * 3.5);

        let (options_head, options) = {
            // Options
            let width = layout_size * 50.0;
            let height = layout_size * 15.0;

            let options = Aabb2::point(screen.align_pos(vec2(0.5, 1.0)))
                .extend_symmetric(vec2(width, 0.0) / 2.0)
                .extend_up(height);

            let t = self.options.window.show.time.get_ratio();
            let t = crate::util::smoothstep(t);
            let offset = -options.height() * t;

            (
                options_head.translate(vec2(0.0, offset)),
                options.translate(vec2(0.0, offset)),
            )
        };

        let (profile_head, profile) = {
            // Profile
            let width = layout_size * 15.0;
            let height = layout_size * 17.0;

            let profile = Aabb2::point(profile_head.top_right())
                .extend_right(width * 0.1)
                .extend_left(width * 0.9)
                .extend_up(height);

            let t = self.profile.window.show.time.get_ratio();
            let t = crate::util::smoothstep(t);
            let offset = -profile.height() * t;

            (
                profile_head.translate(vec2(0.0, offset)),
                profile.translate(vec2(0.0, offset)),
            )
        };

        // Options
        let old_options = state.options.clone();
        update!(self.options, options, &mut state.options);
        context.update_focus(self.options.state.hovered);
        if state.options != old_options {
            preferences::save(OPTIONS_STORAGE, &state.options);
        }

        self.options.window.layout(
            self.options_head.state.hovered,
            !self.options.state.hovered && !self.options_head.state.hovered,
        );

        // Profile
        update!(self.profile, profile, &mut state.leaderboard);
        context.update_focus(self.profile.state.hovered);

        self.profile.window.layout(
            self.profile_head.state.hovered,
            !self.profile.state.hovered && !self.profile_head.state.hovered,
        );

        // Heads
        update!(self.options_head, options_head);
        context.update_focus(self.options_head.state.hovered);

        update!(self.profile_head, profile_head);
        context.update_focus(self.profile_head.state.hovered);

        let cursor_high = context.cursor.position.y > main.max.y;

        {
            // Leaderboard
            let width = layout_size * 22.0;
            let height = main.height() + layout_size * 2.0;

            let leaderboard =
                Aabb2::point(main.bottom_right() + vec2(0.0, 2.0) * base_t * layout_size)
                    .extend_left(width)
                    .extend_down(height);

            let t = self.leaderboard.window.show.time.get_ratio();
            let t = crate::util::smoothstep(t);
            let offset = main.height() * t;

            let leaderboard = leaderboard.translate(vec2(0.0, offset));

            self.leaderboard.update_state(
                &state.leaderboard.status,
                &state.leaderboard.loaded,
                &state.player.info,
            );
            update!(self.leaderboard, leaderboard);
            context.update_focus(self.leaderboard.state.hovered);

            self.leaderboard.window.layout(
                self.leaderboard.state.hovered,
                self.leaderboard.close.text.state.clicked
                    || cursor_high && !self.leaderboard.state.hovered,
            );
        }

        {
            // Mods
            let width = layout_size * 30.0;
            let height = layout_size * 20.0;

            let t = self.level_config.window.show.time.get_ratio();
            let t = crate::util::smoothstep(t);
            let offset = height * t;
            let config = Aabb2::point(main.bottom_left() + vec2(0.0, 2.0) * base_t * layout_size)
                .extend_right(width)
                .extend_down(height)
                .translate(vec2(0.0, offset));

            self.level_config.set_config(&state.config);
            update!(self.level_config, config);
            context.update_focus(self.level_config.state.hovered);
            let old_config = state.config.clone();
            self.level_config.update_config(&mut state.config);
            if old_config != state.config && self.leaderboard.window.show.going_up {
                self.leaderboard.window.request = Some(WidgetRequest::Reload);
            }

            self.level_config.window.layout(
                self.level_config.state.hovered,
                self.level_config.close.text.state.clicked
                    || cursor_high && !self.level_config.state.hovered,
            );
        }

        // Margin
        main.cut_left(layout_size * 0.5);

        // Groups and levels on the left
        let mut side = main;
        let groups = side.cut_left(context.font_size * 6.0);
        let _connections = side.cut_left(layout_size * 3.0);
        let levels = side.cut_left(context.font_size * 5.0);
        update!(self.groups_state, groups);
        update!(self.levels_state, levels);

        {
            // Level groups
            let slide = layout_size * 2.0;

            let scroll = 0.0; // TODO
            let group = Aabb2::point(groups.align_pos(vec2(0.0, 1.0)) + vec2(0.0, scroll))
                .extend_right(groups.width() - slide)
                .extend_down(2.0 * context.font_size);

            // Initialize missing groups
            for _ in 0..state.groups.len() - self.groups.len() {
                self.groups.push(GroupWidget::new());
            }

            // Layout each group
            let mut selected = None;
            for (static_pos, (i, entry)) in group
                .stack(
                    vec2(0.0, -group.height() - layout_size * 0.5),
                    state.groups.len(),
                )
                .into_iter()
                .zip(state.groups.iter().enumerate())
            {
                let Some(group) = self.groups.get_mut(i) else {
                    // should not happen
                    continue;
                };

                // Animate on hover
                let t = group.selected_time.get_ratio();
                let t = crate::util::smoothstep(t);
                let pos = static_pos.translate(vec2(t * slide, 0.0));

                update!(group, pos);
                group.set_group(entry);

                if group.state.clicked {
                    selected = Some(i);
                }

                let target = if state.switch_group == Some(i) {
                    1.0
                } else if group.state.hovered
                    || context.can_focus && static_pos.contains(context.cursor.position)
                {
                    0.5
                } else {
                    0.0
                };
                let delta = (target * group.selected_time.max() - group.selected_time.value())
                    .clamp_abs(context.delta_time);
                group.selected_time.change(delta);
            }

            // Show levels for the group
            if let Some(group) = selected {
                state.show_group(group);
            }
        }

        if let Some(show_group) = &state.show_group {
            if let Some(group) = state.groups.get(show_group.data) {
                // Levels
                let slide = layout_size * 2.0;

                let scroll = 0.0; // TODO

                // Animate slide-in/out
                let sign = if show_group.going_up { 1.0 } else { -1.0 };
                let t = 1.0 - crate::util::smoothstep(show_group.time.get_ratio());
                let scroll = scroll + sign * t * layout_size * 25.0;

                let level = Aabb2::point(levels.align_pos(vec2(0.0, 1.0)) + vec2(0.0, scroll))
                    .extend_right(levels.width() - slide)
                    .extend_down(2.0 * context.font_size);

                // Initialize missing levels
                for _ in 0..group.levels.len() - self.levels.len() {
                    self.levels.push(LevelWidget::new());
                }

                // Layout each level
                let mut selected = None;
                for (static_pos, (i, (_, level_meta))) in level
                    .stack(
                        vec2(0.0, -level.height() - layout_size * 0.5),
                        group.levels.len(),
                    )
                    .into_iter()
                    .zip(group.levels.iter().enumerate())
                {
                    let Some(level) = self.levels.get_mut(i) else {
                        // should not happen
                        continue;
                    };

                    // Animate
                    let t = level.selected_time.get_ratio();
                    let t = crate::util::smoothstep(t);
                    let pos = static_pos.translate(vec2(t * slide, 0.0));

                    update!(level, pos);
                    level.set_level(level_meta);

                    if level.state.clicked {
                        selected = Some(i);
                    }

                    let target = if state.switch_level == Some(i) {
                        1.0
                    } else if level.state.hovered
                        || context.can_focus && static_pos.contains(context.cursor.position)
                    {
                        0.5
                    } else {
                        0.0
                    };
                    let delta = (target * level.selected_time.max() - level.selected_time.value())
                        .clamp_abs(context.delta_time);
                    level.selected_time.change(delta);
                }

                // Show level
                if let Some(level) = selected {
                    if state.show_group.as_ref().is_some_and(|show| show.going_up) {
                        state.show_level(Some(level));
                    }
                }
            }
        }

        !context.can_focus
    }
}
