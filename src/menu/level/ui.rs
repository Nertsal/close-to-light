use super::*;

use crate::ui::{layout, widget::*};

pub struct MenuUI {
    pub ctl_logo: WidgetState,
    pub groups_state: WidgetState,
    pub groups: Vec<GroupWidget>,
    pub level: PlayLevelWidget,
}

impl MenuUI {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            ctl_logo: default(),
            groups_state: default(),
            groups: Vec::new(),
            level: PlayLevelWidget::new(assets),
        }
    }

    pub fn layout(
        &mut self,
        state: &mut MenuState,
        screen: Aabb2<f32>,
        cursor_position: vec2<f32>,
        cursor_down: bool,
        delta_time: f32,
        _geng: &Geng,
    ) {
        // Fix aspect
        let screen = layout::fit_aabb(vec2(16.0, 9.0), screen, vec2::splat(0.5));

        let layout_size = screen.height() * 0.03;

        let context = UiContext {
            font_size: screen.height() * 0.04,
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

        // Margin
        let main = main.extend_left(-layout_size * 2.0);

        // Level groups on the left
        let (groups, side) = layout::cut_left_right(main, layout_size * 11.0);
        update!(self.groups_state, groups);

        {
            // Level groups
            let scroll = 0.0; // TODO
            let group = Aabb2::point(layout::aabb_pos(groups, vec2(0.0, 1.0)) + vec2(0.0, scroll))
                .extend_right(groups.width())
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
                let offset = layout_size * 2.0;
                let pos = pos.translate(vec2(t * offset, 0.0));

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

            // Show the pre-play menu for the hovered level group
            if let Some(group) = hovered {
                state.show_group(group);
            }
        }

        // Play level on the right
        let (_middle, level) = layout::cut_left_right(side, side.width() - layout_size * 30.0);
        if let Some(group) = &state.show_group {
            // Animate the slide-in
            let t = group.time.get_ratio().as_f32();
            let t = crate::util::smoothstep(t);
            let offscreen = screen.max.x - level.min.x;
            let target = offscreen * (1.0 - t);
            let level = level.translate(vec2(target, 0.0));
            update!(self.level, level);
            self.level.update_time(delta_time);

            if let Some(group) = state.groups.get(group.group) {
                self.level.set_group(group);
                self.level.show();

                // Play level
                if self.level.level_normal.text.state.clicked {
                    let level = LevelId {
                        group: group.path.clone(),
                        level: LevelVariation::Normal,
                    };
                    state.play_level(level, self.level.level_config.clone());
                } else if self.level.level_hard.text.state.clicked {
                    let level = LevelId {
                        group: group.path.clone(),
                        level: LevelVariation::Hard,
                    };
                    state.play_level(level, self.level.level_config.clone());
                }
            }
        } else {
            self.level.hide();
        }
    }
}
