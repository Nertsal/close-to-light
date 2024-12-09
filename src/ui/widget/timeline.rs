use super::*;

use crate::{
    editor::{LevelAction, LevelEditor, LightId},
    prelude::*,
    ui::UiState,
};

use std::collections::{BTreeMap, BTreeSet};

pub struct TimelineWidget {
    context: UiContext,
    pub state: WidgetState,

    pub lights: BTreeMap<Time, Vec<TimelineLight>>,
    pub mainline: MainBar,
    pub selected_line: SelectedBar,

    pub current: WidgetState,

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

pub struct TimelineLight {
    pub id: LightId,
    pub state: WidgetState,
    pub waypoints: Option<TimelineWaypoints>,
}

pub struct TimelineWaypoints {
    pub min: Time,
    pub max: Time,
    pub points: Vec<(WaypointId, WidgetState)>,
}

#[derive(Default)]
pub struct MainBar {
    pub state: WidgetState,
    pub bar: WidgetState,
    pub ticks: BTreeSet<Time>,
}

#[derive(Default)]
pub struct SelectedBar {
    pub state: WidgetState,
    pub bar: WidgetState,
    pub points: BTreeMap<Time, (WaypointId, WidgetState)>,
}

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

            lights: BTreeMap::new(),
            mainline: default(),
            selected_line: default(),

            current: default(),

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
        let current = self.current.position.center().x;
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
        self.reload(None);

        // Auto scroll if current time goes off screen
        let margin = 50.0;
        let margin_time = margin / self.scale;

        let min = margin;
        let max = self.state.position.width() - margin;
        let current = self.current.position.center().x - self.state.position.min.x;
        if current < min && self.raw_current_time as f32 > margin_time {
            self.scroll(((min - current) / self.scale) as Time);
        } else if current > max {
            self.scroll(((max - current) / self.scale) as Time);
        }
        self.reload(None);
    }

    // fn reload2(&mut self,)

    fn reload(&mut self, mut editor: Option<(&LevelEditor, &mut Vec<LevelAction>)>) {
        let render_time = |time: Time| {
            let pos = (time + self.scroll) as f32 * self.scale;
            let pos = vec2(
                self.state.position.min.x + pos,
                self.state.position.center().y,
            );
            Aabb2::point(pos).extend_symmetric(vec2(0.1, 0.5) * self.context.font_size / 2.0)
        };

        let current = render_time(self.raw_current_time);

        self.current.update(current, &self.context);

        let height = self.context.font_size * 0.4;
        // for (i, event) in self.level.events.iter().enumerate() {
        //     if let Event::Light(light) = &event.event {
        //         let time = event.time;
        //         let light_id = LightId { event: i };
        //         if Some(light_id) == self.selected_light {
        //             let from = render_time(time).center();
        //             let to = render_time(time + light.movement.total_duration())
        //                 .center()
        //                 .x;
        //             let selected = Aabb2::point(from)
        //                 .extend_right(to - from.x)
        //                 .extend_symmetric(vec2(0.0, 0.1) * self.context.font_size / 2.0);
        //             self.selected.show();
        //             self.selected.update(selected, &self.context);

        //             let size = vec2(0.25, 0.5) * self.context.font_size;
        //             for (waypoint_id, _, offset) in light.movement.timed_positions() {
        //                 let mut state = WidgetState::new();
        //                 let position = render_time(time + offset).center();
        //                 let position = Aabb2::point(position).extend_symmetric(size / 2.0);
        //                 state.update(position, &self.context);
        //                 if state.clicked {
        //                     if let Some((_editor, actions)) = &mut editor {
        //                         actions.extend([
        //                             LevelAction::SelectLight(light_id),
        //                             LevelAction::SelectWaypoint(waypoint_id),
        //                         ]);
        //                     }
        //                 }
        //                 self.waypoints.push((waypoint_id, state));
        //             }
        //         }

        //         let lights = self.lights.entry(time).or_default();

        //         let mut state = WidgetState::new();
        //         let position = render_time(time).center();
        //         let position = Aabb2::point(position)
        //             .extend_symmetric(vec2(height, 0.0) / 2.0)
        //             .extend_down(height)
        //             .translate(-vec2(0.0, height * (lights.len() as f32 + 0.2)));
        //         state.update(position, &self.context);
        //         if state.clicked {
        //             if let Some((_editor, actions)) = &mut editor {
        //                 actions.extend([LevelAction::SelectLight(light_id)]);
        //             }
        //         }

        //         lights.push((light_id, state));
        //     }
        // }

        let render_option = |widget: &mut WidgetState, time: Option<Time>| match time {
            Some(time) => {
                widget.show();
                widget.update(render_time(time), &self.context);
            }
            None => widget.hide(),
        };
    }

    pub fn get_cursor_time(&self) -> Time {
        ((self.context.cursor.position.x - self.state.position.min.x) / self.scale) as Time
            - self.scroll
    }
}

impl StatefulWidget for TimelineWidget {
    type State<'a> = (&'a LevelEditor, Vec<LevelAction>);

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
        (state, actions): &mut Self::State<'_>,
    ) {
        self.state.update(position, context);

        let clickable =
            position.extend_symmetric(vec2(0.0, context.font_size * 0.2 - position.height()) / 2.0);
        let bar = Aabb2::point(clickable.center())
            .extend_symmetric(vec2(clickable.width(), context.font_size * 0.1) / 2.0);
        self.mainline.state.update(clickable, context);
        self.mainline.bar.update(bar, context);

        self.context = context.clone();
        self.level = state.level.clone();
        self.selected_light = state.selected_light;
        self.selected_waypoint = state
            .level_state
            .waypoints
            .as_ref()
            .and_then(|waypoints| waypoints.selected);
        self.reload(Some((state, actions)));
    }
}
