use super::*;

pub struct LevelSelectUI {
    geng: Geng,
    assets: Rc<Assets>,

    pub tab_music: ToggleWidget,
    pub tab_groups: ToggleWidget,
    pub tab_levels: ToggleWidget,

    pub grid_music: Vec<ItemWidget<Id>>,
    pub grid_groups: Vec<ItemWidget<Id>>,
    pub grid_levels: Vec<ItemWidget<usize>>,

    pub groups_state: WidgetState,
    pub groups: Vec<GroupWidget>,
    pub new_group: TextWidget,

    pub levels_state: WidgetState,
    pub levels: Vec<LevelWidget>,
    pub new_level: TextWidget,
}

impl LevelSelectUI {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        let mut ui = Self {
            geng: geng.clone(),
            assets: assets.clone(),

            tab_music: ToggleWidget::new("Music"),
            tab_groups: ToggleWidget::new("Group"),
            tab_levels: ToggleWidget::new("Difficulty"),

            grid_music: Vec::new(),
            grid_groups: Vec::new(),
            grid_levels: Vec::new(),

            groups_state: default(),
            groups: Vec::new(),
            new_group: TextWidget::new("+ New Level Set"),

            levels_state: default(),
            levels: Vec::new(),
            new_level: TextWidget::new("+ New Difficulty"),
        };
        ui.tab_music.selected = true;
        ui.tab_groups.hide();
        ui.tab_levels.hide();
        ui
    }

    pub fn update(
        &mut self,
        main: Aabb2<f32>,
        state: &mut MenuState,
        context: &mut UiContext,
    ) -> Option<SyncWidget> {
        let layout_size = context.layout_size;

        {
            let mut main = main;
            main.cut_top(context.layout_size * 1.5);
            let bar = main.cut_top(context.font_size * 1.2);
            main.cut_top(context.layout_size * 1.0);

            let buttons: Vec<_> = [
                Some(&mut self.tab_music),
                self.tab_groups
                    .text
                    .state
                    .visible
                    .then_some(&mut self.tab_groups),
                self.tab_levels
                    .text
                    .state
                    .visible
                    .then_some(&mut self.tab_levels),
            ]
            .into_iter()
            .flatten()
            .collect();

            let button_size = vec2(7.0 * context.layout_size, bar.height());
            let button = Aabb2::point(bar.center()).extend_symmetric(button_size / 2.0);
            let buttons_layout = button.stack_aligned(button_size, buttons.len(), vec2(0.5, 0.5));
            let mut deselect = false;
            for (button, pos) in buttons.into_iter().zip(buttons_layout) {
                button.update(pos, context);
                if button.text.state.clicked {
                    deselect = true;
                }
            }
            if deselect {
                for button in [
                    &mut self.tab_music,
                    &mut self.tab_groups,
                    &mut self.tab_levels,
                ] {
                    if !button.text.state.clicked {
                        button.selected = false;
                    }
                }
            }

            // Music grid
            let local = state.local.borrow();
            let music: Vec<_> = local.music.iter().sorted_by_key(|(&k, _)| k).collect();

            // Synchronize vec length
            if self.grid_music.len() > music.len() {
                self.grid_music.drain(music.len()..);
            } else {
                for _ in 0..music.len().saturating_sub(self.grid_music.len()) {
                    self.grid_music.push(ItemWidget::new("", 0));
                }
            }

            // Synchronize data
            for (widget, (&music_id, cache)) in self.grid_music.iter_mut().zip(&music) {
                widget.data = music_id;
                widget.text.text = cache.meta.name.clone();
            }

            // Layout
            let columns = 3;
            let rows = self.grid_music.len() / columns + 1;
            let spacing = vec2(1.0, 2.0) * context.layout_size;
            let item_size = vec2(
                (main.width() - spacing.x * (columns as f32 - 1.0)) / columns as f32,
                1.3 * context.font_size,
            );
            for row in 0..rows {
                let top_left =
                    Aabb2::point(main.top_left() - vec2(0.0, item_size.y + spacing.y) * row as f32)
                        .extend_right(item_size.x)
                        .extend_down(item_size.y);
                let layout = top_left.stack(vec2(item_size.x + spacing.x, 0.0), columns);
                let i = columns * row;
                let range = (i + 3).min(self.grid_music.len());
                for (widget, pos) in self.grid_music[i..range].iter_mut().zip(layout) {
                    widget.update(pos, context);
                }
            }
        }

        let mut sync = None;

        // Groups and levels on the left
        let mut side = main;
        let groups = side.cut_left(context.font_size * 6.0);
        let _connections = side.cut_left(layout_size * 3.0);
        let levels = side.cut_left(context.font_size * 5.0);
        self.groups_state.update(groups, context);
        self.levels_state.update(levels, context);

        let group_ids: Vec<Index> = state
            .local
            .borrow()
            .groups
            .iter()
            .map(|(i, _)| i)
            .sorted()
            .collect();

        {
            let mut local = state.local.borrow_mut();

            // Level groups
            let slide = layout_size * 2.0;

            let scroll = 0.0; // TODO
            let group = Aabb2::point(groups.align_pos(vec2(0.0, 1.0)) + vec2(0.0, scroll))
                .extend_right(groups.width() - slide)
                .extend_down(2.0 * context.font_size);

            // Initialize missing groups
            for _ in 0..local.groups.len().saturating_sub(self.groups.len()) {
                self.groups.push(GroupWidget::new(&self.assets));
            }

            // Layout each group
            let mut selected = None;
            let positions = group.stack(
                vec2(0.0, -group.height() - layout_size * 0.5),
                local.groups.len() + 1,
            );
            for (&static_pos, (i, &index)) in positions.iter().zip(group_ids.iter().enumerate()) {
                let Some(group) = self.groups.get_mut(i) else {
                    // should not happen
                    continue;
                };

                // Animate on hover
                let t = group.selected_time.get_ratio();
                let t = crate::util::smoothstep(t);
                let pos = static_pos.translate(vec2(t * slide, 0.0));

                group.static_state.update(static_pos, context);
                group.update(pos, context, &mut local);
                if let Some(entry) = local.groups.get(index) {
                    group.set_group(entry, index);
                }

                if group.state.clicked {
                    selected = Some(i);
                }

                let target = if state.switch_group == Some(i) {
                    1.0
                } else if group.state.hovered
                    || context.can_focus && static_pos.contains(context.cursor.position)
                {
                    0.5
                } else {
                    0.0
                };
                let delta = (target * group.selected_time.max() - group.selected_time.value())
                    .clamp_abs(context.delta_time);
                group.selected_time.change(delta);
            }

            // Show levels for the group
            drop(local);
            if let Some(group) = selected {
                state.show_group(group);
            }

            let create = positions
                .last()
                .unwrap()
                .extend_symmetric(-vec2(0.1, 0.7) * layout_size);
            self.new_group.update(create, context);
            if self.new_group.state.clicked {
                state.new_group();
            }
        }

        if let Some(show_group) = &state.show_group {
            enum Action {
                Sync(Rc<CachedLevel>, usize),
                Edit(usize),
                Show(usize),
                New,
            }
            let mut action = None;

            let local = state.local.borrow();

            let group_index = group_ids.get(show_group.data);
            let group = group_index.and_then(|&group_index| local.groups.get(group_index));
            if group.is_none() {
                // Group got deleted
                state.switch_group = None;
            }

            // Levels
            let levels_count = group.map(|group| group.levels.len()).unwrap_or(0);
            let slide = layout_size * 2.0;

            let scroll = 0.0; // TODO

            // Animate slide-in/out
            let sign = if show_group.going_up { 1.0 } else { -1.0 };
            let t = 1.0 - crate::util::smoothstep(show_group.time.get_ratio());
            let scroll = scroll + sign * t * layout_size * 25.0;

            let level = Aabb2::point(levels.align_pos(vec2(0.0, 1.0)) + vec2(0.0, scroll))
                .extend_right(levels.width() - slide)
                .extend_down(2.0 * context.font_size);

            // Initialize missing levels
            for _ in 0..levels_count.saturating_sub(self.levels.len()) {
                self.levels.push(LevelWidget::new(&self.assets));
            }
            if levels_count < self.levels.len() {
                // Delete extra levels
                self.levels.drain(levels_count..);
            }

            // Layout each level
            let mut selected = None;
            let positions = level.stack(
                vec2(0.0, -level.height() - layout_size * 0.5),
                self.levels.len() + 1,
            );
            for (&static_pos, (i, level)) in
                positions.iter().zip(self.levels.iter_mut().enumerate())
            {
                // Animate
                let t = level.selected_time.get_ratio();
                let t = crate::util::smoothstep(t);
                let pos = static_pos.translate(vec2(t * slide, 0.0));

                level.static_state.update(static_pos, context);
                level.update(pos, context);
                if let Some(cached) = group.and_then(|group| group.levels.get(i)) {
                    level.set_level(&cached.meta);
                }

                if level.state.clicked {
                    selected = Some(i);
                }

                let target = if state.switch_level == Some(i) {
                    1.0
                } else if level.state.hovered
                    || context.can_focus && static_pos.contains(context.cursor.position)
                {
                    0.5
                } else {
                    0.0
                };
                let delta = (target * level.selected_time.max() - level.selected_time.value())
                    .clamp_abs(context.delta_time);
                level.selected_time.change(delta);

                // Buttons
                if level.sync.state.clicked {
                    if let Some(cached) = group.and_then(|group| group.levels.get(i)) {
                        action = Some(Action::Sync(cached.clone(), i));
                    }
                } else if level.edit.state.clicked {
                    action = Some(Action::Edit(i));
                }
            }

            // Show level
            if let Some(level) = selected {
                if state.show_group.as_ref().is_some_and(|show| show.going_up) {
                    action = Some(Action::Show(level));
                }
            }

            let create = positions
                .last()
                .unwrap()
                .extend_symmetric(vec2(1.0, -0.7) * layout_size);
            self.new_level.update(create, context);
            if self.new_level.state.clicked {
                action = Some(Action::New);
            }

            if let Some(&group_index) = group_index {
                if let Some(action) = action {
                    match action {
                        Action::Sync(level, level_index) => {
                            if let Some(group) = group {
                                sync = Some(SyncWidget::new(
                                    &self.geng,
                                    &self.assets,
                                    group,
                                    group_index,
                                    &level,
                                    level_index,
                                ));
                            }
                        }
                        Action::Edit(level) => {
                            drop(local);
                            state.edit_level(group_index, level);
                        }
                        Action::Show(level) => {
                            drop(local);
                            state.show_level(Some(level));
                        }
                        Action::New => {
                            drop(local);
                            state.new_level(group_index);
                        }
                    }
                }
            }
        }

        sync
    }
}

pub struct ItemWidget<T> {
    pub state: WidgetState,
    pub text: TextWidget,
    pub data: T,
}

impl<T> ItemWidget<T> {
    pub fn new(text: impl Into<String>, data: T) -> Self {
        Self {
            state: WidgetState::new(),
            text: TextWidget::new(text).aligned(vec2(0.5, 0.5)),
            data,
        }
    }
}

impl<T> Widget for ItemWidget<T> {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.text.update(position, &mut context.scale_font(0.9));
    }
}
