use super::*;

use crate::ui::{layout, widget::*};

pub struct MenuUI {
    pub ctl_logo: WidgetState,
    pub groups_state: WidgetState,
    pub groups: Vec<GroupWidget>,
    pub level: PlayLevelWidget,
}

impl MenuUI {
    pub fn new() -> Self {
        Self {
            ctl_logo: default(),
            groups_state: default(),
            groups: Vec::new(),
            level: PlayLevelWidget::new(),
        }
    }

    pub fn layout(
        &mut self,
        state: &mut MenuState,
        screen: Aabb2<f32>,
        cursor_position: vec2<f32>,
        cursor_down: bool,
        _geng: &Geng,
    ) {
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

        let screen = screen.extend_uniform(-layout_size * 2.0);

        let (ctl_logo, main) = layout::cut_top_down(screen, layout_size * 4.0);
        update!(self.ctl_logo, ctl_logo);
        let main = main.extend_up(-layout_size * 3.0);

        let main = main.extend_left(-layout_size * 2.0);

        let (groups, side) = layout::cut_left_right(main, layout_size * 11.0);
        update!(self.groups_state, groups);

        {
            // Level groups
            let scroll = 0.0; // TODO
            let group = Aabb2::point(layout::aabb_pos(groups, vec2(0.0, 1.0)) + vec2(0.0, scroll))
                .extend_right(groups.width())
                .extend_down(3.0 * layout_size);

            for _ in 0..state.groups.len() - self.groups.len() {
                self.groups.push(GroupWidget::default());
            }

            let mut hovered = None;
            for (pos, (i, entry)) in
                layout::stack(group, vec2(0.0, 1.0) * group.size(), state.groups.len())
                    .into_iter()
                    .zip(state.groups.iter().enumerate())
            {
                let Some(group) = self.groups.get_mut(i) else {
                    // should not happen
                    continue;
                };
                update!(group, pos);
                group.set_group(entry);

                if group.state.hovered {
                    hovered = Some(i);
                }
            }

            if let Some(group) = hovered {
                state.show_group(group);
            }
        }

        let (_middle, level) = layout::cut_left_right(side, side.width() - layout_size * 30.0);
        {
            update!(self.level, level);
            if let Some(group) = state.show_group {
                if let Some(group) = state.groups.get(group) {
                    self.level.set_group(group);
                    self.level.show();

                    if self.level.level_normal.text.state.clicked {
                        // TODO: select level and options and stuff
                        let level = LevelId {
                            group: group.path.clone(),
                            level: LevelVariation::Normal,
                        };
                        state.play_level(level);
                    } else if self.level.level_hard.text.state.clicked {
                        let level = LevelId {
                            group: group.path.clone(),
                            level: LevelVariation::Hard,
                        };
                        state.play_level(level);
                    }
                }
            } else {
                self.level.hide();
            }
        }
    }
}
