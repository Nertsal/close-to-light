use super::*;

use crate::{prelude::Assets, ui::layout::AreaOps};

use ctl_core::types::{Name, UserInfo};
use ctl_local::{Leaderboard, LeaderboardStatus, LoadedBoard, SavedScore};
use ctl_render_core::SubTexture;
use ctl_ui::util::ScrollState;

pub struct LeaderboardWidget {
    pub state: WidgetState,
    pub assets: Rc<Assets>,
    pub window: UiWindow<()>,
    pub pin: ToggleButtonWidget,
    pub reload: IconButtonWidget,
    pub show_title: bool,
    pub title: TextWidget,
    pub subtitle: TextWidget,
    pub level_name: TextWidget,
    pub separator_title: WidgetState,
    pub status: TextWidget,
    pub scroll: ScrollState,

    pub tab: LeaderboardTab,
    pub tab_global: ToggleButtonWidget,
    pub tab_local: ToggleButtonWidget,

    pub rows_state: WidgetState,
    pub rows: Vec<LeaderboardEntryWidget>,
    pub separator_highscore: WidgetState,
    pub highscore: LeaderboardEntryWidget,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderboardTab {
    #[default]
    Global,
    Local,
}

pub struct BadgeWidget {
    pub state: WidgetState,
    pub color: ThemeColor,
    pub text: Option<TextWidget>,
    pub icon: Option<IconWidget>,
}

impl BadgeWidget {
    pub fn new(color: ThemeColor, text: Option<&str>, icon: Option<SubTexture>) -> Self {
        Self {
            state: WidgetState::new(),
            color,
            text: text.map(TextWidget::new),
            icon: icon.map(IconWidget::new),
        }
    }

    pub fn new_dev() -> Self {
        Self::new(ThemeColor::Danger, Some("dev"), None)
    }

    pub fn new_mapper(assets: &Assets) -> Self {
        Self::new(
            ThemeColor::Highlight,
            None,
            Some(assets.atlas.badge_mapper()),
        )
    }

    pub fn new_musician(assets: &Assets) -> Self {
        Self::new(
            ThemeColor::Highlight,
            None,
            Some(assets.atlas.badge_musician()),
        )
    }
}

pub struct LeaderboardEntryWidget {
    pub state: WidgetState,
    pub rank: TextWidget,
    pub player: TextWidget,
    pub badges: Vec<BadgeWidget>,
    pub score: TextWidget,
    pub accuracy: TextWidget,
    pub highlight: bool,
    pub score_grade: ScoreGrade,
    pub grade: IconWidget,
    pub modifiers: Vec<IconWidget>,
}

impl LeaderboardWidget {
    pub fn new(assets: &Rc<Assets>, show_title: bool, online: bool) -> Self {
        Self {
            state: WidgetState::new().with_sfx(WidgetSfxConfig::hover()),
            assets: assets.clone(),
            window: UiWindow::new((), 0.3).reload_skip(),
            pin: ToggleButtonWidget::new_deselectable("").with_icon(assets.atlas.pin()),
            reload: IconButtonWidget::new_normal(assets.atlas.reset()),
            show_title,
            title: TextWidget::new("LEADERBOARD"),
            subtitle: TextWidget::new("login to submit scores"),
            level_name: TextWidget::new("Level - Difficulty"),
            separator_title: WidgetState::new(),
            status: TextWidget::new(""),
            scroll: ScrollState::new(),

            tab: if online {
                LeaderboardTab::Global
            } else {
                LeaderboardTab::Local
            },
            tab_global: ToggleButtonWidget::new("GLOBAL"),
            tab_local: ToggleButtonWidget::new("LOCAL"),

            rows_state: WidgetState::new(),
            rows: Vec::new(),
            separator_highscore: WidgetState::new(),
            highscore: LeaderboardEntryWidget::new(
                assets,
                &MusicInfo::default(),
                &LevelInfo::default(),
                "",
                SavedScore {
                    user: UserInfo {
                        id: 0,
                        name: "player".into(),
                    },
                    score: 0,
                    meta: ctl_core::score::ScoreMeta::default(),
                },
                false,
            ),
        }
    }

