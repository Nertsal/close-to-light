use super::*;

use crate::{prelude::Assets, ui::layout::AreaOps};

use ctl_local::ScoreMeta;

pub struct ScoreWidget {
    pub state: WidgetState,
    pub assets: Rc<Assets>,
    pub window: UiWindow<()>,
    pub saved_score: ScoreMeta,
    pub music_name: TextWidget,
    pub difficulty_name: TextWidget,
    pub modifiers: Vec<IconWidget>,
    pub score_text: TextWidget,
    pub score_value: TextWidget,
    pub accuracy_bar: WidgetState,
    pub accuracy_value: TextWidget,
    pub accuracy_text: TextWidget,
    pub precision_bar: WidgetState,
    pub precision_value: TextWidget,
    pub precision_text: TextWidget,
}

impl ScoreWidget {
    pub fn new(assets: &Rc<Assets>) -> Self {
        Self {
            state: WidgetState::new(),
            assets: assets.clone(),
            window: UiWindow::new((), 0.3).reload_skip(),
            saved_score: ScoreMeta::default(),
            music_name: TextWidget::new("<Music Title>"),
            difficulty_name: TextWidget::new("<Difficulty>"),
            modifiers: vec![],
            score_text: TextWidget::new("Score"),
            score_value: TextWidget::new("XXXXX"),
            accuracy_bar: WidgetState::new(),
            accuracy_value: TextWidget::new("100.00%"),
            accuracy_text: TextWidget::new("Accuracy"),
            precision_bar: WidgetState::new(),
            precision_value: TextWidget::new("99.99%"),
            precision_text: TextWidget::new("Precision"),
        }
    }

    pub fn update_state(&mut self, score: &ScoreMeta, music: &MusicInfo, level: &LevelInfo) {
        self.saved_score = score.clone();
        self.music_name.text = music.name.clone();
        self.difficulty_name.text = level.name.clone();
        self.modifiers = score
            .category
            .mods
            .iter()
            .map(|modifier| {
                let mut icon = IconWidget::new(self.assets.get_modifier(modifier));
                icon.color = ThemeColor::Danger;
                icon
            })
            .collect();
        self.score_value.text = format!("{}", score.score.calculated.combined).into();
        self.accuracy_value.text =
            format!("{:.2}%", score.score.calculated.accuracy.as_f32() * 100.0).into();
        self.precision_value.text =
            format!("{:.2}%", score.score.calculated.precision.as_f32() * 100.0).into();
    }
}

impl WidgetOld for ScoreWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.window.update(context.delta_time);

        let main = position;

        let mut main = main.extend_symmetric(-vec2(1.0, 1.0) * context.layout_size);

        let music_name = main.cut_top(context.font_size * 1.2);
        self.music_name.update(music_name, &context.scale_font(1.1)); // TODO: better

        let diff = main.cut_top(context.font_size * 1.0);
        self.difficulty_name.update(diff, context);
        self.difficulty_name.options.color = context.theme().highlight;

        let mods_row = main.cut_top(context.font_size * 1.0);
        let mods = mods_row.stack_aligned(
            vec2(mods_row.height(), 0.0),
            self.modifiers.len(),
            vec2(0.5, 0.5),
        );
        for (modifier, pos) in self.modifiers.iter_mut().zip(mods) {
            modifier.update(pos, context);
        }

        let score = main.cut_top(context.font_size * 1.5);
        self.score_value.update(score, &context.scale_font(1.5));

        main.cut_top(context.font_size * 0.2);

        let columns = main.split_columns(2);
        let mut acc_col = columns[0];
        let mut prec_col = columns[1];

        let bar_height = context.font_size * 3.0;
        let bar_width = context.font_size * 1.0;
        let acc_bar = acc_col.cut_top(bar_height);
        self.accuracy_bar
            .update(acc_bar.with_width(bar_width, 0.5), context);
        let prec_bar = prec_col.cut_top(bar_height);
        self.precision_bar
            .update(prec_bar.with_width(bar_width, 0.5), context);

        acc_col.cut_top(context.font_size * 0.1);
        self.accuracy_value
            .update(acc_col.cut_top(context.font_size * 1.0), context);
        self.accuracy_text
            .update(acc_col.cut_top(context.font_size * 1.0), context);

        prec_col.cut_top(context.font_size * 0.1);
        self.precision_value
            .update(prec_col.cut_top(context.font_size * 1.0), context);
        self.precision_text
            .update(prec_col.cut_top(context.font_size * 1.0), context);
    }
}
