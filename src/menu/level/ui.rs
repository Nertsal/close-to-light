use super::*;

use crate::ui::{layout, widget::*};

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
    pub profile: WidgetState,
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
            profile: WidgetState::new(),
            leaderboard: LeaderboardWidget::new(assets),
            level_config: LevelConfigWidget::new(assets),
        }
    }

    /// Layout all the ui elements and return whether any of them is focused.
    pub fn layout(
        &mut self,
        state: &mut MenuState,
        screen: Aabb2<f32>,
        cursor: CursorContext,
        delta_time: f32,
        geng: &Geng,
    ) -> bool {
        // Fix aspect
        let screen = layout::fit_aabb(vec2(16.0, 9.0), screen, vec2::splat(0.5));

        let layout_size = screen.height() * 0.03;

        let mut context = UiContext {
            theme: state.options.theme,
            layout_size,
            font_size: screen.height() * 0.06,
            can_focus: true,
            cursor,
            delta_time,
            mods: KeyModifiers::from_window(geng.window()),
        };
        macro_rules! update {
            ($widget:expr, $position:expr) => {{
                $widget.update($position, &mut context);
            }};
            ($widget:expr, $position:expr, $state:expr) => {{
                $widget.update($position, &mut context, $state);
            }};
        }

        update!(self.screen, screen);

        // Margin
        let main = screen
            .extend_uniform(-layout_size * 2.0)
            .extend_up(-layout_size * 2.0);

        // Logo
        let (ctl_logo, main) = layout::cut_top_down(main, layout_size * 4.0);
        update!(self.ctl_logo, ctl_logo);
        let main = main.extend_up(-layout_size * 3.0);

        let base_t = if state.level_up {
            1.0
        } else {
            state
                .show_level
                .as_ref()
                .map_or(0.0, |show| show.time.get_ratio().as_f32())
        };
        let base_t = crate::util::smoothstep(base_t) * 2.0 - 1.0;

        let (top_bar, _) = layout::cut_top_down(screen, context.font_size * 1.2);
        let top_bar = top_bar.extend_right(-context.layout_size * 7.0);

        let (top_bar, profile_head) =
            layout::cut_left_right(top_bar, top_bar.width() - context.font_size * 1.2);
        let top_bar = top_bar.extend_right(-context.layout_size * 3.0);

        let (_top_bar, options_head) =
            layout::cut_left_right(top_bar, top_bar.width() - context.font_size * 3.5);

        {
            // Options
            let width = layout_size * 50.0;
            let height = layout_size * 15.0;

            let options = Aabb2::point(layout::aabb_pos(screen, vec2(0.5, 1.0)))
                .extend_symmetric(vec2(width, 0.0) / 2.0)
                .extend_up(height);

            let t = state.show_options.time.get_ratio().as_f32();
            let t = crate::util::smoothstep(t);
            let offset = -options.height() * t;

            let options = options.translate(vec2(0.0, offset));

            let head = options_head.translate(vec2(0.0, offset));

            update!(self.options_head, head);
            context.update_focus(self.options_head.state.hovered);

            let old_options = state.options.clone();
            update!(self.options, options, &mut state.options);
            context.update_focus(self.options.state.hovered);
            if state.options != old_options {
                preferences::save(OPTIONS_STORAGE, &state.options);
            }

            if self.options_head.state.hovered && state.show_options.time.is_min() {
                state.options_request = Some(WidgetRequest::Open);
            } else if !self.options.state.hovered && !self.options_head.state.hovered {
                state.options_request = Some(WidgetRequest::Close);
            }
        }

        {
            // Profile
            let width = layout_size * 50.0;
            let height = layout_size * 15.0;

            let profile = Aabb2::point(layout::aabb_pos(screen, vec2(0.5, 1.0)))
                .extend_symmetric(vec2(width, 0.0) / 2.0)
                .extend_up(height);

            let t = state.show_profile.time.get_ratio().as_f32();
            let t = crate::util::smoothstep(t);
            let offset = -profile.height() * t;

            let profile = profile.translate(vec2(0.0, offset));

            let head = profile_head.translate(vec2(0.0, offset));

            update!(self.profile_head, head);
            context.update_focus(self.profile_head.state.hovered);

            // let old_profile = state.profile.clone();
            update!(self.profile, profile); //, &mut state.profile);
            context.update_focus(self.profile.hovered);
            // if state.profile != old_profile {
            //     preferences::save(profile_STORAGE, &state.profile);
            // }

            if self.profile_head.state.hovered && state.show_profile.time.is_min() {
                state.profile_request = Some(WidgetRequest::Open);
            } else if !self.profile.hovered && !self.profile_head.state.hovered {
                state.profile_request = Some(WidgetRequest::Close);
            }
        }

        {
            // Leaderboard
            let width = layout_size * 22.0;
            let height = main.height() + layout_size * 2.0;

            let leaderboard =
                Aabb2::point(main.bottom_right() + vec2(0.0, 2.0) * base_t * layout_size)
                    .extend_left(width)
                    .extend_down(height);

            let t = state.show_leaderboard.time.get_ratio().as_f32();
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

            if self.leaderboard.state.hovered && state.show_leaderboard.time.is_min() {
                state.leaderboard_request = Some(WidgetRequest::Open);
            } else if self.leaderboard.close.text.state.clicked {
                state.leaderboard_request = Some(WidgetRequest::Close);
            }
        }

        {
            // Mods
            let width = layout_size * 30.0;
            let height = layout_size * 20.0;

            let t = state.show_level_config.time.get_ratio().as_f32();
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
            if old_config != state.config && state.show_leaderboard.going_up {
                state.leaderboard_request = Some(WidgetRequest::Reload);
            }

            if self.level_config.state.hovered && state.show_level_config.time.is_min() {
                state.config_request = Some(WidgetRequest::Open);
            } else if self.level_config.close.text.state.clicked {
                state.config_request = Some(WidgetRequest::Close);
            }
        }

        // Margin
        let main = main.extend_left(-layout_size * 0.5);

        // Groups and levels on the left
        let (groups, side) = layout::cut_left_right(main, context.font_size * 6.0);
        let (_connections, side) = layout::cut_left_right(side, layout_size * 3.0);
        let (levels, _side) = layout::cut_left_right(side, context.font_size * 5.0);
        update!(self.groups_state, groups);
        update!(self.levels_state, levels);

        {
            // Level groups
            let slide = layout_size * 2.0;

            let scroll = 0.0; // TODO
            let group = Aabb2::point(layout::aabb_pos(groups, vec2(0.0, 1.0)) + vec2(0.0, scroll))
                .extend_right(groups.width() - slide)
                .extend_down(2.0 * context.font_size);

            // Initialize missing groups
            for _ in 0..state.groups.len() - self.groups.len() {
                self.groups.push(GroupWidget::new());
            }

            // Layout each group
            let mut selected = None;
            for (static_pos, (i, entry)) in layout::stack(
                group,
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
                    .clamp_abs(delta_time);
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
                let t = 1.0 - crate::util::smoothstep(show_group.time.get_ratio().as_f32());
                let scroll = scroll + sign * t * layout_size * 25.0;

                let level =
                    Aabb2::point(layout::aabb_pos(levels, vec2(0.0, 1.0)) + vec2(0.0, scroll))
                        .extend_right(levels.width() - slide)
                        .extend_down(2.0 * context.font_size);

                // Initialize missing levels
                for _ in 0..group.levels.len() - self.levels.len() {
                    self.levels.push(LevelWidget::new());
                }

                // Layout each level
                let mut selected = None;
                for (static_pos, (i, (_, level_meta))) in layout::stack(
                    level,
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
                        .clamp_abs(delta_time);
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
