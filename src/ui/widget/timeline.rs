use std::collections::BTreeSet;

use super::*;

use crate::{
    assets::PixelTexture,
    editor::{LevelAction, LevelEditor, LightId},
    prelude::*,
    ui::{layout::AreaOps, UiState},
};

/// Pixels per unit
const PPU: usize = 2;

pub struct TimelineWidget {
    context: UiContext,
    pub state: WidgetState,
    pub ceiling: WidgetState,
    pub extra_line: WidgetState,
    pub lights_line: WidgetState,
    pub main_line: WidgetState,
    pub highlight_line: WidgetState,

    // pub lights: BTreeMap<Time, Vec<TimelineLight>>,
    // pub mainline: MainBar,
    // pub selected_line: SelectedBar,

    // pub current: WidgetState,

    // pub lights: BTreeMap<Time, Vec<(LightId, WidgetState)>>,
    // pub waypoints: Vec<(WaypointId, WidgetState)>,
    /// Render scale in pixels per beat.
    scale: f32,
    /// The scrolloff in exact time.
    scroll: Time,
    raw_current_time: Time,
    level: Level, // TODO: reuse existing
    selected_light: Option<LightId>,
    selected_waypoint: Option<WaypointId>,
}

// pub struct TimelineLight {
//     pub id: LightId,
//     pub state: WidgetState,
//     pub waypoints: Option<TimelineWaypoints>,
// }

// pub struct TimelineWaypoints {
//     pub min: Time,
//     pub max: Time,
//     pub points: Vec<(WaypointId, WidgetState)>,
// }

// #[derive(Default)]
// pub struct MainBar {
//     pub state: WidgetState,
//     pub bar: WidgetState,
//     pub ticks: BTreeSet<Time>,
// }

// #[derive(Default)]
// pub struct SelectedBar {
//     pub state: WidgetState,
//     pub bar: WidgetState,
//     pub points: BTreeMap<Time, (WaypointId, WidgetState)>,
// }

impl TimelineWidget {
    pub fn new(context: Context) -> Self {
        Self {
            context: UiContext {
                state: UiState::new(),
                geometry: crate::ui::geometry::GeometryContext::new(context.assets.clone()),
                font: context.geng.default_font().clone(),
                screen: Aabb2::ZERO.extend_positive(vec2(1.0, 1.0)),
                layout_size: 1.0,
                font_size: 1.0,
                can_focus: true.into(),
                cursor: CursorContext::new(),
                delta_time: 0.1,
                mods: KeyModifiers::default(),
                text_edit: TextEdit::empty(),
                context,
            },
            state: default(),
            ceiling: default(),
            extra_line: default(),
            lights_line: default(),
            main_line: default(),
            highlight_line: default(),

            // lights: BTreeMap::new(),
            // mainline: default(),
            // selected_line: default(),

            // current: default(),
            scale: 0.5,
            scroll: Time::ZERO,
            raw_current_time: Time::ZERO,
            level: Level::new(),
            selected_light: None,
            selected_waypoint: None,
        }
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
    }

    pub fn rescale(&mut self, new_scale: f32) {
        if new_scale.approx_eq(&0.0) {
            return;
        }

        // scroll so that current beat stays in-place
        let min = self.state.position.min.x;
        let current = self.main_line.position.center().x;
        self.scroll = ((current - min) / new_scale) as Time - self.raw_current_time;

        self.scale = new_scale;
        self.reload(None);
    }

    // pub fn auto_scale(&mut self, max_beat: Time) {
    //     let scale = self.state.position.width() / max_beat.as_f32().max(1.0);
    //     self.scale = scale;
    //     self.reload();
    // }

    pub fn visible_scroll(&self) -> Time {
        (self.state.position.width() / self.scale) as Time
    }

    pub fn get_scroll(&self) -> Time {
        self.scroll
    }

    pub fn scroll(&mut self, delta: Time) {
        self.scroll += delta;
        self.reload(None);
    }

    pub fn update_time(&mut self, current_beat: Time) {
        self.raw_current_time = current_beat;
        self.scroll = -current_beat;
        self.reload(None);
    }

