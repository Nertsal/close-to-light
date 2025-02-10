mod config;
mod edit;
mod widgets;

pub(super) use self::widgets::*;
pub use self::{config::*, edit::*};

use super::*;

use crate::{
    simple_widget_state,
    ui::{layout::AreaOps, widget::*},
};

const HELP: &str = "
Scroll / Arrow keys - move through time
Hold Shift / Alt - scroll slower / faster
Space - play music
Q / E - rotate
Ctrl+Scroll - scale lights
F1 - Hide UI
";

pub struct EditorUi {
    pub game: WidgetState,
    pub edit: EditorEditUi,
    pub config: EditorConfigUi,

    pub context_menu: ContextMenuWidget,
    pub confirm: Option<ConfirmWidget>,
}

impl EditorUi {
    pub fn new(_context: Context) -> Self {
        Self {
            game: default(),
            edit: EditorEditUi::new(),
            config: EditorConfigUi::new(),
            context_menu: ContextMenuWidget::empty(), // TODO: persistent widget in UI state
            confirm: None,                            // TODO: persistent widget in UI state
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

        // self.screen.update(screen, context);

        self.context_menu.update(&mut actions, context);

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

        let exit = top_bar.cut_right(layout_size * 3.0);
        let button = context
            .state
            .get_root_or(|| ButtonWidget::new("Exit").color(ThemeColor::Danger));
        button.update(exit, context);
        if button.text.state.clicked {
            if editor.is_changed() {
                actions.push(
                    EditorAction::PopupConfirm(
                        ConfirmAction::ExitUnsaved,
                        "unsaved changes will be lost".into(),
                    )
                    .into(),
                );
            } else {
                actions.push(EditorStateAction::Exit);
            }
        }

        let save = top_bar.cut_left(layout_size * 4.0);
        let button = context
            .state
            .get_root_or(|| ButtonWidget::new("Save").color(ThemeColor::Highlight));
        button.update(save, context);
        if button.text.state.clicked {
            actions.push(EditorAction::Save.into());
        }
        top_bar.cut_left(layout_size);

        // let help = top_bar.cut_left(layout_size * 3.0);
        // self.help.update(help, context);

        // let help_text = Aabb2::point(help.bottom_right())
        //     .extend_right(layout_size * 12.0)
        //     .extend_down(font_size * HELP.lines().count() as f32);
        // self.help_text.update(help_text, context);
        // context.update_focus(self.help_text.state.hovered);
        // if self.help.state.hovered {
        //     self.help_text.show();
        // } else if !self.help_text.state.hovered
        //     && !Aabb2::from_corners(
        //         self.help.state.position.top_left(),
        //         self.help_text.state.position.bottom_right(),
        //     )
        //     .contains(context.cursor.position)
        // {
        //     self.help_text.hide();
        // }

        let tab_edit = context
            .state
            .get_root_or(|| ToggleButtonWidget::new("Edit"));
        tab_edit.selected = matches!(editor.tab, EditorTab::Edit);
        tab_edit.update(top_bar.cut_left(layout_size * 5.0), context);
        top_bar.cut_left(layout_size);

        let tab_config = context
            .state
            .get_root_or(|| ToggleButtonWidget::new("Config"));
        tab_config.selected = matches!(editor.tab, EditorTab::Config);
        tab_config.update(top_bar.cut_left(layout_size * 5.0), context);
        top_bar.cut_left(layout_size);

        if tab_edit.text.state.clicked {
            actions.push(EditorAction::SwitchTab(EditorTab::Edit).into())
        } else if tab_config.text.state.clicked {
            actions.push(EditorAction::SwitchTab(EditorTab::Config).into());
        }

        let unsaved = top_bar.cut_right(font_size * 10.0);
        if editor.is_changed() {
            let text = context
                .state
                .get_root_or(|| TextWidget::new("Save to apply changes").aligned(vec2(1.0, 0.5)));
            text.update(unsaved, context);
        }

        let main = main.extend_down(-layout_size);
        match editor.tab {
            EditorTab::Edit => {
                self.edit
                    .layout(main, self.game.position, context, editor, &mut actions);
            }
            EditorTab::Config => {
                self.config.layout(
                    main.extend_up(-3.0 * layout_size),
                    context,
                    editor,
                    &mut actions,
                );
            }
        }

        (context.can_focus(), actions)
    }
}
