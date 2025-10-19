use super::*;

pub struct PlayLevelWidget {
    pub music: TextWidget,
    pub music_author: TextWidget,
    pub music_original: TextWidget,
    pub difficulty: TextWidget,
    pub mappers: TextWidget,
}

impl PlayLevelWidget {
    pub fn new() -> Self {
        let mut widget = Self {
            music: TextWidget::new(""),
            music_author: TextWidget::new("").aligned(vec2(1.0, 0.5)),
            music_original: TextWidget::new("original"),
            difficulty: TextWidget::new(""),
            mappers: TextWidget::new("").aligned(vec2(1.0, 0.5)),
        };
        widget.music_original.hide();
        widget
    }

    pub fn update(&mut self, mut main: Aabb2<f32>, state: &mut MenuState, context: &mut UiContext) {
        // Base layout
        let music_pos = main.cut_top(context.font_size * 1.3);
        let mut music_author_pos = main.cut_top(context.font_size * 0.5);
        let music_original = music_author_pos.cut_left(context.font_size * 3.0);
        main.cut_top(context.layout_size * 1.0);
        let difficulty_pos = main.cut_top(context.font_size * 1.0);
        let mappers_pos = main.cut_top(context.font_size * 0.5);

        let font_factor = 1.3; // Scaling factor to fit better in the designated area

        // Sync data and dynamic layout
        let local = &state.context.local;
        let mut music_t = 1.0;
        let mut level_t = 1.0;
        if let Some(show_group) = &state.selected_level
            && let Some(group) = local.get_group(show_group.data)
        {
            // Music
            if let Some(music) = &group.local.music {
                self.music.text = music.meta.name.clone();
                self.music_author.text = author_text("music", music.meta.authors()).into();
                if music.meta.original {
                    self.music_original.show();
                } else {
                    self.music_original.hide();
                }
                music_t = crate::util::smoothstep(1.0 - show_group.time.get_ratio());
            }

            // Difficulty
            if let Some(show_level) = &state.selected_diff
                && let Some(level) = local.get_level(show_group.data, show_level.data)
            {
                self.difficulty.text = level.meta.name.clone();
                self.mappers.text = author_text("mapped", level.meta.authors()).into();
                level_t = crate::util::smoothstep(1.0 - show_level.time.get_ratio());
            }
        }

        // Music
        let slide_off = context.font_size * 2.0;
        let slide = vec2(context.screen.max.x + slide_off - music_pos.min.x, 0.0) * music_t;
        self.music.update(music_pos.translate(slide), context);
        self.music.options.size = music_pos.height() * font_factor;
        self.music_original
            .update(music_original.translate(slide), context);
        self.music_original.options.size = music_original.height() * font_factor;
        self.music_author
            .update(music_author_pos.translate(slide), context);
        self.music_author.options.size = music_author_pos.height() * font_factor;

        // Difficulty
        let slide = vec2(context.screen.max.x + slide_off - difficulty_pos.min.x, 0.0) * level_t;
        self.difficulty
            .update(difficulty_pos.translate(slide), context);
        self.difficulty.options.size = difficulty_pos.height() * font_factor;
        self.mappers.update(mappers_pos.translate(slide), context);
        self.mappers.options.size = mappers_pos.height() * font_factor;
    }
}

fn author_text(prefix: impl AsRef<str>, authors: impl AsRef<str>) -> String {
    let prefix = prefix.as_ref();
    let authors = authors.as_ref();
    if authors.is_empty() {
        format!("{prefix} by <authors unspecified>")
    } else {
        format!("{prefix} by {authors}")
    }
}
