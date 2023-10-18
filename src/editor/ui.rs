use super::*;

/// Layout and state of the UI.
pub struct EditorUI {
    pub screen: Aabb2<f32>,
    pub game: Aabb2<f32>,
    pub level_info: Aabb2<f32>,
    pub general: Aabb2<f32>,
    pub selected: Aabb2<f32>,
}

impl EditorUI {
    pub fn new() -> Self {
        let default_aabb = Aabb2::ZERO.extend_uniform(1.0);
        Self {
            screen: default_aabb,
            game: default_aabb,
            level_info: default_aabb,
            general: default_aabb,
            selected: default_aabb,
        }
    }

    pub fn layout(&mut self, screen: Aabb2<f32>, _cursor_pos: vec2<f32>) {
        let screen = geng_utils::layout::fit_aabb(vec2(16.0, 9.0), screen, vec2::splat(0.5));
        self.screen = screen;

        {
            let max_size = screen.size() * 0.8;

            let ratio = 16.0 / 9.0;
            let max_height = max_size.y.min(max_size.x / ratio);

            let game_height = max_height;
            let game_size = vec2(game_height * ratio, game_height);

            self.game = geng_utils::layout::align_aabb(game_size, screen, vec2(0.0, 1.0));
        }

        let margin = screen.width().min(screen.height()) * 0.02;

        let side_bar = Aabb2 {
            min: vec2(self.game.max.x, screen.min.y),
            max: self.screen.max,
        }
        .extend_uniform(-margin);
        let _bottom_bar = Aabb2 {
            min: self.screen.min,
            max: vec2(self.game.max.x, self.game.min.y),
        }
        .extend_uniform(-margin);

        {
            let info_size = side_bar.size() * vec2(1.0, 0.2);
            self.level_info = geng_utils::layout::align_aabb(info_size, side_bar, vec2(0.5, 0.0));
        }

        {
            let general_size = side_bar.size() * vec2(1.0, 0.3);
            self.general = geng_utils::layout::align_aabb(
                general_size,
                side_bar.extend_down(-self.level_info.height()),
                vec2(0.5, 0.0),
            );
        }

        {
            let select_size = side_bar.size() * vec2(1.0, 0.45);
            self.selected = geng_utils::layout::align_aabb(select_size, side_bar, vec2(0.5, 1.0));
        }
    }
}