    pub fn update_state(&mut self, leaderboard: &Leaderboard) {
        if leaderboard.get_user().is_some() {
            self.subtitle.hide();
        } else {
            self.subtitle.show();
        }

        let user = &leaderboard.get_user().as_ref().map_or(
            UserInfo {
                id: 0,
                name: "local highscore".into(),
            },
            |user| UserInfo {
                id: user.id,
                name: user.name.clone(),
            },
        );
        // let player_name = board.local_high.as_ref().map_or("", |entry| &entry.player);

        self.rows.clear();
        self.load_scores(&leaderboard.get_loaded(), user);
        self.status.text = "".into();
        let global = self.tab == LeaderboardTab::Global;
        match leaderboard.get().status {
            LeaderboardStatus::None => {
                if global {
                    self.status.text = "NOT AVAILABLE".into()
                }
            }
            LeaderboardStatus::Offline => {
                if global {
                    self.status.text = "OFFLINE".into()
                }
            }
            LeaderboardStatus::Pending => {
                if global {
                    self.status.text = "LOADING...".into()
                }
            }
            LeaderboardStatus::Failed => {
                if global {
                    self.status.text = "FETCH FAILED :(".into()
                }
            }
            LeaderboardStatus::Done => {
                if self.rows.is_empty() {
                    self.status.text = "EMPTY :(".into();
                }
            }
        }
    }

    pub fn load_scores(&mut self, board: &LoadedBoard, user: &UserInfo) {
        self.level_name.text = format!("{} - {}", board.music.name, board.level.name).into();
        let scores = match self.tab {
            LeaderboardTab::Global => &board.filtered,
            LeaderboardTab::Local => &board.local,
        };
        self.rows = scores
            .iter()
            .enumerate()
            .map(|(rank, entry)| {
                let score = SavedScore {
                    user: entry.user.clone(),
                    score: entry.score.score(),
                    meta: entry.score.clone(),
                };
                LeaderboardEntryWidget::new(
                    &self.assets,
                    &board.music,
                    &board.level,
                    (rank + 1).to_string(),
                    score,
                    entry.user.id == user.id,
                )
            })
            .collect();
        match &board.local_high {
            None => self.highscore.hide(),
            Some(score) => {
                self.highscore = LeaderboardEntryWidget::new(
                    &self.assets,
                    &board.music,
                    &board.level,
                    board
                        .my_position
                        .map_or("??".into(), |rank| format!("{}", rank + 1)),
                    score.clone(),
                    false,
                );
            }
        }
    }
}

impl WidgetOld for LeaderboardWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        if self.pin.selected {
            // Nullifying a request will prevent the window from getting closed.
            self.window.request = None;
        }
        self.window.update(context.delta_time);

        let main = position;

        self.scroll.drag(context, &self.state);

        let pin = main
            .extend_uniform(-0.5 * context.layout_size)
            .align_aabb(vec2::splat(1.0) * context.font_size, vec2(0.0, 1.0));
        self.pin.update(pin, context);

        let reload = main
            .extend_uniform(-0.5 * context.layout_size)
            .align_aabb(vec2::splat(1.0) * context.font_size, vec2(1.0, 1.0));
        self.reload.update(reload, context);
        if self.reload.icon.state.mouse_left.clicked {
            self.window.request = Some(WidgetRequest::Reload);
        }

        let mut main = main
            .extend_symmetric(-vec2(1.0, 0.0) * context.layout_size)
            .extend_up(-context.layout_size);

        let title = main.cut_top(context.font_size * 1.2);
        if self.show_title {
            self.title.update(title, &context.scale_font(1.1));
        }

        if self.subtitle.state.visible {
            let subtitle = main.cut_top(context.font_size * 0.7);
            self.subtitle.update(subtitle, context);

            let level_name = main.cut_top(context.font_size * 1.0);
            self.level_name.update(level_name, context);

            main.cut_top(context.font_size * 0.3);
        } else {
            main.cut_top(context.font_size * 0.5);
            let level_name = main.cut_top(context.font_size * 1.0);
            main.cut_top(context.font_size * 0.5);
            self.level_name.update(level_name, context);
        }

        let separator = main.cut_top(context.font_size * 0.1);
        self.separator_title.update(separator, context);

        let mut status = main.clone().cut_top(context.font_size * 3.0);
        status.cut_top(context.font_size);
        self.status.update(status, context);

        let highscore = main.cut_bottom(context.font_size * 2.0);
        self.highscore.update(highscore, context);

        let separator = main
            .cut_bottom(context.font_size * 0.5)
            .with_height(context.font_size * 0.1, 0.0)
            .with_width(main.width() * 0.8, 0.5);
        self.separator_highscore.update(separator, context);

        let tabs = main
            .cut_bottom(context.font_size * 1.0)
            .with_width(main.width() * 0.6, 0.5);
        let widgets = [
            (&mut self.tab_global, LeaderboardTab::Global),
            (&mut self.tab_local, LeaderboardTab::Local),
        ];
        for (pos, (widget, tab)) in itertools::izip![tabs.split_columns(widgets.len()), widgets] {
            widget.selected = tab == self.tab;
            widget.update(pos, context);
            if widget.state.mouse_left.clicked {
                self.tab = tab;
            }
        }

        main.cut_bottom(0.2 * context.font_size);

