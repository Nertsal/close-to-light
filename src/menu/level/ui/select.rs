use crate::ui::UiWindow;
use ctl_local::{CachedGroup, SavedScore};

use super::*;

pub struct LevelSelectUI {
    // geng: Geng,
    assets: Rc<Assets>,
    pub state: WidgetState,
    pub tab_levels: TextWidget,
    pub light_level: SelectLightUi,
    pub tab_diffs: TextWidget,
    pub light_diff: SelectLightUi,
    pub separator: WidgetState,

    pub levels: Vec<ItemLevelWidget>,
    pub diffs: Vec<ItemDiffWidget>,
    pub no_diffs: TextWidget,
    pub no_level_selected: TextWidget,
}

pub struct SelectLightUi {
    pub radius: f32,
    pub pos_x: f32,
    pub light_y: ctl_util::Lerp<f32>,
    // pub telegraph_y: ctl_util::Lerp<f32>,
}

impl Default for SelectLightUi {
    fn default() -> Self {
        Self {
            radius: 0.0,
            pos_x: 0.0,
            light_y: ctl_util::Lerp::new_smooth(0.25, 0.0, 0.0),
            // telegraph_y: ctl_util::Lerp::new_smooth(0.25, 0.0, 0.0),
        }
    }
}

impl SelectLightUi {
    pub fn update(&mut self, delta_time: f32) {
        self.light_y.update(delta_time);
        // self.telegraph_y.update(delta_time);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LevelSelectAction {
    EditDifficulty(Index, usize),
    DeleteDifficulty(Index, usize),
    SyncGroup(Index),
    EditGroup(Index),
    DeleteGroup(Index),
}

impl LevelSelectUI {
    pub fn new(_geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            // geng: geng.clone(),
            assets: assets.clone(),
            state: WidgetState::new(),
            tab_levels: TextWidget::new("Level"),
            light_level: SelectLightUi::default(),
            tab_diffs: TextWidget::new("Difficulty"),
            light_diff: SelectLightUi::default(),
            separator: WidgetState::new(),

            levels: Vec::new(),
            diffs: Vec::new(),
            no_diffs: TextWidget::new("Create a Difficulty in the editor"),
            no_level_selected: TextWidget::new("Select a level\n<-"),
        }
    }

    pub fn update(
        &mut self,
        main: Aabb2<f32>,
        state: &mut MenuState,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        self.state.update(main, context);
        self.light_level.update(context.delta_time);
        self.light_diff.update(context.delta_time);

        let mut main = main.extend_uniform(-context.font_size * 0.5);
        main.cut_top(context.layout_size * 1.5);
        let bar = main.cut_top(context.font_size * 1.2);
        main.cut_top(context.layout_size * 1.0);

        let light_size = context.font_size;
        self.tabs(bar.extend_symmetric(-vec2(light_size, 0.0)), context);

        if self.light_level.pos_x == 0.0 {
            self.light_level.light_y.snap_to(bar.center().y);
            // self.light_level.telegraph_y.snap_to(bar.center().y);
        }
        if self.light_diff.pos_x == 0.0 {
            self.light_diff.light_y.snap_to(bar.center().y);
            // self.light_diff.telegraph_y.snap_to(bar.center().y);
        }

        let mut levels = main;
        let mut diffs = levels.split_right(0.5);
        let light_levels = levels.cut_left(light_size);
        let light_diffs = diffs.cut_right(light_size);
        self.light_level.pos_x = light_levels.center().x;
        self.light_level.radius = light_levels.width() * 0.4;
        self.light_diff.pos_x = light_diffs.center().x;
        self.light_diff.radius = light_diffs.width() * 0.4;

        if state.selected_level.is_none() {
            self.light_level.light_y.change_target(bar.center().y);
        }
        if state.selected_diff.is_none() {
            self.light_diff.light_y.change_target(bar.center().y);
        }

        let mut action = None;
        let act = self.layout_levels(levels, state, context);
        action = action.or(act);
        let act = self.layout_diffs(diffs, state, context);
        action = action.or(act);

        action
    }

