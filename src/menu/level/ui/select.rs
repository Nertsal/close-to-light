use crate::ui::UiWindow;

use super::*;

pub struct LevelSelectUI {
    // geng: Geng,
    assets: Rc<Assets>,
    pub tab_music: ToggleWidget,
    pub tab_groups: ToggleWidget,
    pub tab_levels: ToggleWidget,
    pub separator: WidgetState,

    pub add_music: TextWidget,
    pub grid_music: Vec<ItemMusicWidget>,
    pub add_group: AddItemWidget,
    pub grid_groups: Vec<ItemGroupWidget>,
    pub add_level: TextWidget,
    pub grid_levels: Vec<ItemLevelWidget>,
}

#[derive(Debug, Clone, Copy)]
pub enum LevelSelectAction {
    SyncLevel(Index, usize),
    EditLevel(Index, usize),
    DeleteLevel(Index, usize),
    SyncGroup(Index),
    EditGroup(Index),
    DeleteGroup(Index),
}

#[derive(Debug, Clone, Copy)]
pub enum LevelSelectTab {
    Music,
    Group,
    Difficulty,
}

impl LevelSelectUI {
    pub fn new(_geng: &Geng, assets: &Rc<Assets>) -> Self {
        let mut ui = Self {
            // geng: geng.clone(),
            assets: assets.clone(),
            tab_music: ToggleWidget::new("Music"),
            tab_groups: ToggleWidget::new("Group"),
            tab_levels: ToggleWidget::new("Difficulty"),
            separator: WidgetState::new(),

            add_music: TextWidget::new("+"),
            grid_music: Vec::new(),
            add_group: AddItemWidget::new(assets),
            grid_groups: Vec::new(),
            add_level: TextWidget::new("+"),
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
            LevelSelectTab::Group => {
                self.tab_music.show();
                &mut self.tab_groups
            }
            LevelSelectTab::Difficulty => {
                self.tab_music.show();
                self.tab_groups.show();
                &mut self.tab_levels
            }
        };
        tab.show();
        tab.selected = true;
    }

    pub fn update(
        &mut self,
        main: Aabb2<f32>,
        state: &mut MenuState,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        let mut main = main;
        main.cut_top(context.layout_size * 1.5);
        let bar = main.cut_top(context.font_size * 1.2);
        main.cut_top(context.layout_size * 1.0);

        self.tabs(bar, context);

        let mut action = None;
        if self.tab_music.selected {
            self.grid_music(main, state, context);
        } else if self.tab_groups.selected {
            let act = self.grid_groups(main, state, context);
            action = action.or(act);
        } else if self.tab_levels.selected {
            let act = self.grid_levels(main, state, context);
            action = action.or(act);
        }

        action
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
        let local = state.context.local.clone();
        let music: Vec<_> = local
            .inner
            .borrow()
            .music
            .iter()
            .sorted_by_key(|(&k, _)| k)
            .map(|(_, music)| music.clone())
            .collect();

        // Synchronize vec length
        if self.grid_music.len() > music.len() {
            self.grid_music.drain(music.len()..);
        } else if let Some(cached) = music.first() {
            for _ in 0..music.len().saturating_sub(self.grid_music.len()) {
                self.grid_music
                    .push(ItemMusicWidget::new("", cached.clone()));
            }
        }

        // Synchronize data
        for (widget, cache) in self.grid_music.iter_mut().zip(&music) {
            widget.music = cache.clone();
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

            let mut row_items = 3;
            let mut skip = 0;
            if row == 0 {
                skip = 1;
                let pos = layout[0];
                self.add_music
                    .update(pos.extend_symmetric(-pos.size() * 0.1), context);
            }
            row_items -= skip;

            let i = columns * row;
            let range = (i + row_items).min(self.grid_music.len());

            let mut tab = None;
            for (widget, pos) in self.grid_music[i..range]
                .iter_mut()
                .zip(layout.into_iter().skip(skip))
            {
                widget.update(pos, context);
                if widget.state.clicked {
                    state.select_music(widget.music.meta.id);
                    tab = Some(LevelSelectTab::Group);
                }
            }
            if let Some(tab) = tab {
                self.select_tab(tab);
            }
        }
    }

