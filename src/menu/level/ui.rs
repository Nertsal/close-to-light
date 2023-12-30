use super::*;

use crate::ui::{layout, widget::*};

pub struct MenuUI {
    pub ctl_logo: WidgetState,
    pub groups_state: WidgetState,
    pub groups: Vec<GroupWidget>,
    pub levels_state: WidgetState,
    pub levels: Vec<LevelWidget>,
    // pub play_group: LevelGroupWidget,
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
            // play_group: LevelGroupWidget::new(assets),
            leaderboard: LeaderboardWidget::new(),
            level_config: LevelConfigWidget::new(),
        }
    }

    /// Layout all the ui elements and return whether any of them is focused.
    pub fn layout(
        &mut self,
        state: &mut MenuState,
        screen: Aabb2<f32>,
        cursor_position: vec2<f32>,
        cursor_down: bool,
        delta_time: f32,
        _geng: &Geng,
    ) -> bool {
        // Fix aspect
        let screen = layout::fit_aabb(vec2(16.0, 9.0), screen, vec2::splat(0.5));

        let layout_size = screen.height() * 0.03;

        let mut context = UiContext {
            font_size: screen.height() * 0.04,
            can_focus: true,
            cursor_position,
            cursor_down,
        };
        macro_rules! update {
            ($widget:expr, $position:expr) => {{
                $widget.update($position, &context);
            }};
        }

        // Margin
        let screen = screen.extend_uniform(-layout_size * 2.0);

        // Logo
        let (ctl_logo, main) = layout::cut_top_down(screen, layout_size * 4.0);
        update!(self.ctl_logo, ctl_logo);
        let main = main.extend_up(-layout_size * 3.0);

        {
            // Leaderboard
            let width = layout_size * 20.0;
            let height = screen.height();
            let t = state.show_leaderboard.time.get_ratio().as_f32();
            let t = crate::util::smoothstep(t);
            let offset = main.height() * t;
            let leaderboard = Aabb2::point(main.bottom_right() + vec2(0.0, 2.0) * layout_size)
                .extend_left(width)
                .extend_down(height)
                .translate(vec2(0.0, offset));

            self.leaderboard.update_state(&state.show_leaderboard.data);
            update!(self.leaderboard, leaderboard);
            context.update_focus(self.leaderboard.state.hovered);

            if self.leaderboard.state.hovered && state.show_leaderboard.time.is_min() {
                state.leaderboard_request = Some(WidgetRequest::Open);
            }
        }

        {
            // Mods
            let width = layout_size * 30.0;
            let height = layout_size * 20.0;
            let t = state.show_level_config.time.get_ratio().as_f32();
            let t = crate::util::smoothstep(t);
            let offset = height * t;
            let config = Aabb2::point(main.bottom_left() + vec2(0.0, 2.0) * layout_size)
                .extend_right(width)
                .extend_down(height)
                .translate(vec2(0.0, offset));

            update!(self.level_config, config);
            context.update_focus(self.level_config.state.hovered);

            if self.level_config.state.hovered && state.show_level_config.time.is_min() {
                state.config_request = Some(WidgetRequest::Open);
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

        // // Play level on the right
        // let (_middle, level) = layout::cut_left_right(side, side.width() - layout_size * 30.0);
        // if let Some(group) = &state.show_group {
        //     // Animate the slide-in
        //     let t = group.time.get_ratio().as_f32();
        //     let t = crate::util::smoothstep(t);
        //     let offscreen = screen.max.x - level.min.x;
        //     let target = offscreen * (1.0 - t);
        //     let level = level.translate(vec2(target, 0.0));
        //     update!(self.play_group, level);
        //     self.play_group.update_time(delta_time);

        //     if let Some(group) = state.groups.get(group.group) {
        //         self.play_group.set_group(group);
        //         self.play_group.show();

        //         // Play level
        //         let mut play = None;
        //         for ((level_path, _), level) in group.levels.iter().zip(&self.play_group.levels) {
        //             if level.play.text.state.clicked {
        //                 play = Some(level_path.clone());
        //             }
        //         }
        //         if let Some(level_path) = play {
        //             state.play_level(level_path, self.play_group.level_config.clone());
        //         }
        //     }
        // } else {
        //     self.play_group.hide();
        // }

        !context.can_focus
    }
}
