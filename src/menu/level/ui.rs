use super::*;

use egui::Widget;
use geng_egui::{
    egui::{Align, Frame, Label, Layout, RichText, Ui},
    *,
};

pub trait AppComponent {
    type Context;

    #[allow(unused)]
    fn add(ctx: &mut Self::Context, ui: &mut Ui) {}
}

pub struct IconButton<'a> {
    icon: &'a Icon,
    size: vec2<f32>,
    with_frame: bool,
}

impl<'a> IconButton<'a> {
    pub fn new(icon: &'a Icon) -> Self {
        Self {
            icon,
            size: icon.size.as_f32(),
            with_frame: false,
        }
    }

    pub fn with_height(self, height: f32) -> Self {
        let width = self.icon.size.as_f32().aspect() * height;
        Self {
            size: vec2(width, height),
            ..self
        }
    }

    pub fn scaled(self, scale: vec2<f32>) -> Self {
        Self {
            size: self.size * scale,
            ..self
        }
    }

    pub fn sized(self, size: vec2<f32>) -> Self {
        Self { size, ..self }
    }
}

impl<'a> Widget for IconButton<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let size = egui::Vec2 {
            x: self.size.x,
            y: self.size.y,
        };

        let image = egui::ImageSource::Texture(egui::load::SizedTexture {
            id: self.icon.id(),
            size,
        });
        let button = egui::ImageButton::new(image).frame(self.with_frame);
        let response = ui.add(button);
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    }
}

pub struct GroupWidget<'a> {
    group: &'a GroupEntry,
    icon: IconButton<'a>,
}

impl<'a> GroupWidget<'a> {
    pub fn new(group: &'a GroupEntry, icon: IconButton<'a>) -> Self {
        Self { group, icon }
    }
}

impl<'a> Widget for GroupWidget<'a> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let icon = ui.add(self.icon);

        let rect = icon.rect.shrink(15.0);
        ui.allocate_ui_at_rect(rect, |ui| {
            ui.with_layout(
                Layout::top_down(Align::Min).with_main_align(Align::Center),
                |ui| {
                    ui.label(RichText::new(self.group.meta.name.to_string()).size(30.0));
                    let author = format!("by {}", self.group.meta.music.author);
                    ui.label(RichText::new(author).size(20.0));
                },
            );
        });

        icon
    }
}

pub struct GroupsComponent;

impl AppComponent for GroupsComponent {
    type Context = MenuState;

    fn add(ctx: &mut Self::Context, ui: &mut Ui) {
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            for group in &ctx.groups {
                let icon = IconButton::new(&ctx.icons.level_frame).with_height(ctx.font_size * 3.5);
                let _response = ui.add(GroupWidget::new(group, icon));
                ui.add_space(ctx.font_size * 2.5);
            }
        });
    }
}

impl LevelMenu {
    pub fn ui(&mut self, _delta_time: Time) {
        let ui_ctx = self.ui.get_context();
        let ctx = &mut self.state;

        egui::TopBottomPanel::top("title").show(ui_ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(ctx.font_size * 2.0);
                ui.add(IconButton::new(&ctx.icons.title).with_height(ctx.font_size * 5.0));
                ui.add_space(ctx.font_size * 3.0);
            });
        });

        egui::SidePanel::left("left-panel")
            .show_separator_line(false)
            .resizable(false)
            .show(ui_ctx, |ui| {
                ui.add_space(ctx.font_size * 1.0);
            });

        egui::SidePanel::right("right-panel")
            .show_separator_line(false)
            .resizable(false)
            .show(ui_ctx, |ui| {
                ui.add_space(ctx.font_size * 1.0);
            });

        egui::CentralPanel::default().show(ui_ctx, |ui| {
            GroupsComponent::add(ctx, ui);
        });
    }
}
