use super::*;

pub struct PanelsUI {
    pub options_head: TextWidget,
    pub options: OptionsWidget,
    pub explore_head: TextWidget,
    pub explore: ExploreWidget,
    pub profile_head: IconWidget,
    pub profile: ProfileWidget,
}

impl PanelsUI {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            options_head: TextWidget::new("Options"),
            options: OptionsWidget::new(
                Options::default(),
                vec![
                    // TODO: custom
                    PaletteWidget::new("Classic", Theme::classic()),
                    PaletteWidget::new("Test", Theme::test()),
                ],
            ),
            explore_head: TextWidget::new("Browse"),
            explore: ExploreWidget::new(assets),
            profile_head: IconWidget::new(&assets.sprites.head),
            profile: ProfileWidget::new(),
        }
    }

    pub fn update(&mut self, state: &mut MenuState, context: &mut UiContext) {
        let screen = context.screen;
        let layout_size = context.layout_size;

        let mut top_bar = screen.clone().cut_top(context.font_size * 1.2);
        top_bar.cut_right(context.layout_size * 7.0);

        let profile_head = top_bar.cut_right(context.font_size * 1.2);
        top_bar.cut_right(context.layout_size * 3.0);

        let explore_head = top_bar.cut_right(context.font_size * 3.5);
        top_bar.cut_right(context.layout_size * 3.0);

        let options_head = top_bar.cut_right(context.font_size * 3.5);

        let (options_head, options) = {
            // Options
            let width = layout_size * 50.0;
            let height = layout_size * 15.0;

            let options = Aabb2::point(screen.align_pos(vec2(0.5, 1.0)))
                .extend_symmetric(vec2(width, 0.0) / 2.0)
                .extend_up(height);

            let t = self.options.window.show.time.get_ratio();
            let t = crate::util::smoothstep(t);
            let offset = -options.height() * t;

            (
                options_head.translate(vec2(0.0, offset)),
                options.translate(vec2(0.0, offset)),
            )
        };

        let (explore_head, explore) = {
            // Explore
            let width = layout_size * 50.0;
            let height = layout_size * 20.0;

            let explore = Aabb2::point(screen.align_pos(vec2(0.5, 1.0)))
                .extend_symmetric(vec2(width, 0.0) / 2.0)
                .extend_up(height);

            let t = self.explore.window.show.time.get_ratio();
            let t = crate::util::smoothstep(t);
            let offset = -explore.height() * t;

            (
                explore_head.translate(vec2(0.0, offset)),
                explore.translate(vec2(0.0, offset)),
            )
        };

        let (profile_head, profile) = {
            // Profile
            let width = layout_size * 15.0;
            let height = layout_size * 17.0;

            let profile = Aabb2::point(profile_head.top_right())
                .extend_right(width * 0.1)
                .extend_left(width * 0.9)
                .extend_up(height);

            let t = self.profile.window.show.time.get_ratio();
            let t = crate::util::smoothstep(t);
            let offset = -profile.height() * t;

            (
                profile_head.translate(vec2(0.0, offset)),
                profile.translate(vec2(0.0, offset)),
            )
        };

        // Options
        let old_options = state.options.clone();
        self.options.update(options, context, &mut state.options);
        context.update_focus(self.options.state.hovered);
        if state.options != old_options {
            preferences::save(OPTIONS_STORAGE, &state.options);
        }

        self.options.window.layout(
            self.options_head.state.hovered,
            !self.options.state.hovered && !self.options_head.state.hovered,
        );

        // Explore
        self.explore
            .update(explore, context, &mut state.context.local.clone());
        context.update_focus(self.explore.state.hovered);

        self.explore.window.layout(
            self.explore_head.state.hovered,
            !self.explore.state.hovered && !self.explore_head.state.hovered,
        );

        // Profile
        self.profile
            .update(profile, context, &mut state.leaderboard);
        context.update_focus(self.profile.state.hovered);

        self.profile.window.layout(
            self.profile_head.state.hovered,
            !self.profile.state.hovered && !self.profile_head.state.hovered,
        );

        // Heads
        self.options_head.update(options_head, context);
        context.update_focus(self.options_head.state.hovered);

        self.explore_head.update(explore_head, context);
        context.update_focus(self.explore_head.state.hovered);

        self.profile_head.update(profile_head, context);
        context.update_focus(self.profile_head.state.hovered);
    }
}
