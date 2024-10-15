mod config;
mod edit;

pub use self::{config::*, edit::*};

use super::*;

use crate::ui::{layout::AreaOps, widget::*};

const HELP: &str = "
Scroll / Arrow keys - move through time
Hold Shift / Alt - scroll slower / faster
Space - play music
Q / E - rotate
Ctrl+Scroll - scale lights
F1 - Hide UI
";

/// Layout and state of the UI.
pub struct EditorUI {
    pub screen: WidgetState,
    pub game: WidgetState,

    pub confirm: Option<ConfirmWidget>,

    pub exit: ButtonWidget,
    pub help: IconWidget,
    pub tab_edit: ButtonWidget,
    pub tab_config: ButtonWidget,

    pub unsaved: TextWidget,
    pub save: ButtonWidget,

    pub help_text: TextWidget,
    pub edit: EditorEditWidget,
    pub config: EditorConfigWidget,
}

impl EditorUI {
    pub fn new(context: Context) -> Self {
        let assets = &context.assets;
        Self {
            screen: default(),
            game: default(),

            confirm: None,

            exit: ButtonWidget::new("Exit"),
            help: IconWidget::new(&assets.sprites.help),
            tab_edit: ButtonWidget::new("Edit"),
            tab_config: ButtonWidget::new("Config"),

            unsaved: TextWidget::new("Save to apply changes").aligned(vec2(1.0, 0.5)),
            save: ButtonWidget::new("Save"),

            help_text: TextWidget::new(HELP).aligned(vec2(0.0, 1.0)),
            config: {
                let mut w = EditorConfigWidget::new(assets);
                w.hide();
                w
            },
            edit: EditorEditWidget::new(context),
        }
    }

    pub fn layout(
        &mut self,
        editor: &Editor,
        screen: Aabb2<f32>,
        context: &mut UiContext,
    ) -> (bool, Vec<EditorStateAction>) {
        let screen = screen.fit_aabb(vec2(16.0, 9.0), vec2::splat(0.5));

        let mut actions = vec![];

        let font_size = screen.height() * 0.035;
        let layout_size = screen.height() * 0.03;

        context.font_size = font_size;
        context.layout_size = layout_size;
        context.screen = screen;

        self.screen.update(screen, context);

        {
            let max_size = screen.size() * 0.75;

            let ratio = 15.0 / 9.0;
            let max_height = max_size.y.min(max_size.x / ratio);

            let game_height = max_height;
            let game_size = vec2(game_height * ratio, game_height);

            let game = screen.align_aabb(game_size, vec2(0.5, 0.5));
            self.game.update(game, context);
        }

        if let Some(confirm) = &mut self.confirm {
            let size = vec2(20.0, 10.0) * layout_size;
            let window = screen.align_aabb(size, vec2(0.5, 0.5));
            confirm.update(window, context);
            if confirm.confirm.state.clicked {
                confirm.window.show.going_up = false;
                actions.push(EditorStateAction::ConfirmPopupAction);
            } else if confirm.discard.state.clicked {
                confirm.window.show.going_up = false;
                actions.push(EditorAction::ClosePopup.into());
            } else if confirm.window.show.time.is_min() {
                self.confirm = None;
            }

            // NOTE: When confirm is active, you cant interact with other widgets
            context.update_focus(true);
        } else if let Some(popup) = &editor.confirm_popup {
            let mut confirm = ConfirmWidget::new(
                &editor.context.assets,
                popup.title.clone(),
                popup.message.clone(),
            );
            confirm.window.show.going_up = true;
            self.confirm = Some(confirm);
        }

        let mut main = screen;

        let mut top_bar = main.cut_top(font_size * 1.5);

        let exit = top_bar.cut_left(layout_size * 5.0);
        self.exit.update(exit, context);
        if self.exit.text.state.clicked {
            if editor.is_changed() {
                actions.push(
                    EditorAction::PopupConfirm(
                        ConfirmAction::ExitUnsaved,
                        "there are unsaved changes".into(),
                    )
                    .into(),
                );
            } else {
                actions.push(EditorStateAction::Exit);
            }
        }

        let help = top_bar.cut_left(layout_size * 3.0);
        self.help.update(help, context);

        let help_text = Aabb2::point(help.bottom_right())
            .extend_right(layout_size * 12.0)
            .extend_down(font_size * HELP.lines().count() as f32);
        self.help_text.update(help_text, context);
        context.update_focus(self.help_text.state.hovered);
        if self.help.state.hovered {
            self.help_text.show();
        } else if !self.help_text.state.hovered
            && !Aabb2::from_corners(
                self.help.state.position.top_left(),
                self.help_text.state.position.bottom_right(),
            )
            .contains(context.cursor.position)
        {
            self.help_text.hide();
        }

        let tabs = [&mut self.tab_edit, &mut self.tab_config];
        let tab = Aabb2::point(top_bar.bottom_left())
            .extend_positive(vec2(layout_size * 5.0, top_bar.height()));
        let tabs_pos = tab.stack(vec2(tab.width() + layout_size, 0.0), tabs.len());
        for (tab, pos) in tabs.into_iter().zip(tabs_pos) {
            tab.update(pos, context);
        }

        if self.tab_edit.text.state.clicked {
            self.edit.show();
            self.config.hide();
        } else if self.tab_config.text.state.clicked {
            self.edit.hide();
            self.config.show();
        }

        let save = top_bar.cut_right(layout_size * 5.0);
        self.save.update(save, context);
        if self.save.text.state.clicked {
            actions.push(EditorAction::Save.into());
        }

        let unsaved = top_bar.cut_right(layout_size * 10.0);
        if editor.is_changed() {
            self.unsaved.show();
            self.unsaved.update(unsaved, context);
        } else {
            self.unsaved.hide();
        }

        let main = main.extend_down(-layout_size);
        let mut state = (editor, actions);
        if self.edit.state.visible {
            self.edit.update(main, context, &mut state);
        }
        if self.config.state.visible {
            self.config
                .update(main.extend_up(-3.0 * layout_size), context, &mut state);
        }

        (context.can_focus, state.1)
    }
}