    fn tabs(&mut self, main: Aabb2<f32>, context: &mut UiContext) {
        let sep_size = vec2(main.width() * 0.9, 0.1 * context.layout_size);
        let sep = main.align_aabb(sep_size, vec2(0.5, 0.0));
        self.separator.update(sep, context);

        let buttons = [
            self.tab_levels
                .state
                .visible
                .then_some(&mut self.tab_levels),
            self.tab_diffs.state.visible.then_some(&mut self.tab_diffs),
        ];
        let all_buttons = buttons.len();
        let buttons: Vec<_> = buttons.into_iter().flatten().collect();

        let buttons_layout = main.split_columns(all_buttons);

        for (button, pos) in buttons.into_iter().zip(buttons_layout) {
            button.update(pos, context);
        }
    }

    fn layout_levels(
        &mut self,
        main: Aabb2<f32>,
        state: &mut MenuState,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        let local = state.context.local.inner.borrow();
        let groups: Vec<_> = local
            .groups
            .iter()
            .sorted_by_key(|(_, group)| group.local.meta.id)
            .collect();

        // Synchronize vec length
        if self.levels.len() != groups.len() {
            self.levels = vec![
                ItemLevelWidget::new(&self.assets, "", Index::from_raw_parts(0, 0),);
                groups.len()
            ];
        }

        // Synchronize data
        for (widget, &(group_id, cached)) in self.levels.iter_mut().zip(&groups) {
            widget.sync(group_id, cached);
        }

        drop(local);

        // Layout
        let spacing = vec2(1.0, 0.75) * context.layout_size;
        let item_size = vec2(main.width() - spacing.x, 1.3 * context.font_size);
        let rows = Aabb2::point(main.top_left() + vec2(spacing.x * 0.5, -item_size.y - spacing.y))
            .extend_positive(item_size)
            .stack(vec2(0.0, -item_size.y - spacing.y), self.levels.len());

        let mut action = None;
        for (widget, pos) in self.levels.iter_mut().zip(rows) {
            let act = widget.update(pos, context);
            action = action.or(act);
            if widget.state.mouse_left.clicked {
                state.select_level(widget.index);
                self.light_level.light_y.change_target(pos.center().y);
            }
            // if widget.state.hovered {
            //     self.light_level.telegraph_y.change_target(pos.center().y);
            // }
        }

        action
    }

    fn layout_diffs(
        &mut self,
        main: Aabb2<f32>,
        state: &mut MenuState,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        let local = state.context.local.inner.borrow();
        let group_idx = state.switch_level;

        if group_idx.is_none() {
            self.no_level_selected.show();
            self.no_diffs.hide();
            let size = vec2(main.width().min(5.0), 1.2) * context.font_size;
            let pos = main
                .align_aabb(size, vec2(0.5, 1.0))
                .translate(vec2(0.0, -1.0) * context.font_size);
            self.no_level_selected.update(pos, context);
        } else {
            self.no_level_selected.hide();
        }

        let levels: Vec<_> = group_idx
            .and_then(|group| local.groups.get(group))
            .map(|group| {
                group
                    .local
                    .data
                    .levels
                    .iter()
                    .zip(&group.local.meta.levels)
                    .map(|(data, meta)| LevelFull {
                        meta: meta.clone(),
                        data: data.clone(),
                    })
            })
            .into_iter()
            .flatten()
            .collect();

        // Synchronize vec length
        if self.diffs.len() != levels.len() {
            if let Some(cached) = levels.first() {
                self.diffs = vec![
                    ItemDiffWidget::new(
                        &self.assets,
                        "",
                        Index::from_raw_parts(0, 0),
                        0,
                        cached.clone(),
                    );
                    levels.len()
                ];
            } else {
                self.diffs.clear();
            }
        }

        let group_idx = group_idx?;
        let group = state.context.local.get_group(group_idx)?;

        // Synchronize data
        for (widget, (level_id, cached)) in self.diffs.iter_mut().zip(levels.iter().enumerate()) {
            let origin_hash = group.origin.as_ref().and_then(|info| {
                info.levels
                    .iter()
                    .find(|level| level.id == cached.meta.id)
                    .map(|level| &level.hash)
            });
            let edited =
                origin_hash.is_some_and(|hash| Some(hash) != group.level_hashes.get(level_id));
            let local_score = state
                .leaderboard
                .loaded
                .all_highscores
                .get(&cached.meta.hash);
            widget.sync(group_idx, level_id, cached, local_score, edited, context);
        }

        drop(local);

        if self.diffs.is_empty() {
            self.no_diffs.show();
            let size = vec2(10.0, 1.2) * context.font_size;
            let pos = main.align_aabb(size, vec2(0.5, 0.5));
            self.no_diffs.update(pos, context);
        } else {
            self.no_diffs.hide();
        }

        // Layout
        let spacing = vec2(1.0, 0.75) * context.layout_size;
        let item_size = vec2(main.width() - spacing.x * 5.0, 1.15 * context.font_size);
        let rows = Aabb2::point(
            main.top_right() + vec2(-main.width() / 2.0, -item_size.y * 0.5 - spacing.y),
        )
        .extend_symmetric(item_size / 2.0)
        .stack(vec2(0.0, -item_size.y - spacing.y), self.diffs.len());

        let mut action = None;
        for (widget, pos) in self.diffs.iter_mut().zip(rows) {
            let act = widget.update(pos, context);
            action = action.or(act);
            if widget.state.mouse_left.clicked {
                state.select_difficulty(widget.index);
                self.light_diff.light_y.change_target(pos.center().y);
            }
            // if widget.state.hovered {
            //     self.light_diff.telegraph_y.change_target(pos.center().y);
            // }
        }

        action
    }
}

