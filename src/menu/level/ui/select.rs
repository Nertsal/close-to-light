use super::*;

pub struct LevelSelectUI {
    // geng: Geng,
    // assets: Rc<Assets>,
    pub tab_music: ToggleWidget,
    pub tab_groups: ToggleWidget,
    pub tab_levels: ToggleWidget,
    pub separator: WidgetState,

    pub grid_music: Vec<ItemWidget<Id>>,
    pub grid_groups: Vec<ItemWidget<Index>>,
    pub grid_levels: Vec<ItemWidget<usize>>,
}

#[derive(Debug, Clone, Copy)]
pub enum LevelSelectTab {
    Music,
    Group,
    Difficulty,
}

impl LevelSelectUI {
    pub fn new(_geng: &Geng, _assets: &Rc<Assets>) -> Self {
        let mut ui = Self {
            // geng: geng.clone(),
            // assets: assets.clone(),
            tab_music: ToggleWidget::new("Music"),
            tab_groups: ToggleWidget::new("Group"),
            tab_levels: ToggleWidget::new("Difficulty"),
            separator: WidgetState::new(),

            grid_music: Vec::new(),
            grid_groups: Vec::new(),
            grid_levels: Vec::new(),
        };
        ui.tab_music.selected = true;
        ui.tab_groups.hide();
        ui.tab_levels.hide();
        ui
    }

    pub fn select_tab(&mut self, tab: LevelSelectTab) {
        for button in [
            &mut self.tab_music,
            &mut self.tab_groups,
            &mut self.tab_levels,
        ] {
            if !button.text.state.clicked {
                button.selected = false;
            }
        }

        let tab = match tab {
            LevelSelectTab::Music => &mut self.tab_music,
            LevelSelectTab::Group => &mut self.tab_groups,
            LevelSelectTab::Difficulty => &mut self.tab_levels,
        };
        tab.show();
        tab.selected = true;
    }

    pub fn update(
        &mut self,
        main: Aabb2<f32>,
        state: &mut MenuState,
        context: &mut UiContext,
    ) -> Option<SyncWidget> {
        let mut main = main;
        main.cut_top(context.layout_size * 1.5);
        let bar = main.cut_top(context.font_size * 1.2);
        main.cut_top(context.layout_size * 1.0);

        self.tabs(bar, context);

        if self.tab_music.selected {
            self.grid_music(main, state, context);
        } else if self.tab_groups.selected {
            self.grid_groups(main, state, context);
        } else if self.tab_levels.selected {
            self.grid_levels(main, state, context);
        }

        // TODO sync
        None
    }

    fn tabs(&mut self, main: Aabb2<f32>, context: &mut UiContext) {
        let sep_size = vec2(main.width() * 0.9, 0.3 * context.layout_size);
        let sep = main.align_aabb(sep_size, vec2(0.5, 0.0));
        self.separator.update(sep, context);

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

        let spacing = 1.0 * context.layout_size;
        let button_size = vec2(7.0 * context.layout_size, main.height());
        let button = Aabb2::point(main.center()).extend_symmetric(button_size / 2.0);

        let all_buttons = 3;
        let buttons_layout = button.stack_aligned(
            vec2(button_size.x + spacing, 0.0),
            all_buttons,
            vec2(0.5, 0.5),
        );

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
    }

    fn grid_music(&mut self, main: Aabb2<f32>, state: &mut MenuState, context: &mut UiContext) {
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

        drop(local);

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
            let mut tab = None;
            for (widget, pos) in self.grid_music[i..range].iter_mut().zip(layout) {
                widget.update(pos, context);
                if widget.state.clicked {
                    state.select_music(widget.data);
                    tab = Some(LevelSelectTab::Group);
                }
            }
            if let Some(tab) = tab {
                self.select_tab(tab);
            }
        }
    }

    fn grid_groups(&mut self, main: Aabb2<f32>, state: &mut MenuState, context: &mut UiContext) {
        let local = state.local.borrow();
        let groups: Vec<_> = local
            .groups
            .iter()
            .filter(|(_, group)| {
                Some(group.meta.music) == state.selected_music.as_ref().map(|m| m.data)
            })
            .sorted_by_key(|(_, group)| group.meta.id)
            .collect();

        // Synchronize vec length
        if self.grid_groups.len() > groups.len() {
            self.grid_groups.drain(groups.len()..);
        } else {
            for _ in 0..groups.len().saturating_sub(self.grid_groups.len()) {
                self.grid_groups
                    .push(ItemWidget::new("", Index::from_raw_parts(0, 0)));
            }
        }

        // Synchronize data
        for (widget, &(groups_id, cache)) in self.grid_groups.iter_mut().zip(&groups) {
            widget.data = groups_id;
            widget.text.text = cache.mappers();
        }

        drop(local);

        // Layout
        let columns = 3;
        let rows = self.grid_groups.len() / columns + 1;
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
            let range = (i + 3).min(self.grid_groups.len());
            let mut tab = None;
            for (widget, pos) in self.grid_groups[i..range].iter_mut().zip(layout) {
                widget.update(pos, context);
                if widget.state.clicked {
                    state.select_group(widget.data);
                    tab = Some(LevelSelectTab::Difficulty);
                }
            }
            if let Some(tab) = tab {
                self.select_tab(tab);
            }
        }
    }

    fn grid_levels(&mut self, main: Aabb2<f32>, state: &mut MenuState, context: &mut UiContext) {
        let local = state.local.borrow();
        let levels: Vec<_> = state
            .selected_group
            .as_ref()
            .and_then(|group| local.groups.get(group.data))
            .map(|group| group.levels.iter().sorted_by_key(|level| level.meta.id))
            .into_iter()
            .flatten()
            .collect();

        // Synchronize vec length
        if self.grid_levels.len() > levels.len() {
            self.grid_levels.drain(levels.len()..);
        } else {
            for _ in 0..levels.len().saturating_sub(self.grid_levels.len()) {
                self.grid_levels.push(ItemWidget::new("", 0));
            }
        }

        // Synchronize data
        for (widget, (levels_id, cache)) in
            self.grid_levels.iter_mut().zip(levels.iter().enumerate())
        {
            widget.data = levels_id;
            widget.text.text = cache.meta.name.clone();
        }

        drop(local);

        // Layout
        let columns = 3;
        let rows = self.grid_levels.len() / columns + 1;
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
            let range = (i + 3).min(self.grid_levels.len());
            let mut tab = None;
            for (widget, pos) in self.grid_levels[i..range].iter_mut().zip(layout) {
                widget.update(pos, context);
                if widget.state.clicked {
                    state.select_level(widget.data);
                    tab = Some(LevelSelectTab::Difficulty);
                }
            }
            if let Some(tab) = tab {
                self.select_tab(tab);
            }
        }
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
