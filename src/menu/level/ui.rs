use super::*;

use crate::ui::{layout, widget::*};

pub struct MenuUI {
    pub ctl_logo: WidgetState,
    pub groups_state: WidgetState,
    pub groups: Vec<GroupWidget>,
    pub levels_state: WidgetState,
    pub levels: Vec<LevelWidget>,
    pub options_head: TextWidget,
    pub options: OptionsWidget,
    pub leaderboard: LeaderboardWidget,
    pub level_config: LevelConfigWidget,
}

impl MenuUI {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
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
        _geng: &Geng,
    ) -> bool {
        // Fix aspect
        let screen = layout::fit_aabb(vec2(16.0, 9.0), screen, vec2::splat(0.5));

        let layout_size = screen.height() * 0.03;

        let mut context = UiContext {
            font_size: screen.height() * 0.04,
            can_focus: true,
            cursor,
        };
        macro_rules! update {
            ($widget:expr, $position:expr) => {{
                $widget.update($position, &context);
            }};
        }

        // Margin
        let main = screen.extend_uniform(-layout_size * 2.0);

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

        {
            // Options
            let width = layout_size * 50.0;
            let height = layout_size * 20.0;

            let options = Aabb2::point(layout::aabb_pos(screen, vec2(0.5, 1.0)))
                .extend_symmetric(vec2(width, 0.0) / 2.0)
                .extend_up(height);

            let t = state.show_options.time.get_ratio().as_f32();
            let t = crate::util::smoothstep(t);
            let offset = -options.height() * t;

            let options = options.translate(vec2(0.0, offset));

            let head = Aabb2::point(screen.top_left() + vec2(10.0, 0.0) * layout_size)
                .extend_right(layout_size * 7.0)
                .extend_down(layout_size * 2.0)
                .translate(vec2(0.0, offset));

            update!(self.options_head, head);
            context.update_focus(self.options_head.state.hovered);

            self.options.set_options(state.options.clone());
            update!(self.options, options);
            context.update_focus(self.options.state.hovered);
            self.options.update_options(&mut state.options);

            if self.options_head.state.hovered && state.show_options.time.is_min() {
                state.options_request = Some(WidgetRequest::Open);
            } else if !self.options.state.hovered && !self.options_head.state.hovered {
                state.options_request = Some(WidgetRequest::Close);
            }
        }

        {
            // Leaderboard
            let width = layout_size * 20.0;
            let height = screen.height();

            let leaderboard =
                Aabb2::point(main.bottom_right() + vec2(0.0, 2.0) * base_t * layout_size)
                    .extend_left(width)
                    .extend_down(height);

            let t = state.show_leaderboard.time.get_ratio().as_f32();
            let t = crate::util::smoothstep(t);
            let offset = main.height() * t;

            let leaderboard = leaderboard.translate(vec2(0.0, offset));

            self.leaderboard.update_state(&state.show_leaderboard.data);
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
            if old_config != state.config && state.show_leaderboard.time.is_max() {
                state.leaderboard_request = Some(WidgetRequest::Reload);
            }

            if self.level_config.state.hovered && state.show_level_config.time.is_min() {
                state.config_request = Some(WidgetRequest::Open);
            } else if self.level_config.close.text.state.clicked {
                state.config_request = Some(WidgetRequest::Close);
            }
        }

        // Margin
        let main = main.extend_left(-layout_size * 2.0);

        // Groups and levels on the left
        let (groups, side) = layout::cut_left_right(main, layout_size * 13.0);
        let (_connections, side) = layout::cut_left_right(side, layout_size * 3.0);
        let (levels, _side) = layout::cut_left_right(side, layout_size * 9.0);
        update!(self.groups_state, groups);
        update!(self.levels_state, levels);

        {
            // Level groups
            let slide = layout_size * 2.0;

            let scroll = 0.0; // TODO
            let group = Aabb2::point(layout::aabb_pos(groups, vec2(0.0, 1.0)) + vec2(0.0, scroll))
                .extend_right(groups.width() - slide)
                .extend_down(3.0 * layout_size);

            // Initialize missing groups
            for _ in 0..state.groups.len() - self.groups.len() {
                self.groups.push(GroupWidget::new());
            }

            // Layout each group
            let mut hovered = None;
            for (pos, (i, entry)) in layout::stack(
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
                let pos = pos.translate(vec2(t * slide, 0.0));

                update!(group, pos);
                group.set_group(entry);

                if group.state.hovered {
                    hovered = Some(i);
                }
                if group.state.hovered || state.switch_group == Some(i) {
                    group.selected_time.change(delta_time);
                } else {
                    group.selected_time.change(-delta_time);
                }
            }

            // Show levels for the group
            if let Some(group) = hovered {
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
                        .extend_down(3.0 * layout_size);

                // Initialize missing levels
                for _ in 0..group.levels.len() - self.levels.len() {
                    self.levels.push(LevelWidget::new());
                }

                // Layout each level
                let mut hovered = None;
                for (pos, (i, (_, level_meta))) in layout::stack(
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

                    // Animate on hover
                    let t = level.selected_time.get_ratio();
                    let t = crate::util::smoothstep(t);
                    let pos = pos.translate(vec2(t * slide, 0.0));

                    update!(level, pos);
                    level.set_level(level_meta);

                    if level.state.hovered {
                        hovered = Some(i);
                    }
                    if level.state.hovered || state.switch_level == Some(i) {
                        level.selected_time.change(delta_time);
                    } else {
                        level.selected_time.change(-delta_time);
                    }
                }

                // Show level
                if let Some(level) = hovered {
                    state.show_level(Some(level));
                }
            }
        }

        !context.can_focus
    }
}
