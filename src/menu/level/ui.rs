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
        Self::new_sized(icon, icon.size.as_f32())
    }

    pub fn new_scaled(icon: &'a Icon, scale: vec2<f32>) -> Self {
        Self::new_sized(icon, icon.size.as_f32() * scale)
    }

    pub fn new_sized(icon: &'a Icon, size: vec2<f32>) -> Self {
        Self {
            icon,
            size,
            with_frame: false,
        }
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
                let icon = IconButton::new_scaled(&ctx.icons.level_frame, vec2::splat(5.5));
                let _response = ui.add(GroupWidget::new(group, icon));
                ui.add_space(30.0);
            }
        });
    }
}

impl LevelMenu {
    pub fn ui(&mut self, _delta_time: Time) {
        let ctx = self.ui.get_context();

        // ctx.style_mut(|style| {
        //     for font_id in &mut style.text_styles.values_mut() {
        //         font_id.size = 12.0;
        //     }
        // });

        egui::TopBottomPanel::top("title").show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(50.0);
                ui.add(IconButton::new_scaled(
                    &self.state.icons.title,
                    vec2::splat(2.0),
                ));
                ui.add_space(100.0);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            GroupsComponent::add(&mut self.state, ui);
        });
    }
}