    fn grid_groups(
        &mut self,
        main: Aabb2<f32>,
        state: &mut MenuState,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        let local = state.context.local.inner.borrow();
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
                self.grid_groups.push(ItemGroupWidget::new(
                    &self.assets,
                    "",
                    Index::from_raw_parts(0, 0),
                ));
            }
        }

        // Synchronize data
        for (widget, &(groups_id, cache)) in self.grid_groups.iter_mut().zip(&groups) {
            widget.index = groups_id;
            widget.text.text = cache.mappers().into();
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

        let mut action = None;
        for row in 0..rows {
            let top_left =
                Aabb2::point(main.top_left() - vec2(0.0, item_size.y + spacing.y) * row as f32)
                    .extend_right(item_size.x)
                    .extend_down(item_size.y);
            let layout = top_left.stack(vec2(item_size.x + spacing.x, 0.0), columns);

            let mut row_items = 3;
            let mut skip = 0;
            if row == 0 {
                skip = 1;
                let pos = layout[0];
                self.add_group
                    .update(pos.extend_symmetric(-pos.size() * 0.1), context);
            }
            row_items -= skip;

            let i = columns * row;
            let range = (i + row_items).min(self.grid_groups.len());

            let mut tab = None;
            for (widget, pos) in self.grid_groups[i..range]
                .iter_mut()
                .zip(layout.into_iter().skip(skip))
            {
                let act = widget.update(pos, context);
                action = action.or(act);
                if widget.state.clicked {
                    state.select_group(widget.index);
                    tab = Some(LevelSelectTab::Difficulty);
                }
            }
            if let Some(tab) = tab {
                self.select_tab(tab);
            }
        }

        action
    }

    fn grid_levels(
        &mut self,
        main: Aabb2<f32>,
        state: &mut MenuState,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        let local = state.context.local.inner.borrow();
        let group_idx = state.selected_group.as_ref().map(|group| group.data);
        let levels: Vec<_> = group_idx
            .and_then(|group| local.groups.get(group))
            .map(|group| {
                group
                    .levels
                    .iter()
                    .sorted_by_key(|level| level.meta.id)
                    .cloned()
            })
            .into_iter()
            .flatten()
            .collect();

        // Synchronize vec length
        if self.grid_levels.len() > levels.len() {
            self.grid_levels.drain(levels.len()..);
        } else if let Some(cached) = levels.first() {
            for _ in 0..levels.len().saturating_sub(self.grid_levels.len()) {
                self.grid_levels.push(ItemLevelWidget::new(
                    &self.assets,
                    "",
                    Index::from_raw_parts(0, 0),
                    0,
                    cached.clone(),
                ));
            }
        }

        // Synchronize data
        for (widget, (levels_id, cache)) in
            self.grid_levels.iter_mut().zip(levels.iter().enumerate())
        {
            widget.index = levels_id;
            widget.group = group_idx.unwrap();
            widget.level = cache.clone();
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

        let mut action = None;
        for row in 0..rows {
            let top_left =
                Aabb2::point(main.top_left() - vec2(0.0, item_size.y + spacing.y) * row as f32)
                    .extend_right(item_size.x)
                    .extend_down(item_size.y);
            let layout = top_left.stack(vec2(item_size.x + spacing.x, 0.0), columns);

            let mut row_items = 3;
            let mut skip = 0;
            if row == 0 {
                skip = 1;
                let pos = layout[0];
                self.add_level
                    .update(pos.extend_symmetric(-pos.size() * 0.1), context);
            }
            row_items -= skip;

            let i = columns * row;
            let range = (i + row_items).min(self.grid_levels.len());

            let mut tab = None;
            for (widget, pos) in self.grid_levels[i..range]
                .iter_mut()
                .zip(layout.into_iter().skip(skip))
            {
                let act = widget.update(pos, context);
                action = action.or(act);
                if widget.state.clicked {
                    state.select_level(widget.index);
                    tab = Some(LevelSelectTab::Difficulty);
                }
            }
            if let Some(tab) = tab {
                self.select_tab(tab);
            }
        }

        action
    }
}

pub struct AddItemWidget {
    pub state: WidgetState,
    pub text: TextWidget,
    pub menu: NewMenuWidget,
}

impl AddItemWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            state: WidgetState::new(),
            text: TextWidget::new("+"),
            menu: NewMenuWidget::new(assets),
        }
    }
}

impl Widget for AddItemWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.text.update(position, context);
        if self.state.clicked {
            self.menu.window.request = Some(WidgetRequest::Open);
            self.menu.show();
        } else if !self.state.hovered && !self.menu.state.hovered {
            self.menu.window.request = Some(WidgetRequest::Close);
        }
        self.menu.window.update(context.delta_time);
        if self.menu.window.show.time.is_min() {
            self.menu.hide();
        } else {
            self.menu.update(position, context);
        }
        if self.menu.state.hovered {
            context.can_focus = false;
        }
    }
}

pub struct ItemMusicWidget {
    pub state: WidgetState,
    pub text: TextWidget,
    pub music: Rc<CachedMusic>,
}

impl ItemMusicWidget {
    pub fn new(text: impl Into<Name>, music: Rc<CachedMusic>) -> Self {
        Self {
            state: WidgetState::new(),
            text: TextWidget::new(text).aligned(vec2(0.5, 0.5)),
            music,
        }
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.text.update(position, &mut context.scale_font(0.9));
    }
}

pub struct ItemGroupWidget {
    pub state: WidgetState,
    pub menu: ItemMenuWidget,
    pub text: TextWidget,
    pub index: Index,
}

