use super::*;

/// Layout and state of the UI.
pub struct EditorUI {
    pub screen: Aabb2<f32>,
    pub game: Aabb2<f32>,
}

impl EditorUI {
    pub fn new() -> Self {
        let default_aabb = Aabb2::ZERO.extend_uniform(1.0);
        Self {
            screen: default_aabb,
            game: default_aabb,
        }
    }

    pub fn layout(&mut self, screen: Aabb2<f32>, _cursor_pos: vec2<f32>) {
        let screen = geng_utils::layout::fit_aabb(vec2(16.0, 9.0), screen, vec2::splat(0.5));
        self.screen = screen;

        {
            let max_size = screen.size(); // * 0.8;

            let ratio = 16.0 / 9.0;
            let max_height = max_size.y.min(max_size.x / ratio);

            let game_height = max_height;
            let game_size = vec2(game_height * ratio, game_height);

            self.game = geng_utils::layout::align_aabb(game_size, screen, vec2(0.0, 1.0));
        }
    }
}