    fn reload(&mut self, mut editor: Option<(&LevelEditor, &mut Vec<LevelAction>)>) {
        let sprites = &self.context.context.assets.sprites.timeline;

        let render_time = |line: &WidgetState, time: Time| {
            let size = vec2::splat(18) * PPU;
            let pos = (time + self.scroll) as f32 * self.scale;
            let pos = line.position.center() + vec2(pos, 0.0);
            geng_utils::pixel::pixel_perfect_aabb(
                pos,
                vec2(0.5, 0.5),
                size,
                &geng::PixelPerfectCamera,
                self.context.geometry.framebuffer_size.as_f32(),
            )
        };

        // let height = self.context.font_size * 0.4;
        // Render events on the timeline
        let mut occupied = BTreeSet::new();
        for (i, event) in self.level.events.iter().enumerate() {
            if let Event::Light(light_event) = &event.event {
                let time = event.time + light_event.movement.fade_in;
                let light_id = LightId { event: i };
                let is_selected = Some(light_id) == self.selected_light;

                // Selected light waypoints
                // if is_selected {
                //     let from = render_time(time).center();
                //     let to = render_time(time + light.movement.total_duration())
                //         .center()
                //         .x;
                //     let selected = Aabb2::point(from)
                //         .extend_right(to - from.x)
                //         .extend_symmetric(vec2(0.0, 0.1) * self.context.font_size / 2.0);
                //     self.selected.show();
                //     self.selected.update(selected, &self.context);

                //     let size = vec2(0.25, 0.5) * self.context.font_size;
                //     for (waypoint_id, _, offset) in light.movement.timed_positions() {
                //         let mut state = WidgetState::new();
                //         let position = render_time(time + offset).center();
                //         let position = Aabb2::point(position).extend_symmetric(size / 2.0);
                //         state.update(position, &self.context);
                //         if state.clicked {
                //             if let Some((_editor, actions)) = &mut editor {
                //                 actions.extend([
                //                     LevelAction::SelectLight(light_id),
                //                     LevelAction::SelectWaypoint(waypoint_id),
                //                 ]);
                //             }
                //         }
                //         self.waypoints.push((waypoint_id, state));
                //     }
                // }

                // Light icon
                if occupied.insert(time) {
                    let light = render_time(&self.lights_line, time);
                    let texture = match light_event.shape {
                        Shape::Circle { .. } => &sprites.circle,
                        Shape::Line { .. } => &sprites.square,
                        Shape::Rectangle { .. } => &sprites.square,
                    };
                    let icon = self.context.state.get_or(|| IconButtonWidget::new(texture));
                    icon.update(light, &self.context);
                    icon.icon.color = if is_selected {
                        ThemeColor::Highlight
                    } else if light_event.danger {
                        ThemeColor::Danger
                    } else {
                        ThemeColor::Light
                    };
                    icon.icon.texture = texture.clone();
                    if icon.state.clicked {
                        if let Some((_editor, actions)) = &mut editor {
                            actions.extend([LevelAction::SelectLight(light_id)]);
                        }
                    }
                } else {
                    // Dots to indicate there are more light in that position
                    let dots = render_time(&self.extra_line, time);
                    let texture = &sprites.dots;
                    let icon = self.context.state.get_or(|| IconWidget::new(texture));
                    icon.update(dots, &self.context);
                }
            }
        }
    }

    pub fn get_cursor_time(&self) -> Time {
        ((self.context.cursor.position.x - self.state.position.min.x) / self.scale) as Time
            - self.scroll
    }

    pub fn update(
        &mut self,
        mut position: Aabb2<f32>,
        context: &UiContext,
        state: &LevelEditor,
        actions: &mut Vec<LevelAction>,
    ) {
        self.state.update(position, context);
        let pixel = PPU as f32;

        let ceiling = position.cut_top(pixel * 5.0);
        self.ceiling.update(ceiling, context);

        let extra = position.cut_top(pixel * 3.0);
        self.extra_line.update(extra, context);
        position.cut_top(pixel * 3.0);

        let lights = position.cut_top(pixel * 16.0);
        self.lights_line.update(lights, context);
        position.cut_top(pixel * 4.0);

        // let clickable =
        //     position.extend_symmetric(vec2(0.0, context.font_size * 0.2 - position.height()) / 2.0);
        // let bar = Aabb2::point(clickable.center())
        //     .extend_symmetric(vec2(clickable.width(), context.font_size * 0.1) / 2.0);
        let main = position.cut_top(pixel * 16.0);
        self.main_line.update(main, context);
        position.cut_top(pixel * 4.0);

        let highlight = position.cut_top(pixel * 16.0);
        self.highlight_line.update(highlight, context);

        self.context = context.clone();
        self.level = state.level.clone();
        self.selected_light = state.selected_light;
        self.selected_waypoint = state
            .level_state
            .waypoints
            .as_ref()
            .and_then(|waypoints| waypoints.selected);

        // if self.main_line.hovered {
        //     let scroll = self.context.cursor.scroll;
        //     if context.mods.shift {
        //         self.scroll(scroll);
        //     } else if context.mods.ctrl {
        //         self.rescale(self.scale + scroll);
        //     }
        // }

        self.reload(Some((state, actions)));
    }
}

