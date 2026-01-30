use super::*;

use ctl_ui::layout::AreaOps;

pub struct PauseWidget {
    pub state: WidgetState,
    pub window: UiWindow<()>,
    pub title: TextWidget,
    pub resume: ButtonWidget,
    pub retry: ButtonWidget,
    pub quit: ButtonWidget,
}

impl PauseWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.5),
            title: TextWidget::new("Paused"),
            resume: ButtonWidget::new("Resume"),
            retry: ButtonWidget::new("Retry"),
            quit: ButtonWidget::new("Quit"),
        }
    }
}

impl WidgetOld for PauseWidget {
    simple_widget_state!();

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.window.update(context.delta_time);
        self.state.update(position, context);
        let mut main = position.extend_uniform(-context.font_size * 0.5);

        let title = main.cut_top(context.font_size * 1.2);
        self.title.update(title, context);

        let buttons = [&mut self.resume, &mut self.retry, &mut self.quit];
        let layout = main.split_rows(buttons.len());
        for (button, pos) in buttons.into_iter().zip(layout) {
            let pos = pos.extend_symmetric(-vec2(0.0, 0.1) * context.font_size);
            button.update(pos, context);
        }
    }
}
