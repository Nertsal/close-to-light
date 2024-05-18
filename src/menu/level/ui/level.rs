use super::*;

pub struct PlayLevelWidget {
    pub music: TextWidget,
    pub music_author: TextWidget,
    pub difficulty: TextWidget,
    pub mappers: TextWidget,
}

impl PlayLevelWidget {
    pub fn new() -> Self {
        Self {
            music: TextWidget::new(""),
            music_author: TextWidget::new("").aligned(vec2(1.0, 0.5)),
            difficulty: TextWidget::new(""),
            mappers: TextWidget::new("").aligned(vec2(1.0, 0.5)),
        }
    }

    pub fn update(&mut self, mut main: Aabb2<f32>, state: &mut MenuState, context: &mut UiContext) {
        // Base layout
        let music_pos = main.cut_top(context.font_size * 1.3);
        let music_author_pos = main.cut_top(context.font_size * 0.5);
        main.cut_top(context.layout_size * 1.0);
        let difficulty_pos = main.cut_top(context.font_size * 1.0);
        let mappers_pos = main.cut_top(context.font_size * 0.5);

        let font_factor = 1.3; // Scaling factor to fit better in the designated area

        // Sync data and dynamic layout
        let local = &state.context.local;
        if let Some(show) = &state.selected_music {
            if let Some(music) = local.get_music(show.data) {
                self.music.text = music.meta.name.clone();
                self.music_author.text = music.meta.authors().into();

                let t = crate::util::smoothstep(1.0 - show.time.get_ratio());
                let slide = vec2(context.screen.max.x - music_pos.min.x, 0.0) * t;

                self.music.update(music_pos.translate(slide), context);
                self.music.options.size = music_pos.height() * font_factor;
                self.music_author
                    .update(music_author_pos.translate(slide), context);
                self.music_author.options.size = music_author_pos.height() * font_factor;
            }
        }
        if let Some(group) = &state.selected_group {
            if let Some(show) = &state.selected_level {
                if let Some(level) = local.get_level(group.data, show.data) {
                    self.difficulty.text = level.meta.name.clone();
                    self.mappers.text = level.meta.authors().into();

                    let t = crate::util::smoothstep(1.0 - show.time.get_ratio());
                    let slide = vec2(context.screen.max.x - difficulty_pos.min.x, 0.0) * t;

                    self.difficulty
                        .update(difficulty_pos.translate(slide), context);
                    self.difficulty.options.size = difficulty_pos.height() * font_factor;
                    self.mappers.update(mappers_pos.translate(slide), context);
                    self.mappers.options.size = mappers_pos.height() * font_factor;
                }
            }
        }
    }
}