impl ItemGroupWidget {
    pub fn new(assets: &Rc<Assets>, text: impl Into<Name>, index: Index) -> Self {
        Self {
            state: WidgetState::new(),
            menu: ItemMenuWidget::new(assets),
            text: TextWidget::new(text).aligned(vec2(0.5, 0.5)),
            index,
        }
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        if self.state.right_clicked {
            self.menu.window.request = Some(WidgetRequest::Open);
            self.menu.show();
        } else if !self.state.hovered && !self.menu.state.hovered {
            self.menu.window.request = Some(WidgetRequest::Close);
        }
        self.menu.window.update(context.delta_time);
        if self.menu.window.show.time.is_min() {
            self.menu.hide();
        }
        self.menu.update(position, context);
        if self.menu.state.hovered {
            context.can_focus = false;
        }

        self.state.update(position, context);
        self.text.update(position, &mut context.scale_font(0.9));

        let mut action = None;
        if self.menu.edit.state.clicked {
            action = Some(LevelSelectAction::EditGroup(self.index));
        } else if self.menu.sync.state.clicked {
            action = Some(LevelSelectAction::SyncGroup(self.index));
        } else if self.menu.delete.state.clicked {
            action = Some(LevelSelectAction::DeleteGroup(self.index));
        }
        action
    }
}

pub struct ItemLevelWidget {
    pub state: WidgetState,
    pub text: TextWidget,
    pub group: Index,
    pub index: usize,
    pub level: Rc<CachedLevel>,
    pub menu: ItemMenuWidget,
}

impl ItemLevelWidget {
    pub fn new(
        assets: &Rc<Assets>,
        text: impl Into<Name>,
        group: Index,
        index: usize,
        level: Rc<CachedLevel>,
    ) -> Self {
        Self {
            state: WidgetState::new(),
            text: TextWidget::new(text).aligned(vec2(0.5, 0.5)),
            group,
            index,
            level,
            menu: ItemMenuWidget::new(assets),
        }
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        if self.state.right_clicked {
            self.menu.window.request = Some(WidgetRequest::Open);
            self.menu.show();
        } else if !self.state.hovered && !self.menu.state.hovered {
            self.menu.window.request = Some(WidgetRequest::Close);
        }
        self.menu.window.update(context.delta_time);
        if self.menu.window.show.time.is_min() {
            self.menu.hide();
        }
        self.menu.update(position, context);
        if self.menu.state.hovered {
            context.can_focus = false;
        }

        self.state.update(position, context);
        self.text.update(position, &mut context.scale_font(0.9));

        let mut action = None;
        if self.menu.edit.state.clicked {
            action = Some(LevelSelectAction::EditLevel(self.group, self.index));
        } else if self.menu.sync.state.clicked {
            action = Some(LevelSelectAction::SyncLevel(self.group, self.index));
        } else if self.menu.delete.state.clicked {
            action = Some(LevelSelectAction::DeleteLevel(self.group, self.index));
        }
        action
    }
}

pub struct NewMenuWidget {
    pub window: UiWindow<()>,
    pub state: WidgetState,
    pub create: TextWidget,
    pub browse: TextWidget,
}

impl NewMenuWidget {
    pub fn new(_assets: &Rc<Assets>) -> Self {
        Self {
            window: UiWindow::new((), 0.15),
            state: WidgetState::new(),
            create: TextWidget::new("create"),
            browse: TextWidget::new("browse"),
        }
    }
}

impl Widget for NewMenuWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        let position =
            position.translate(vec2(0.0, -position.height() + context.layout_size * 0.5));
        let size = vec2(position.width(), 2.5 * context.font_size);
        let position = position.align_aabb(size, vec2(0.0, 1.0));

        self.state.update(position, context);
        let rows = [&mut self.browse, &mut self.create];
        let spacing = context.layout_size * 0.3;
        let item_size = vec2(
            size.x - context.layout_size * 0.6,
            (size.y - spacing) / rows.len() as f32 - spacing,
        );
        let item = position
            .extend_up(-spacing)
            .align_aabb(item_size, vec2(0.5, 1.0));
        let positions = item.stack(vec2(0.0, -item.height() - spacing), rows.len());
        for (widget, pos) in rows.into_iter().zip(positions) {
            widget.update(pos, &mut context.scale_font(0.7));
        }
    }
}

pub struct ItemMenuWidget {
    pub window: UiWindow<()>,
    pub state: WidgetState,
    pub sync: IconButtonWidget,
    pub edit: IconButtonWidget,
    pub delete: IconButtonWidget,
}

impl ItemMenuWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            window: UiWindow::new((), 0.15),
            state: WidgetState::new(),
            sync: IconButtonWidget::new_normal(&assets.sprites.reset),
            edit: IconButtonWidget::new_normal(&assets.sprites.edit),
            delete: IconButtonWidget::new_danger(&assets.sprites.trash),
        }
    }
}

impl Widget for ItemMenuWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        let position = position.translate(vec2(context.layout_size, -position.height()));
        let size = vec2(3.0, 1.0) * 2.0 * context.layout_size;
        let position = position.align_aabb(size, vec2(0.0, 1.0));

        self.state.update(position, context);
        let columns = [&mut self.sync, &mut self.edit, &mut self.delete];
        let positions = position.split_columns(columns.len());
        for (widget, pos) in columns.into_iter().zip(positions) {
            widget.update(pos, context);
        }
    }
}
