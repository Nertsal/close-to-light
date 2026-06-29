use super::*;

pub struct PracticeWidget {
    pub state: WidgetState,
    pub window: UiWindow<()>,

    pub title: TextWidget,
    pub close: IconButtonWidget,
    pub confirm: IconButtonWidget,

    pub preview: WidgetState,
    pub preview_time: TimeInterpolation,
    pub level_duration: Time,
    pub cached_level: Option<Rc<Level>>,
    pub rendered: Option<LevelState>,

    pub timeline: WidgetState,
    pub timeline_interactive: WidgetState,
    pub timeline_current_time: TextWidget,
    pub timeline_selected_text: TextWidget,
    pub timeline_start: WidgetState,
    pub timeline_end: WidgetState,
    pub timeline_current: WidgetState,
    pub timeline_from: WidgetState,
    pub timeline_to: WidgetState,
    pub select_from: Time,
    pub select_to: Time,
}

impl PracticeWidget {
    pub fn new(assets: &Assets) -> Self {
        Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),

            title: TextWidget::new("Practice Section"),
            close: IconButtonWidget::new_close_button(assets.atlas.button_close()),
            confirm: IconButtonWidget::new(
                assets.atlas.button_confirm(),
                ThemeColor::Highlight,
                IconBackgroundKind::Circle,
            ),

            preview: WidgetState::new(),
            preview_time: TimeInterpolation::new(5.0),
            level_duration: 0,
            cached_level: None,
            rendered: None,