#[derive(Clone)]
pub struct ItemLevelWidget {
    pub state: WidgetState,
    pub edited: IconWidget,
    pub local: IconWidget,
    pub menu: ItemMenuWidget,
    pub text: TextWidget,
    pub index: Index,
}

impl ItemLevelWidget {
    pub fn new(assets: &Rc<Assets>, text: impl Into<Name>, index: Index) -> Self {
        Self {
            state: WidgetState::new().with_sfx(WidgetSfxConfig::all()),
            edited: IconWidget::new(assets.atlas.star()),
            local: IconWidget::new(assets.atlas.local()),
            menu: ItemMenuWidget::new(assets),
            text: TextWidget::new(text).aligned(vec2(0.5, 0.5)),
            index,
        }
    }

    pub fn sync(&mut self, group_id: Index, cached: &CachedGroup) {
        self.index = group_id;
        self.text.text = cached
            .local
            .music
            .as_ref()
            .map(|music| music.meta.name.clone())
            .unwrap_or_else(|| cached.local.meta.owner.name.clone());
        if cached.local.meta.id == 0 {
            self.local.show();
            self.edited.hide();
        } else {
            self.local.hide();
            if cached
                .origin
                .as_ref()
                .is_some_and(|info| info.hash != cached.local.meta.hash)
            {
                self.edited.show();
            } else {
                self.edited.hide();
            }
        }
    }

    fn update(
        &mut self,
        mut position: Aabb2<f32>,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        if self.state.mouse_right.clicked {
            self.menu.window.request = Some(WidgetRequest::Open);
            self.menu.show();
        } else if !self.state.hovered && !self.menu.state.hovered {
            self.menu.window.request = Some(WidgetRequest::Close);
        }
        self.menu.update(position, context);
        if self.menu.state.hovered {
            context.update_focus(true);
        }

        self.state.update(position, context);

        let widgets = [&mut self.edited, &mut self.local];
        if widgets.iter().any(|widget| widget.state.visible) {
            let icons = position
                .cut_left(position.height() / 2.0)
                .extend_left(-context.font_size * 0.2)
                .extend_symmetric(-vec2(0.0, context.font_size * 0.15));
            let positions = icons.split_rows(widgets.len());
            for (widget, pos) in widgets.into_iter().zip(positions) {
                widget.update(pos, context);
            }
        }

        self.text.update(position, &context.scale_font(0.9));

        let mut action = None;
        if self.menu.edit.icon.state.mouse_left.clicked {
            action = Some(LevelSelectAction::EditGroup(self.index));
        } else if self.menu.sync.icon.state.mouse_left.clicked {
            action = Some(LevelSelectAction::SyncGroup(self.index));
        } else if self.menu.delete.icon.state.mouse_left.clicked {
            action = Some(LevelSelectAction::DeleteGroup(self.index));
        }
        action
    }
}

#[derive(Clone)]
pub struct ItemDiffWidget {
    pub state: WidgetState,
    pub edited: IconWidget,
    pub local: IconWidget,
    pub text: TextWidget,
    pub group: Index,
    pub index: usize,
    pub level: LevelFull,
    pub grade: IconWidget,
    pub menu: ItemMenuWidget,
}