// TODO: move to layout (we have scroll in context)
// if shift && self.ui.edit.timeline.state.hovered {
//     actions.push(EditorStateAction::TimelineScroll(scroll));
// } else if ctrl {
//     if self.ui.edit.timeline.state.hovered {
//         // Zoom on the timeline
//         actions.push(EditorStateAction::TimelineZoom(scroll));
//     } else if let State::Place { .. }
//     | State::Waypoints {
//         state: WaypointsState::New,
//         ..
//     } = level_editor.state
//     {
//         // Scale light or waypoint placement
//         let delta = scroll * r32(0.1);
//         actions.push(LevelAction::ScalePlacement(delta).into());
//     } else if let Some(waypoints) = &level_editor.level_state.waypoints {
//         if let Some(selected) = waypoints.selected {
//             let delta = scroll * r32(0.1);
//             actions.push(
//                 LevelAction::ScaleWaypoint(waypoints.light, selected, delta)
//                     .into(),
//             );
//         }
//     } else if let Some(id) = level_editor.selected_light {
//         // Control fade time
//         let scroll = scroll.as_f32() as Time;
//         let change = scroll * self.editor.config.scroll_slow.as_time(beat_time);
//         let action = if shift {
//             LevelAction::ChangeFadeOut(id, Change::Add(change))
//         } else {
//             LevelAction::ChangeFadeIn(id, Change::Add(change))
//         };
//         actions.push(action.into());
//     }

impl Widget for TimelineWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let pixel_scale = PPU;
        let pixel = pixel_scale as f32;
        let theme = context.theme();
        let sprites = &context.context.assets.sprites.timeline;

        let mut geometry = Geometry::new();

        {
            let texture = &sprites.current_arrow;
            let size = texture.size() * pixel_scale;
            let position = geng_utils::pixel::pixel_perfect_aabb(
                self.ceiling.position.align_pos(vec2(0.5, 1.0)),
                vec2(0.5, 1.0),
                size,
                &geng::PixelPerfectCamera,
                context.geometry.framebuffer_size.as_f32(),
            );

            geometry.merge(context.geometry.texture(
                position,
                mat3::identity(),
                theme.highlight,
                texture,
            ));
        }

        // geometry.merge(context.geometry.texture_pp_at(
        //     self.extra_line.position.center(),
        //     theme.light,
        //     pixel_scale,
        //     &sprites.dots,
        // ));

        // geometry.merge(context.geometry.texture_pp_at(
        //     self.lights_line.position.center(),
        //     theme.light,
        //     pixel_scale,
        //     &sprites.circle,
        // ));
        // geometry.merge(context.geometry.texture_pp_at(
        //     self.lights_line.position.center(),
        //     theme.highlight,
        //     pixel_scale,
        //     &sprites.circle_fill,
        // ));

        // geometry.merge(context.geometry.texture_pp_at(
        //     self.main_line.position.center(),
        //     theme.highlight,
        //     pixel_scale,
        //     &sprites.tick_big,
        // ));

        let main_bar = self.main_line.position;
        let main_bar = main_bar.align_aabb(vec2(main_bar.width(), pixel * 4.0), vec2(0.5, 0.5));
        geometry.merge(context.geometry.quad(main_bar, theme.light));

        geometry
    }
}

#[derive(Clone)]
struct IconWidget {
    state: WidgetState,
    texture: PixelTexture,
    color: ThemeColor,
}

impl IconWidget {
    pub fn new(texture: &PixelTexture) -> Self {
        Self {
            state: default(),
            texture: texture.clone(),
            color: ThemeColor::Light,
        }
    }
    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }
}

impl Widget for IconWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let pixel_scale = PPU;
        let theme = context.theme();
        let mut geometry = Geometry::new();

        let fg_color = theme.get_color(self.color);

        geometry.merge(context.geometry.texture_pp_at(
            self.state.position.center(),
            fg_color,
            pixel_scale,
            &self.texture,
        ));

        geometry
    }
}

#[derive(Clone)]
struct IconButtonWidget {
    state: WidgetState,
    icon: IconWidget,
}

impl IconButtonWidget {
    pub fn new(texture: &PixelTexture) -> Self {
        Self {
            state: WidgetState::new(),
            icon: IconWidget::new(texture),
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }
}

impl Widget for IconButtonWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let pixel_scale = PPU;
        let theme = context.theme();
        let outline_width = pixel_scale as f32 * 3.0;
        let mut geometry = Geometry::new();

        let mut fg_color = theme.get_color(self.icon.color);
        let mut bg_color = theme.dark;
        if self.state.hovered {
            std::mem::swap(&mut fg_color, &mut bg_color);
        }

        geometry.merge(context.geometry.texture_pp_at(
            self.state.position.center(),
            fg_color,
            pixel_scale,
            &self.icon.texture,
        ));
        geometry.merge(
            context
                .geometry
                .quad_fill(self.state.position.extend_uniform(outline_width), bg_color),
        );

        geometry
    }
}
