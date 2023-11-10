use super::*;

use crate::ui::{layout, widget::*};

pub struct MenuUI {
    pub ctl_logo: WidgetState,
    pub groups_state: WidgetState,
    pub groups: Vec<GroupWidget>,
}

impl MenuUI {
    pub fn new() -> Self {
        Self {
            ctl_logo: default(),
            groups_state: default(),
            groups: Vec::new(),
        }
    }

    pub fn layout(
        &mut self,
        group_entries: &Vec<GroupEntry>,
        screen: Aabb2<f32>,
        cursor_position: vec2<f32>,
        cursor_down: bool,
        geng: &Geng,
    ) {
        let screen = layout::fit_aabb(vec2(16.0, 9.0), screen, vec2::splat(0.5));

        let font_size = screen.height() * 0.03;

        let context = UiContext {
            font_size,
            cursor_position,
            cursor_down,
        };
        macro_rules! update {
            ($widget:expr, $position:expr) => {{
                $widget.update($position, &context);
            }};
        }

        let screen = screen.extend_uniform(-font_size * 2.0);

        let (ctl_logo, main) = layout::cut_top_down(screen, font_size * 4.0);
        update!(self.ctl_logo, ctl_logo);
        let main = main.extend_up(-font_size * 3.0);

        let main = main.extend_left(-font_size * 2.0);

        let (groups, side) = layout::cut_left_right(main, font_size * 11.0);
        update!(self.groups_state, groups);

        {
            // Level groups
            let scroll = 0.0; // TODO
            let group = Aabb2::point(layout::aabb_pos(groups, vec2(0.0, 1.0)) + vec2(0.0, scroll))
                .extend_right(groups.width())
                .extend_down(3.0 * font_size);

            for _ in 0..group_entries.len() - self.groups.len() {
                self.groups.push(GroupWidget::default());
            }
            for (pos, (i, entry)) in
                layout::stack(group, vec2(0.0, 1.0) * group.size(), group_entries.len())
                    .into_iter()
                    .zip(group_entries.iter().enumerate())
            {
                let Some(group) = self.groups.get_mut(i) else {
                    // should not happen
                    continue;
                };
                update!(group, pos);
                group.name.text = entry.meta.name.to_string();
                // group.author = entry.meta..to_string();
            }
        }
    }
}