impl ItemDiffWidget {
    pub fn new(
        assets: &Rc<Assets>,
        text: impl Into<Name>,
        group: Index,
        index: usize,
        level: LevelFull,
    ) -> Self {
        let mut menu = ItemMenuWidget::new(assets);
        menu.sync.hide();
        Self {
            state: WidgetState::new().with_sfx(WidgetSfxConfig::all()),
            edited: IconWidget::new(assets.atlas.star()),
            local: IconWidget::new(assets.atlas.local()),
            text: TextWidget::new(text).aligned(vec2(0.5, 0.5)),
            group,
            index,
            level,
            grade: IconWidget::new(assets.atlas.grade_s()),
            menu,
        }
    }

    pub fn sync(
        &mut self,
        group_idx: Index,
        level_index: usize,
        cached: &LevelFull,
        local_highscore: Option<&SavedScore>,
        edited: bool,
        context: &UiContext,
    ) {
        self.index = level_index;
        self.group = group_idx;
        self.level = cached.clone();
        self.text.text = cached.meta.name.clone();
        if cached.meta.id == 0 {
            self.local.show();
            self.edited.hide();
        } else {
            self.local.hide();
            if edited {
                self.edited.show();
            } else {
                self.edited.hide();
            }
        }

        match local_highscore {
            Some(highscore) => {
                let grade = highscore
                    .meta
                    .score
                    .calculate_grade(highscore.meta.completion);
                self.grade.texture = context.context.assets.get_grade(grade);
                self.grade.color = match grade {
                    ScoreGrade::F => ThemeColor::Danger,
                    _ => ThemeColor::Highlight,
                };
                self.grade.show();
            }
            None => {
                self.grade.hide();
            }
        }
    }

    fn update(
        &mut self,
        mut position: Aabb2<f32>,
        context: &mut UiContext,
    ) -> Option<LevelSelectAction> {
        if self.state.mouse_right.clicked {
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
            context.update_focus(true);
        }

        self.state.update(position, context);

        let grade = position
            .align_aabb(vec2::splat(position.height()), vec2(1.0, 0.5))
            .translate(vec2(position.height(), 0.0));
        self.grade.update(grade, context);

        let widgets = [&mut self.edited, &mut self.local];
        if widgets.iter().any(|widget| widget.state.visible) {
            let icons = position
                .cut_left(position.height() / 2.0)
                .extend_left(-context.font_size * 0.2)
                .extend_symmetric(-vec2(0.0, context.font_size * 0.15));
            let positions = icons.split_rows(widgets.len());
            for (widget, pos) in widgets.into_iter().zip(positions) {
                widget.update(pos, context);
            }
        }

        self.text.update(position, &context.scale_font(0.9));

        let mut action = None;
        if self.menu.edit.icon.state.mouse_left.clicked {
            action = Some(LevelSelectAction::EditDifficulty(self.group, self.index));
        } else if self.menu.sync.icon.state.mouse_left.clicked {
            action = Some(LevelSelectAction::SyncGroup(self.group));
        } else if self.menu.delete.icon.state.mouse_left.clicked {
            action = Some(LevelSelectAction::DeleteDifficulty(self.group, self.index));
        }
        action
    }
}

#[derive(Clone)]
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
            sync: IconButtonWidget::new_normal(assets.atlas.reset()),
            edit: IconButtonWidget::new_normal(assets.atlas.edit()),
            delete: IconButtonWidget::new_danger(assets.atlas.trash()),
        }
    }
}

impl WidgetOld for ItemMenuWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.window.update(context.delta_time);
        if self.window.show.time.is_min() {
            self.hide();
        }

        let mut columns: Vec<_> = [&mut self.delete, &mut self.edit, &mut self.sync]
            .into_iter()
            .filter(|widget| widget.icon.state.visible)
            .collect();

        let position = position.translate(vec2(context.layout_size, -position.height()));
        let size = vec2(columns.len() as f32, 1.0) * 2.0 * context.layout_size;
        let position = position.align_aabb(size, vec2(0.0, 1.0));

        self.state.update(position, context);

        if !self.window.show.time.is_max() {
            return;
        }

        let positions = position.split_columns(columns.len());
        for (widget, pos) in columns.iter_mut().zip(positions) {
            widget.update(pos, context);
        }

        context.update_focus(self.state.hovered);
    }
}