            timeline: WidgetState::new(),
            timeline_interactive: WidgetState::new(),
            timeline_current_time: TextWidget::new("0:00"),
            timeline_selected_text: TextWidget::new("0:00 - 0:00"),
            timeline_start: WidgetState::new(),
            timeline_end: WidgetState::new(),
            timeline_current: WidgetState::new(),
            timeline_from: WidgetState::new(),
            timeline_to: WidgetState::new(),
            select_from: 0,
            select_to: 0,
        }
    }

    pub fn reload_level(&mut self, _music: &LocalMusic, level: &LevelFull) {
        // TODO: music waveform maybe
        self.cached_level = Some(level.data.clone());
        self.level_duration = level.data.last_time();
        self.preview_time.snap_to(0);
        self.select_from = 0;
        self.select_to = 0;
    }

    pub fn selected_range(&self) -> (Time, Time) {
        (
            std::cmp::min(self.select_from, self.select_to),
            std::cmp::max(self.select_from, self.select_to),
        )
    }

    pub fn update(&mut self, position: Aabb2<f32>, state: &mut MenuState, context: &UiContext) {
        let spacing = context.font_size * 0.2;
        let title_height = context.font_size * 1.2;
        let preview_res = crate::render::PREVIEW_RESOLUTION.as_f32();
        let preview_height = position.width() / preview_res.aspect();
        let timeline_height = context.font_size * 1.8;
        let timeline_space = context.font_size * 0.3;
        let position = position.with_height(
            spacing + title_height + preview_height + timeline_height + timeline_space,
            0.0,
        );

        self.window.update(context.delta_time);
        let t = 1.0 - self.window.show.time.get_ratio();
        let t = crate::util::smoothstep(t);
        let mut position = position.translate(vec2(0.0, context.screen.min.y - position.max.y) * t);
        self.state.update(position, context);

        position.cut_top(spacing);
        let title = position
            .cut_top(title_height)
            .extend_symmetric(-vec2(spacing, 0.0));
        self.title.update(title, context);
        self.close.update(
            title.align_aabb(vec2::splat(title.height()), vec2(0.0, 0.5)),
            context,
        );
        self.confirm
            .icon
            .state
            .set_visibility(self.select_to != self.select_from);
        self.confirm.update(
            title.align_aabb(vec2::splat(title.height()), vec2(1.0, 0.5)),
            context,
        );
        if self.close.icon.state.mouse_left.clicked {
            self.window.request = Some(WidgetRequest::Close);
        }
        if self.confirm.icon.state.mouse_left.clicked {
            state.practice_section = Some(self.selected_range());
        }

        // Selected area text
        self.timeline_selected_text.update(
            position.align_aabb(vec2(10.0, 0.8) * context.font_size, vec2(0.5, 1.0)),
            context,
        );
        self.timeline_selected_text.text = {
            let (from, to) = self.selected_range();
            format!(
                "{} - {}",
                ctl_util::display_time(from, false),
                ctl_util::display_time(to, false)
            )
            .into()
        };

        // Timeline
        position.cut_top(timeline_space);
        let timeline_pos = position.cut_top(timeline_height);
        self.timeline_interactive.update(timeline_pos, context);
        self.timeline.update(
            timeline_pos.extend_symmetric(-vec2(spacing * 2.0, 0.0)),
            context,
        );

        // Cursor time text
        self.timeline_current_time
            .state
            .set_visibility(self.timeline_interactive.hovered);
        self.timeline_current_time.update(
            timeline_pos.align_aabb(vec2(10.0, 0.8) * context.font_size, vec2(0.5, 0.0)),
            context,
        );
        self.timeline_current_time.text =
            ctl_util::display_time(self.preview_time.target, false).into();

        let tick = |time| {
            Aabb2::point(
                self.timeline
                    .position
                    .align_pos(vec2(time as f32 / self.level_duration as f32, 0.5)),
            )
            .extend_symmetric(
                vec2(
                    0.5 * context.font_size,
                    self.timeline_interactive.position.height(),
                ) / 2.0,
            )
        };
        self.timeline_start.update(tick(0), context);
        self.timeline_end.update(tick(self.level_duration), context);
        self.timeline_current
            .update(tick(self.preview_time.value), context);
        self.timeline_from.update(tick(self.select_from), context);
        self.timeline_to.update(tick(self.select_to), context);

        let local = &state.context.local;
        if let Some(show_group) = &state.selected_level
            && let Some(group) = local.get_group(show_group.data)
            && let Some(music) = &group.local.music
            && let Some(show_level) = &state.selected_diff
            && let Some(level) = local.get_level(show_group.data, show_level.data)
        {
            if self
                .cached_level
                .as_ref()
                .is_none_or(|cached| !Rc::ptr_eq(cached, &level.data))
            {
                self.reload_level(music, &level);
            }

            let t = (context.cursor.position.x - self.timeline.position.min.x)
                / self.timeline.position.width();
            let t = t.clamp(0.0, 1.0);
            let cursor_time = (self.level_duration as f32 * t) as Time;
            let cursor_time = level.data.timing.snap_to_beat(cursor_time, BeatTime::WHOLE);
            let mut preview_time = self.timeline_interactive.hovered;
            if self.timeline_to.mouse_left.pressed.is_some() {
                self.select_to = cursor_time;
                preview_time = true;
            } else if self.timeline_from.mouse_left.pressed.is_some() {
                self.select_from = cursor_time;
                preview_time = true;
            } else if self.timeline_interactive.mouse_left.just_pressed {
                self.select_from = cursor_time;
                self.select_to = cursor_time;
            } else if self.timeline_interactive.mouse_left.pressed.is_some() {
                self.select_to = cursor_time;
                preview_time = true;
            }
            if preview_time {
                self.preview_time.scroll_time(Change::Set(cursor_time));
            }

            if let Some(level) = &self.cached_level {
                let mut vfx = Vfx::new();
                self.rendered = Some(LevelState::render(
                    level,
                    self.preview_time.value,
                    None,
                    Some(&mut vfx),
                ));
            }
        }

        let preview_pos = position.cut_top(preview_height);
        self.preview.update(preview_pos, context);
        self.preview_time.update(r32(context.delta_time));

        context.update_focus(self.state.hovered);
    }
}