        self.rows_state.update(main, context);
        let main = main.translate(vec2(0.0, -self.scroll.state.current));
        let row = Aabb2::point(main.top_left())
            .extend_right(main.width())
            .extend_down(context.font_size * 2.0);
        let rows = row.stack(vec2(0.0, -row.height()), self.rows.len());
        let height =
            rows.first().map_or(0.0, |row| row.max.y) - rows.last().map_or(0.0, |row| row.min.y);
        for (row, position) in self.rows.iter_mut().zip(rows) {
            row.update(position, context);
        }

        self.scroll
            .overflow(context.delta_time, height, main.height());
    }
}

impl LeaderboardEntryWidget {
    pub fn new(
        assets: &Rc<Assets>,
        music: &MusicInfo,
        level: &LevelInfo,
        rank: impl Into<Name>,
        score: SavedScore,
        highlight: bool,
    ) -> Self {
        let rank = rank.into();
        let mut rank = TextWidget::new(format!("{rank}."));
        rank.align(vec2(1.0, 0.0));

        let mut player = TextWidget::new(score.user.name.clone());
        player.align(vec2(0.0, 0.0));

        let mut badges = Vec::new();
        let player_id = score.user.id;
        if player_id != 0 {
            if player_id == 1 {
                // TODO: query developer id from the server
                badges.push(BadgeWidget::new_dev());
            }
            if music
                .authors
                .iter()
                .any(|author| author.user == Some(player_id))
            {
                badges.push(BadgeWidget::new_musician(assets));
            }
            if level.authors.iter().any(|author| author.id == player_id) {
                badges.push(BadgeWidget::new_mapper(assets));
            }
        }

        let modifiers = score
            .meta
            .category
            .mods
            .iter()
            .map(|modifier| IconWidget::new(assets.get_modifier(modifier)))
            .collect();

        let score_grade = score.meta.score.calculate_grade(score.meta.completion);
        let grade = IconWidget::new(assets.get_grade(score_grade));

        let accuracy = TextWidget::new(format!(
            "rhythm: {}%",
            (score.meta.score.calculated.accuracy.as_f32() * 100.0).floor() as i32,
        ))
        .aligned(vec2(1.0, 1.0));
        let score = TextWidget::new(format!("{}", score.score)).aligned(vec2(1.0, 0.0));

        Self {
            state: WidgetState::new(),
            rank,
            player,
            badges,
            score,
            accuracy,
            highlight,
            score_grade,
            grade,
            modifiers,
        }
    }
}

impl WidgetOld for LeaderboardEntryWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        let mut main = position;
        let theme = context.theme();

        main.cut_top(context.layout_size * 0.5);
        main.cut_bottom(context.layout_size * 0.5);

        let mut top_row = main;
        let bottom_row = top_row.split_bottom(0.5);
        let mod_pos = bottom_row.align_aabb(
            vec2(bottom_row.height(), bottom_row.height()),
            vec2(0.5, 0.5),
        );
        let mods = mod_pos.stack_aligned(
            vec2(mod_pos.width(), 0.0),
            self.modifiers.len(),
            vec2(0.5, 0.5),
        );
        for (modifier, pos) in self.modifiers.iter_mut().zip(mods) {
            modifier.update(pos, context);
            modifier.color = ThemeColor::Danger;
        }

        let mut right = main;
        right.cut_right(context.layout_size);
        let right = right.cut_right(main.width() / 3.0);
        let mut score = right;
        score.min.y = right.center().y - context.font_size * 0.25;
        self.score.update(score, context);
        let mut acc = right;
        acc.max.y = score.min.y;
        self.accuracy.update(acc, &context.scale_font(0.5));

        let grade = if self.modifiers.is_empty() {
            main
        } else {
            top_row
        };
        self.grade.update(grade, &context.scale_font(1.0));
        self.grade.color = match self.score_grade {
            ScoreGrade::F => ThemeColor::Danger,
            _ => ThemeColor::Highlight,
        };

        let mut rank_player = main;
        rank_player.min.y = main.center().y - context.font_size * 0.25;
        let rank = rank_player.cut_left(context.font_size * 1.0);
        self.rank.update(rank, context);
        rank_player.cut_left(context.font_size * 0.2);

        rank_player.max.x = main.center().x;

        self.player.update(rank_player, context);
        self.player.options.color = if self.highlight {
            theme.highlight
        } else {
            theme.light
        };

        let mut badges = position.extend_uniform(-context.pixel_size);
        badges.cut_left(context.font_size * 1.0);
        let mut badges = badges.cut_bottom(context.font_size * 0.9);
        for badge in &mut self.badges {
            let width = if badge.text.is_some() { 1.5 } else { 1.0 };
            let position = badges.cut_left(width * badges.height());
            badge.state.update(position, context);
            if let Some(state) = &mut badge.text {
                state.update(position, context);
            }
            if let Some(state) = &mut badge.icon {
                state.update(position, context);
            }
        }
    }
}
