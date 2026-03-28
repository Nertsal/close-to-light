use super::*;

use crate::{Change, EditorAction, HistoryLabel, LevelAction, LevelEditor, LightId, ScrollSpeed};

use std::collections::BTreeMap;

use ctl_render_core::SubTexture;
use ctl_util::SecondOrderState;
use num_rational::Ratio;

/// Pixels per unit
const PPU: usize = 2;
const LIGHT_LINE_WIDTH: f32 = 16.0;
const LIGHT_LINE_SPACE: f32 = 4.0;

// TODO: unmagic constant - max click shake distance and duration
const MAX_CLICK_DISTANCE: f32 = 25.0;
const MAX_CLICK_DURATION: f32 = 0.5;

pub struct TimelineWidget {
    cursor_pos: vec2<f32>,
    expansion: SecondOrderState<f32>,
    /// The whole allocated position including the side panels.
    pub allocated_position: WidgetState,
    /// Interactive position of the moving timeline itself.
    pub state: WidgetState,
    pub ceiling: WidgetState,
    pub extra_line: WidgetState,
    pub lights_line: WidgetState,
    pub main_line: WidgetState,
    pub highlight_line: WidgetState,
    highlight_bar: Option<HighlightBar>,
    dots: Vec<vec2<f32>>,
    marks: Vec<(vec2<f32>, Color)>,
    /// Ticks with position and subdivision indicator used to select color and texture.
    ticks: Vec<(vec2<f32>, i64)>,

    /// Render scale in pixels per beat.
    scale: f32,
    /// The scrolloff in exact time.
    scroll: Time,
    raw_current_time: Time,
    raw_target_time: Time,
}

struct HighlightBar {
    from_time: Time,
    from: vec2<f32>,
    to_time: Time,
    to: vec2<f32>,
}

impl Default for TimelineWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineWidget {
    pub fn new() -> Self {
        Self {
            cursor_pos: vec2::ZERO,
            expansion: SecondOrderState::new(3.0, 1.0, 1.0, 0.0),
            allocated_position: default(),
            state: default(),
            ceiling: default(),
            extra_line: default(),
            lights_line: default(),
            main_line: default(),
            highlight_line: default(),
            highlight_bar: None,
            dots: Vec::new(),
            marks: Vec::new(),
            ticks: Vec::new(),

            scale: 0.5,
            scroll: Time::ZERO,
            raw_current_time: Time::ZERO,
            raw_target_time: Time::ZERO,
        }
    }

    pub fn rescale(&mut self, new_scale: f32) {
        if new_scale.approx_eq(&0.0) {
            return;
        }

        self.scale = new_scale;
    }

    // pub fn auto_scale(&mut self, max_beat: Time) {
    //     let scale = self.state.position.width() / max_beat.as_f32().max(1.0);
    //     self.scale = scale;
    // }

    pub fn visible_scroll(&self) -> Time {
        (self.state.position.width() / self.scale) as Time
    }

    pub fn update_time(&mut self, current_beat: Time, target_beat: Time) {
        self.raw_current_time = current_beat;
        self.raw_target_time = target_beat;
        self.scroll = -current_beat;
    }

    fn reload(
        &mut self,
        context: &UiContext,
        editor: &Editor,
        level_editor: &LevelEditor,
        actions: &mut Vec<EditorStateAction>,
    ) {
        let atlas = &context.context.assets.atlas;
        let theme = context.theme();

        // Selection mode for clicking on the icons on the timeline
        let selection_mode = if context.mods.shift {
            SelectMode::Toggle
        } else {
            SelectMode::Set
        };
        let multi_select_mode = context.mods.shift;

        let enable_beat_snap = !context.mods.ctrl;
        let beat_snap = level_editor.beat_snap;

        // from time to screen position
        let render_at = |center: vec2<f32>, time: Time| {
            let size = vec2::splat(18) * PPU;
            let pos = (time + self.scroll) as f32 * self.scale;
            let pos = center + vec2(pos, 0.0);
            geng_utils::pixel::pixel_perfect_aabb(
                pos,
                vec2(0.5, 0.5),
                size,
                &geng::PixelPerfectCamera,
                context.geometry.framebuffer_size.as_f32(),
            )
        };
        let render_time = |line: &WidgetState, time: Time| render_at(line.position.center(), time);
        let render_light = |time: Time, i: usize| {
            let pos = self.lights_line.position.align_pos(vec2(0.5, 0.0))
                + vec2(
                    0.0,
                    (LIGHT_LINE_WIDTH * 0.5 + ((LIGHT_LINE_WIDTH + LIGHT_LINE_SPACE) * i as f32))
                        * PPU as f32,
                );
            render_at(pos, time)
        };

        // from screen position to time
        let unrender_time = |pos: f32| {
            ((pos - self.main_line.position.center().x) / self.scale).round() as Time - self.scroll
        };

        // Check highlight bounds
        let light_selection = level_editor
            .selection
            .light_single()
            .and_then(|id| level_editor.level.events.get(id.event))
            .and_then(|event| {
                if let Event::Light(light) = &event.event {
                    let from_time = event.time;
                    let from = render_time(&self.highlight_line, from_time).center();
                    let to_time = event.time + light.movement.duration();
                    let to = render_time(&self.highlight_line, to_time).center();
                    Some(HighlightBar {
                        from_time,
                        from,
                        to_time,
                        to,
                    })
                } else {
                    None
                }
            });
        self.highlight_bar = light_selection.or_else(|| {
            level_editor
                .selection
                .single()
                .and_then(|id| {
                    if let EditorEventIdx::Event(id) = id {
                        level_editor.level.events.get(id)
                    } else {
                        None
                    }
                })
                .and_then(|event| {
                    let duration = match &event.event {
                        Event::Light(_) => return None,
                        Event::Effect(effect) => match effect {
                            EffectEvent::PaletteSwap(duration)
                            | EffectEvent::RgbSplit(duration)
                            | EffectEvent::CameraShake(duration, _) => duration,
                        },
                    };
                    let from_time = event.time;
                    let from = render_time(&self.highlight_line, from_time).center();
                    let to_time = event.time + duration;
                    let to = render_time(&self.highlight_line, to_time).center();
                    Some(HighlightBar {
                        from_time,
                        from,
                        to_time,
                        to,
                    })
                })
        });

        let timeline_tick =
            |size: vec2<f32>,
             pos: vec2<f32>,
             texture: SubTexture,
             actions: &mut Vec<EditorStateAction>,
             while_pressed: &mut dyn FnMut(&mut Vec<EditorStateAction>, Time),
             on_release: &mut dyn FnMut(&mut Vec<EditorStateAction>)| {
                if !self.state.position.contains(pos) {
                    return;
                }
                let position = Aabb2::point(pos).extend_symmetric(size / 2.0);
                let tick = context.state.get_or(self.state.id, || {
                    // TODO: somehow mask this with other stuff
                    IconButtonWidget::new(texture)
                        .highlight(HighlightMode::Color(ThemeColor::Highlight))
                });
                tick.update(position, context);
                if tick.state.mouse_left.pressed.is_some() {
                    let mut target = unrender_time(context.cursor.position.x);
                    target = level_editor.level.timing.snap_to_beat(target, beat_snap);
                    while_pressed(actions, target);
                }
                if tick.state.mouse_left.just_released {
                    on_release(actions);
                }
            };

        // Render events on the timeline
        let mut occupied = BTreeMap::new();
        self.dots.clear();
        let focus = {
            let mut focus = context.can_focus.borrow_mut();
            let f = *focus;
            *focus = self.state.hovered;
            f
        };
        let can_focus = context.can_focus();
        let visible_scroll = self.visible_scroll();

        let regular_event = |event_i: EditorEventIdx,
                             event_time: Time,
                             event_duration: Time,
                             texture: SubTexture,
                             actions: &mut Vec<EditorStateAction>,
                             occupied: &mut BTreeMap<i64, usize>,
                             dots: &mut Vec<vec2<f32>>| {
            let is_selected = level_editor.selection.is_selected(event_i);

            let on_top_of_highlight = !is_selected
                && self
                    .highlight_bar
                    .as_ref()
                    .is_some_and(|bar| (bar.from_time..=bar.to_time).contains(&event_time));
            let overlapped = *occupied
                .entry(event_time)
                .and_modify(|x| *x += 1)
                .or_insert(if on_top_of_highlight { 1 } else { 0 });

            let mut is_hovered = false;
            let visible = (event_time + self.scroll).abs() < visible_scroll / 2;
            if visible && overlapped as f32 <= self.expansion.current + 0.9 {
                let position = render_light(event_time, overlapped).center();
                let position = Aabb2::point(position).extend_uniform(5.0 * PPU as f32);
                let icon = context.state.get_or(self.state.id, || {
                    IconButtonWidget::new(atlas.timeline_metronome())
                });
                icon.texture = texture;
                icon.update(position, context);
                is_hovered = is_hovered || icon.state.hovered;
                if is_selected && !is_hovered {
                    icon.color = ThemeColor::Light;
                    icon.bg_color = ThemeColor::Highlight;
                } else {
                    icon.color = ThemeColor::Light;
                    icon.bg_color = ThemeColor::Dark;
                }
                if icon.state.mouse_left.just_pressed {
                    let ids = if is_selected {
                        level_editor.selection.to_editor_events()
                    } else {
                        vec![event_i]
                    };
                    let targets = ids
                        .into_iter()
                        .filter_map(|id| Some((id, editor_event_time(id, level_editor)?)))
                        .collect();
                    actions.extend([
                        LevelAction::SelectEvent(event_i).into(),
                        EditorStateAction::StartDrag(DragTarget::TimelineEvent {
                            initial_time: event_time,
                            targets,
                        }),
                    ]);
                }
            }

            if is_selected || is_hovered {
                // Dots
                let last_dot_time = event_time;
                let time = event_time + event_duration;

                // TODO: variable timing within this segment
                let timing = level_editor.level.timing.get_timing(event_time);

                let resolution = 4.0; // Ticks per beat
                let step = timing.beat_time / r32(resolution);
                let ds = ((time_to_seconds(time - last_dot_time) / step).as_f32() + 0.1).floor()
                    as usize;
                let overlapped = if is_selected { 0 } else { overlapped };
                let ds = (0..=ds)
                    .map(|i| {
                        let time = last_dot_time + seconds_to_time(step * r32(i as f32));
                        render_light(time, overlapped).center()
                    })
                    .filter(|&pos| self.state.position.contains(pos));

                dots.extend(ds);
            }
        };

        // Update drag
        if let Some(drag) = &editor.drag
            && let DragTarget::TimelineEvent {
                initial_time,
                targets: drag_ids,
            } = &drag.target
        {
            // Release drag
            if !can_focus || !context.cursor.left.down {
                actions.push(EditorStateAction::EndDrag);
                if (context.cursor.position - drag.from_screen).len_sqr() < MAX_CLICK_DISTANCE
                    && (context.real_time - drag.from_real_time.as_f32()).abs() < MAX_CLICK_DURATION
                {
                    // short click
                } else {
                    actions.push(LevelAction::Deselect.into());
                }
                return;
            }

            let mut cursor_time = unrender_time(context.cursor.position.x);
            if enable_beat_snap {
                cursor_time = if let Some(timing_i) = drag_ids.iter().find_map(|(id, _)| {
                    if let EditorEventIdx::Timing(i) = id {
                        Some(*i)
                    } else {
                        None
                    }
                }) {
                    level_editor
                        .level
                        .timing
                        .snap_to_beat_without(timing_i, cursor_time, beat_snap)
                } else {
                    level_editor
                        .level
                        .timing
                        .snap_to_beat(cursor_time, beat_snap)
                };
            }

            actions.push(
                LevelAction::MoveEvents(
                    drag_ids
                        .iter()
                        .map(|(event_i, event_time)| {
                            let target_time = cursor_time - initial_time + event_time;

                            (*event_i, Change::Set(target_time))
                        })
                        .collect(),
                )
                .into(),
            );
        }

        // Timing points
        for (idx, point) in level_editor.level.timing.points.iter().enumerate() {
            let idx = EditorEventIdx::Timing(idx);
            regular_event(
                idx,
                point.time,
                0,
                atlas.timeline_metronome(),
                actions,
                &mut occupied,
                &mut self.dots,
            );
        }

        // Events
        for (event_i, event) in level_editor.level.events.iter().enumerate() {
            let event_idx = EditorEventIdx::Event(event_i);
            let visible = (event.time + pre_event_time(&event.event) + self.scroll).abs()
                < self.visible_scroll() / 2;

            match &event.event {
                Event::Light(light_event) => {
                    let light_id = LightId { event: event_i };
                    let is_light_selected_single = level_editor.selection.is_light_single(light_id);
                    if is_light_selected_single {
                        let from_time = event.time;
                        let from = render_time(&self.highlight_line, from_time).center();
                        let to_time = event.time + light_event.movement.duration();
                        let to = render_time(&self.highlight_line, to_time).center();

                        let tick_size = vec2(4.0, 16.0) * PPU as f32;

                        // Fade in
                        timeline_tick(
                            tick_size,
                            from,
                            atlas.timeline_tick_smol(),
                            actions,
                            &mut |actions, target| {
                                // Drag fade in
                                let fade_in =
                                    event.time + light_event.movement.get_fade_in() - target;
                                actions.push(
                                    LevelAction::ChangeFadeIn(light_id, Change::Set(fade_in))
                                        .into(),
                                );
                            },
                            &mut |actions| {
                                actions.push(
                                    LevelAction::FlushChanges(Some(HistoryLabel::FadeIn(light_id)))
                                        .into(),
                                );
                            },
                        );

                        // Fade out
                        timeline_tick(
                            tick_size,
                            to,
                            atlas.timeline_tick_smol(),
                            actions,
                            &mut |actions, target| {
                                let fade_out =
                                    target - to_time + light_event.movement.get_fade_out();
                                actions.push(
                                    LevelAction::ChangeFadeOut(light_id, Change::Set(fade_out))
                                        .into(),
                                );
                            },
                            &mut |actions| {
                                actions.push(
                                    LevelAction::FlushChanges(Some(HistoryLabel::FadeOut(
                                        light_id,
                                    )))
                                    .into(),
                                );
                            },
                        );

                        // Waypoints
                        let last_id = WaypointId::Frame(
                            light_event.movement.waypoints.len().saturating_sub(1),
                        );
                        for (waypoint_id, _, offset) in light_event.movement.timed_transforms() {
                            let is_waypoint_selected = level_editor
                                .selection
                                .is_waypoint_selected(light_id, waypoint_id);

                            let waypoint_time = event.time + offset;
                            let position = render_light(waypoint_time, 0).center();
                            if !self.state.position.contains(position) {
                                continue;
                            }

                            // Icon
                            let position = Aabb2::point(position).extend_uniform(5.0 * PPU as f32);
                            let texture = atlas.timeline_waypoint();
                            // TODO: somehow mask this with other stuff
                            let icon = context
                                .state
                                .get_or(self.state.id, || IconButtonWidget::new(texture));
                            icon.color = if is_waypoint_selected {
                                ThemeColor::Highlight
                            } else {
                                ThemeColor::Light
                            };
                            icon.update(position, context);

                            // Tick
                            let position =
                                render_time(&self.highlight_line, event.time + offset).center();
                            let position = Aabb2::point(position).extend_symmetric(tick_size / 2.0);
                            let texture = match waypoint_id {
                                WaypointId::Initial => atlas.timeline_tick_big(),
                                WaypointId::Frame(_) if waypoint_id == last_id => {
                                    atlas.timeline_tick_mid()
                                }
                                WaypointId::Frame(_) => atlas.timeline_tick_smol(),
                                WaypointId::Last => atlas.timeline_tick_mid(),
                            };
                            let tick = context.state.get_or(self.state.id, || {
                                // TODO: somehow mask this with other stuff
                                IconButtonWidget::new(texture)
                                    .highlight(HighlightMode::Color(ThemeColor::Highlight))
                            });
                            tick.update(position, context);

                            // Waypoint drag
                            let is_dragging = if let Some(drag) = &editor.drag
                                && let DragTarget::TimelineEvent { targets, .. } = &drag.target
                                && targets.iter().any(|(id, _)| {
                                    *id == EditorEventIdx::Waypoint(light_id, waypoint_id)
                                }) {
                                true
                            } else {
                                false
                            };
                            if icon.state.mouse_left.just_pressed
                                || tick.state.mouse_left.just_pressed
                            {
                                if multi_select_mode {
                                    // Toggle selection
                                    actions.push(
                                        LevelAction::SelectWaypoint(
                                            selection_mode,
                                            light_id,
                                            vec![waypoint_id],
                                            false,
                                        )
                                        .into(),
                                    );
                                } else if is_waypoint_selected {
                                    actions.extend([EditorStateAction::StartDrag(
                                        DragTarget::TimelineEvent {
                                            initial_time: waypoint_time,
                                            targets: level_editor
                                                .selection
                                                .to_editor_events()
                                                .into_iter()
                                                .filter_map(|id| {
                                                    Some((id, editor_event_time(id, level_editor)?))
                                                })
                                                .collect(),
                                        },
                                    )]);
                                } else {
                                    let id = EditorEventIdx::Waypoint(light_id, waypoint_id);
                                    actions.extend([
                                        LevelAction::SelectWaypoint(
                                            SelectMode::Set,
                                            light_id,
                                            vec![waypoint_id],
                                            false,
                                        )
                                        .into(),
                                        EditorStateAction::StartDrag(DragTarget::TimelineEvent {
                                            initial_time: waypoint_time,
                                            targets: vec![(id, waypoint_time)],
                                        }),
                                    ]);
                                }
                            } else if !context.cursor.left.down && is_dragging {
                                actions.extend([
                                    LevelAction::FlushChanges(Some(
                                        HistoryLabel::MoveWaypointTime(light_id, waypoint_id),
                                    ))
                                    .into(),
                                    EditorStateAction::EndDrag,
                                ]);
                            }
                        }
                    }

                    let mut is_hovered = false;
                    let mut overlapped = 0;
                    let light_time = event.time + light_event.movement.get_fade_in();
                    // Idle light icon
                    if visible && !is_light_selected_single {
                        let on_top_of_highlight = self
                            .highlight_bar
                            .as_ref()
                            .is_some_and(|bar| (bar.from_time..=bar.to_time).contains(&light_time));
                        overlapped = *occupied
                            .entry(light_time)
                            .and_modify(|x| *x += 1)
                            .or_insert(if on_top_of_highlight { 1 } else { 0 });

                        // Check if there is enough visual space to render the event that high
                        if overlapped as f32 <= self.expansion.current + 0.5 {
                            let light = render_light(light_time, overlapped);
                            let texture = match light_event.shape {
                                Shape::Circle { .. } => atlas.timeline_circle(),
                                Shape::Line { .. } => atlas.timeline_square(),
                                Shape::Rectangle { .. } => atlas.timeline_square(),
                            };
                            // TODO: somehow mask this with other stuff
                            let icon = context
                                .state
                                .get_or(self.state.id, || IconButtonWidget::new(texture.clone()));
                            icon.update(light, context);
                            icon.color = if level_editor.selection.is_light_selected(light_id) {
                                ThemeColor::Highlight
                            } else if light_event.danger {
                                ThemeColor::Danger
                            } else {
                                ThemeColor::Light
                            };
                            icon.texture = texture;
                            is_hovered = is_hovered || icon.state.hovered;
                            if icon.state.hovered {
                                actions.push(LevelAction::HoverLight(light_id).into());
                            }
                            if icon.state.mouse_left.just_pressed {
                                if multi_select_mode {
                                    // Toggle selection
                                    actions.push(
                                        LevelAction::SelectLight(selection_mode, vec![light_id])
                                            .into(),
                                    );
                                } else if level_editor.selection.is_light_selected(light_id) {
                                    // Drag whole selection
                                    actions.push(EditorStateAction::StartDrag(
                                        DragTarget::TimelineEvent {
                                            initial_time: light_time,
                                            targets: level_editor
                                                .selection
                                                .to_editor_events()
                                                .into_iter()
                                                .filter_map(|id| {
                                                    Some((id, editor_event_time(id, level_editor)?))
                                                })
                                                .collect(),
                                        },
                                    ));
                                } else {
                                    // Drag single light
                                    actions.extend([
                                        LevelAction::SelectLight(SelectMode::Set, vec![light_id])
                                            .into(),
                                        EditorStateAction::StartDrag(DragTarget::TimelineEvent {
                                            initial_time: light_time,
                                            targets: vec![(event_idx, event.time)],
                                        }),
                                    ]);
                                }
                            }
                        } else {
                            // Dots to indicate there are more lights in that position
                            let dots = render_time(&self.extra_line, light_time);
                            let texture = atlas.timeline_dots();
                            // TODO: somehow mask this with other stuff
                            let icon = context
                                .state
                                .get_or(self.state.id, || IconWidget::new(texture));
                            icon.update(dots, context);
                        }
                    }
                    let is_hovered =
                        is_hovered || level_editor.level_state.hovered_light == Some(light_id);
                    if !is_light_selected_single && is_hovered {
                        // Hover preview waypoints
                        for (_, _, offset) in light_event.movement.timed_transforms() {
                            // Icon
                            let position = render_light(event.time + offset, overlapped).center();
                            if !self.state.position.contains(position) {
                                continue;
                            }

                            let position = Aabb2::point(position).extend_uniform(5.0 * PPU as f32);
                            let texture = atlas.timeline_waypoint();
                            // TODO: somehow mask this with other stuff
                            let icon = context
                                .state
                                .get_or(self.state.id, || IconButtonWidget::new(texture));
                            icon.color = ThemeColor::Light;
                            icon.update(position, context);
                        }
                    }
                    if is_light_selected_single || is_hovered {
                        // Dots
                        let last_dot_time = event.time;
                        let time = event.time + light_event.movement.duration();

                        // TODO: variable timing within this segment
                        let timing = level_editor.level.timing.get_timing(event.time);

                        let resolution = 4.0; // Ticks per beat
                        let step = timing.beat_time / r32(resolution);
                        let dots = ((time_to_seconds(time - last_dot_time) / step).as_f32() + 0.1)
                            .floor() as usize;
                        let overlapped = if is_light_selected_single {
                            0
                        } else {
                            overlapped
                        };
                        let dots = (0..=dots)
                            .map(|i| {
                                let time = last_dot_time + seconds_to_time(step * r32(i as f32));
                                render_light(time, overlapped).center()
                            })
                            .filter(|&pos| self.state.position.contains(pos));

                        self.dots.extend(dots);
                    }
                }
                Event::Effect(effect) => {
                    let is_selected = level_editor.selection.is_single(event_idx);
                    let duration = match *effect {
                        EffectEvent::PaletteSwap(duration)
                        | EffectEvent::RgbSplit(duration)
                        | EffectEvent::CameraShake(duration, _) => duration,
                    };
                    if is_selected {
                        // Start time
                        timeline_tick(
                            vec2(4.0, 16.0) * PPU as f32,
                            render_time(&self.highlight_line, event.time).center(),
                            atlas.timeline_tick_smol(),
                            actions,
                            &mut |actions, target| {
                                // Drag start time
                                let duration = (duration + event.time - target).max(1);
                                actions.push(
                                    LevelAction::list_with(
                                        HistoryLabel::MoveEvent(event_idx),
                                        [
                                            LevelAction::MoveEvent(event_idx, Change::Set(target)),
                                            LevelAction::ChangeEffectDuration(
                                                event_i,
                                                Change::Set(duration),
                                            ),
                                        ],
                                    )
                                    .into(),
                                );
                            },
                            &mut |actions| {
                                actions.push(
                                    LevelAction::FlushChanges(Some(HistoryLabel::MoveEvent(
                                        event_idx,
                                    )))
                                    .into(),
                                );
                            },
                        );

                        // End time
                        timeline_tick(
                            vec2(4.0, 16.0) * PPU as f32,
                            render_time(&self.highlight_line, event.time + duration).center(),
                            atlas.timeline_tick_smol(),
                            actions,
                            &mut |actions, target| {
                                // Drag end time
                                let duration = (target - event.time).max(1);
                                actions.push(
                                    LevelAction::ChangeEffectDuration(
                                        event_i,
                                        Change::Set(duration),
                                    )
                                    .into(),
                                );
                            },
                            &mut |actions| {
                                actions.push(
                                    LevelAction::FlushChanges(Some(HistoryLabel::EventDuration(
                                        event_i,
                                    )))
                                    .into(),
                                );
                            },
                        );
                    }

                    if visible {
                        let texture = match effect {
                            EffectEvent::PaletteSwap(_) => atlas.timeline_palette_swap(),
                            EffectEvent::RgbSplit(_) => atlas.timeline_rgb_split(),
                            EffectEvent::CameraShake(..) => atlas.timeline_shake(),
                        };

                        regular_event(
                            EditorEventIdx::Event(event_i),
                            event.time,
                            duration,
                            texture,
                            actions,
                            &mut occupied,
                            &mut self.dots,
                        );
                    }
                }
            }
        }

        let dragging = if let Some(drag) = &editor.drag
            && let DragTarget::TimelineEvent { .. } = drag.target
        {
            true
        } else {
            false
        };
        // NOTE: overlapping events
        // Normally, events stack on top of each other on the timeline.
        // However, when one light is selected, the timeline view squishes down
        // so it is easier to edit the waypoints of that light.
        // Unless SHIFT is pressed, in which case we're in *multi-select mode*:
        // timeline view grows to fit all lights allowing us to select them.
        self.expansion.target = if self.state.hovered
            && (self.highlight_bar.is_none() || multi_select_mode || dragging)
        {
            occupied.into_values().max().unwrap_or(0) as f32
        } else {
            0.0
        };
        if dragging {
            // Limit expansion dropping which could prevent the dragging from finishing
            self.expansion.target = self.expansion.target.max(self.expansion.current);
        }

        // Main line ticks
        self.ticks.clear();
        let points = &level_editor.level.timing.points;
        for (timing, next) in points
            .iter()
            .zip(points.iter().skip(1).map(Some).chain([None]))
        {
            let from = timing.time;
            let until = next.map(|timing| timing.time);
            for i in 0i32.. {
                let offset = r32(i as f32) * timing.beat_time;
                let time = from + seconds_to_time(offset);

                let mut check_time = -(time + self.scroll);
                if time < -self.scroll {
                    // NOTE: technically imprecise calculation of the next beat timing
                    // but this is strictly for visualization so it doesn't matter much
                    check_time -= seconds_to_time(timing.beat_time);
                }
                if check_time > self.visible_scroll() / 2 {
                    continue;
                }

                let mut tick = |offset: BeatTime, subdivision: i64| {
                    let offset = (r32(i as f32) + offset.as_beats()) * timing.beat_time;
                    let time = from + seconds_to_time(offset);
                    if until.is_none_or(|limit| time < limit) {
                        self.ticks
                            .push((render_time(&self.main_line, time).center(), subdivision));
                    }
                };

                let ticks_per_beat = BeatTime::WHOLE.units() / beat_snap.units();
                for i in 1..ticks_per_beat {
                    let ratio = Ratio::new(beat_snap.units() * i, BeatTime::WHOLE.units());
                    tick(beat_snap * i, *ratio.denom());
                }
                // tick(BeatTime::HALF, 2);
                // tick(BeatTime::QUARTER, 4);
                // tick(BeatTime::QUARTER * 3, 4);

                if until.is_some_and(|limit| time >= limit)
                    || time + self.scroll > self.visible_scroll() / 2
                {
                    break;
                }

                self.ticks
                    .push((render_time(&self.main_line, time).center(), 1));
            }
        }
        // Sort by descending subdivision: whole beats first, then half, quarter, etc.
        self.ticks.sort_by_key(|(_, subdivision)| -*subdivision);

        // Time marks
        self.marks.clear();
        if let Some(level) = &level_editor.level_state.dynamic_level {
            let pos = render_time(&self.main_line, level.time()).center();
            if self.state.position.contains(pos) {
                self.marks
                    .push((pos, Color::lerp(theme.dark, theme.light, 0.5)));
            }
        }
        if let Some(level) = &level_editor.level_state.static_level {
            let pos = render_time(&self.main_line, level.time()).center();
            if self.state.position.contains(pos) {
                self.marks.push((pos, theme.light));
            }
        }

        *context.can_focus.borrow_mut() = focus;
    }

    pub fn get_cursor_time(&self) -> Time {
        self.get_time_at(self.cursor_pos.x)
    }

    fn get_time_at(&self, pos: f32) -> Time {
        ((pos - self.state.position.center().x) / self.scale) as Time - self.scroll
    }

    pub fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &UiContext,
        editor: &Editor,
        level_editor: &LevelEditor,
        actions: &mut Vec<EditorStateAction>,
    ) {
        self.cursor_pos = context.cursor.position;
        self.expansion.update(context.delta_time);

        let pixel = PPU as f32;

        let allocated_position = position;
        let panel_width = 5.0 * context.layout_size;

        // Expand the timeline view up
        let expansion = self.expansion.current * pixel * (LIGHT_LINE_WIDTH + LIGHT_LINE_SPACE);
        let mut position = position
            .extend_up(expansion)
            .extend_symmetric(vec2(-panel_width, 0.0));
        let state_top = position.max.y;

        let ceiling = position.cut_top(pixel * 3.0);
        self.ceiling.update(ceiling, context);

        let extra = position.cut_top(pixel * 3.0);
        self.extra_line.update(extra, context);
        position.cut_top(pixel * 2.0);

        let lights = position.cut_top(pixel * LIGHT_LINE_WIDTH + expansion);
        self.lights_line.update(lights, context);
        position.cut_top(pixel * 4.0);

        let main = position.cut_top(pixel * 16.0);
        self.main_line.update(main, context);
        position.cut_top(pixel * 4.0);

        let highlight = position.cut_top(pixel * 16.0);
        self.highlight_line.update(highlight, context);

        // TODO: unduplicate code from handle_event
        let scroll_speed = if context.mods.shift {
            ScrollSpeed::Slow
        } else if context.mods.alt {
            ScrollSpeed::Fast
        } else {
            ScrollSpeed::Normal
        };

        // Calculate allocated state for side panels
        let mut allocated_position =
            allocated_position.with_height(allocated_position.max.y - position.max.y, 1.0);
        self.allocated_position.update(allocated_position, context);

        // Cut panels on the sides for extra info
        let mut left_panel = allocated_position.cut_left(panel_width);
        let right_panel = allocated_position.cut_right(panel_width);

        {
            // Left panel - Current time
            let mut current_time = left_panel.split_top(0.5);
            current_time.cut_left(current_time.width() * 0.1);
            let time = context.state.get_or(self.state.id, || {
                TextWidget::new("Time: XX:XX.XXX").aligned(vec2(0.0, 0.5))
            });

            let focus_time = editor.drag.as_ref().and_then(|drag| {
                if let DragTarget::TimelineEvent { targets, .. } = &drag.target
                    && targets.len() == 1
                    && let Some(&(id, _)) = targets.first()
                {
                    match id {
                        EditorEventIdx::Event(i) => level_editor
                            .level
                            .events
                            .get(i)
                            .map(|event| event.time + pre_event_time(&event.event)),
                        EditorEventIdx::Waypoint(light_id, waypoint_id) => level_editor
                            .level
                            .events
                            .get(light_id.event)
                            .and_then(|event| {
                                if let Event::Light(light) = &event.event
                                    && let Some(waypoint_time) =
                                        light.movement.get_time(waypoint_id)
                                {
                                    Some(event.time + waypoint_time)
                                } else {
                                    None
                                }
                            }),
                        EditorEventIdx::Timing(i) => level_editor
                            .level
                            .timing
                            .points
                            .get(i)
                            .map(|point| point.time),
                    }
                } else {
                    None
                }
            });
            let color = if focus_time.is_some() {
                context.theme().highlight
            } else {
                context.theme().light
            };

            let display_time = focus_time.unwrap_or(self.raw_current_time);
            let mut ms = display_time;
            let mut secs = ms / 1000;
            ms -= secs * 1000;
            let mins = secs / 60;
            secs -= mins * 60;

            time.text = format!("Time: {:02}:{:02}.{:03}", mins, secs, ms).into();
            time.update(current_time, context);
            time.options.size = current_time.height() * 0.4;
            time.options.color = color;

            // Current beat
            let mut current_beat = left_panel;
            current_beat.cut_left(current_beat.width() * 0.1);
            let beat = context.state.get_or(self.state.id, || {
                TextWidget::new("Beat: XX X/X").aligned(vec2(0.0, 1.0))
            });

            let beat_time = focus_time.unwrap_or(self.raw_target_time);
            let beat_time = level_editor.level.timing.get_relative_beat_time(beat_time);
            let ratio = Ratio::new(beat_time.units(), BeatTime::UNITS_PER_BEAT);
            let sub_division = *ratio.denom();
            let mut sub_beat = *ratio.numer();
            let beat_whole = sub_beat / sub_division;
            sub_beat -= beat_whole * sub_division;

            beat.text = if sub_beat > 0 {
                format!("Beat: {}  {}/{}", beat_whole + 1, sub_beat, sub_division).into()
            } else {
                format!("Beat: {}", beat_whole + 1).into()
            };
            beat.update(current_beat, context);
            beat.options.size = current_beat.height() * 0.4;
            beat.options.color = color;
        }

        {
            // Right panel - Timing subdivision
            let allowed_subdivisions = [1, 2, 3, 4, 6, 8, 12, 16];
            let current_subdivision = BeatTime::WHOLE.units() / level_editor.beat_snap.units();
            let current_i = allowed_subdivisions
                .iter()
                .position(|d| *d == current_subdivision)
                .unwrap_or(0);
            let mut new_i = current_i;

            let mut panel = right_panel;

            let text_pos = panel.split_top(0.5);
            let text = context.state.get_or(self.state.id, || {
                TextWidget::new("1 / X").aligned(vec2(0.5, 0.0))
            });
            text.update(text_pos, context);
            text.text = format!("1 / {}", current_subdivision).into();

            let button_left = panel.split_left(0.5);
            let button = context.state.get_or(self.state.id, || {
                IconButtonWidget::new(context.context.assets.atlas.button_prev())
            });
            button.update(button_left, context);
            if button.state.mouse_left.clicked {
                new_i = new_i
                    .checked_sub(1)
                    .unwrap_or(allowed_subdivisions.len() - 1);
            }

            let button = context.state.get_or(self.state.id, || {
                IconButtonWidget::new(context.context.assets.atlas.button_next())
            });
            button.update(panel, context);
            if button.state.mouse_left.clicked {
                new_i += 1;
                if new_i >= allowed_subdivisions.len() {
                    new_i = 0;
                }
            }

            if current_i != new_i
                && let Some(&subdivision) = allowed_subdivisions.get(new_i)
            {
                actions.push(
                    LevelAction::SetBeatSnap(BeatTime::from_units(
                        BeatTime::WHOLE.units() / subdivision,
                    ))
                    .into(),
                );
            }
        }

        // Update state
        let state_full = Aabb2 {
            min: vec2(position.min.x, position.max.y),
            max: vec2(position.max.x, state_top),
        };
        self.state.update(state_full, context);
        if self.state.hovered {
            let delta = context.cursor.scroll_dir();
            if delta != 0 {
                if context.mods.ctrl {
                    // Zoom on the timeline
                    let delta = delta as f32;
                    actions.push(LevelAction::TimelineZoom(Change::Add(delta)).into());
                } else {
                    // Scroll on the timeline
                    actions.push(EditorAction::ScrollTimeBy(scroll_speed, delta).into());
                }
            }
        }
        if self.state.mouse_right.clicked {
            // TODO: maybe more specific to actual timeline actions
            actions.push(LevelAction::Cancel.into());
        }

        if self.main_line.mouse_left.clicked {
            let time = self.get_cursor_time();
            actions.push(LevelAction::ScrollTime(time - level_editor.current_time.target).into());
        }

        self.reload(context, editor, level_editor, actions);

        context.update_focus(self.state.hovered); // Take focus
    }
}

/// The time at the start of the event that exists but is not visualized.
fn pre_event_time(event: &Event) -> Time {
    match event {
        Event::Light(light) => light.movement.get_fade_in(),
        _ => 0,
    }
}

impl Widget for TimelineWidget {
    simple_widget_state!();
    fn draw_top(&self, context: &UiContext) -> Geometry {
        let pixel_scale = PPU;
        let pixel = pixel_scale as f32;
        let theme = context.theme();
        let atlas = &context.context.assets.atlas;

        let mut geometry = Geometry::new();

        // Current arrow
        {
            let texture = &atlas.timeline_current_arrow();
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

        // Main line ticks
        for &(pos, subdivision) in &self.ticks {
            let white = theme.light;
            let red = theme.danger;
            let cyan = theme.highlight;
            let mut yellow =
                Color::from_vec4(Color::WHITE.to_vec4() - cyan.to_vec4()).map_rgb(|x| x.max(0.0));
            yellow.a = 1.0;
            let (color, texture) = match subdivision {
                1 => (white, &atlas.timeline_tick_big()),
                2 => (red, &atlas.timeline_tick_mid()),
                3 => (yellow, &atlas.timeline_tick_mid()),
                4 => (cyan, &atlas.timeline_tick_smol()),
                6 => (Color::lerp(yellow, red, 0.5), &atlas.timeline_tick_smol()),
                8 => (Color::lerp(yellow, cyan, 0.25), &atlas.timeline_tick_tiny()),
                12 => (Color::lerp(yellow, red, 0.8), &atlas.timeline_tick_tiny()),
                16 => (Color::lerp(cyan, red, 0.5), &atlas.timeline_tick_tiny()),
                _ => {
                    // Unknown beat separation
                    (red, &atlas.timeline_tick_smol())
                }
            };
            geometry.merge(
                context
                    .geometry
                    .texture_pp_at(pos, color, pixel_scale, texture),
            );
        }

        let position = self.state.position;
        geometry = context.geometry.masked(position, geometry);
        let width = pixel * 2.0;
        geometry.merge(context.geometry.quad_outline(
            position.extend_uniform(width),
            width,
            theme.light,
        ));

        geometry
    }
    fn draw(&self, context: &UiContext) -> Geometry {
        let pixel_scale = PPU;
        let pixel = pixel_scale as f32;
        let theme = context.theme();
        let atlas = &context.context.assets.atlas;
        let bounds = self.state.position;

        let mut geometry = Geometry::new();

        // Lifetime dots
        for &dot in &self.dots {
            let dot = geng_utils::pixel::pixel_perfect_aabb(
                dot,
                vec2(0.5, 0.5),
                vec2::splat(pixel_scale * 2),
                &geng::PixelPerfectCamera,
                context.geometry.framebuffer_size.as_f32(),
            );
            geometry.merge(context.geometry.quad(dot, theme.light));
        }

        // Main
        let main_bar = self.main_line.position;
        let mut main_bar = main_bar.align_aabb(vec2(main_bar.width(), pixel * 4.0), vec2(0.5, 0.5));
        main_bar.min = main_bar
            .min
            .clamp_coordinates(bounds.min.x..=bounds.max.x, bounds.min.y..=bounds.max.y);
        main_bar.max = main_bar
            .max
            .clamp_coordinates(bounds.min.x..=bounds.max.x, bounds.min.y..=bounds.max.y);
        geometry.merge(context.geometry.quad(main_bar, theme.light));

        // Highlight
        if let Some(bar) = &self.highlight_bar {
            let highlight_bar = Aabb2::from_corners(bar.from, bar.to);
            let mut highlight_bar =
                highlight_bar.align_aabb(vec2(highlight_bar.width(), pixel * 4.0), vec2(0.5, 0.5));
            highlight_bar.min = highlight_bar
                .min
                .clamp_coordinates(bounds.min.x..=bounds.max.x, bounds.min.y..=bounds.max.y);
            highlight_bar.max = highlight_bar
                .max
                .clamp_coordinates(bounds.min.x..=bounds.max.x, bounds.min.y..=bounds.max.y);
            geometry.merge(context.geometry.quad(highlight_bar, theme.highlight));
        }

        // Time marks
        for &(pos, color) in &self.marks {
            let texture = atlas.timeline_time_mark();
            geometry.merge(
                context
                    .geometry
                    .texture_pp_at(pos, color, pixel_scale, &texture),
            );
        }

        // NOTE: mask is done manually because it weirdly affects the rendering order
        let width = pixel * 2.0;
        geometry.merge(
            context
                .geometry
                .quad_fill(bounds.extend_uniform(width), width, theme.dark),
        );

        // Side panels

        geometry.merge(context.geometry.quad_outline(
            self.allocated_position.position.extend_uniform(width),
            width,
            theme.light,
        ));

        geometry
    }
}

#[derive(Clone)]
struct IconWidget {
    state: WidgetState,
    texture: SubTexture,
    color: ThemeColor,
}

impl IconWidget {
    pub fn new(texture: SubTexture) -> Self {
        Self {
            state: default(),
            texture,
            color: ThemeColor::Light,
        }
    }
    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }
}

impl Widget for IconWidget {
    simple_widget_state!();
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

#[derive(Debug, Clone, Copy)]
enum HighlightMode {
    SwapColors,
    Color(ThemeColor),
}

#[derive(Clone)]
struct IconButtonWidget {
    state: WidgetState,
    texture: SubTexture,
    color: ThemeColor,
    bg_color: ThemeColor,
    highlight: HighlightMode,
}

impl IconButtonWidget {
    pub fn new(texture: SubTexture) -> Self {
        Self {
            state: WidgetState::new(),
            texture,
            color: ThemeColor::Light,
            bg_color: ThemeColor::Dark,
            highlight: HighlightMode::SwapColors,
        }
    }

    pub fn highlight(mut self, mode: HighlightMode) -> Self {
        self.highlight = mode;
        self
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        context.update_focus(self.state.hovered);
    }
}

impl Widget for IconButtonWidget {
    simple_widget_state!();
    fn draw(&self, context: &UiContext) -> Geometry {
        let pixel_scale = PPU;
        let theme = context.theme();
        let outline_width = pixel_scale as f32 * 3.0;
        let mut geometry = Geometry::new();

        let mut fg_color = theme.get_color(self.color);
        let mut bg_color = theme.get_color(self.bg_color);

        match self.highlight {
            HighlightMode::SwapColors => {
                if self.state.hovered {
                    std::mem::swap(&mut fg_color, &mut bg_color);
                }

                let size = self.texture.size() * pixel_scale;
                let position = geng_utils::pixel::pixel_perfect_aabb(
                    self.state.position.center(),
                    vec2(0.5, 0.5),
                    size,
                    &geng::PixelPerfectCamera,
                    context.geometry.framebuffer_size.as_f32(),
                );

                geometry.merge(context.geometry.texture(
                    position,
                    mat3::identity(),
                    fg_color,
                    &self.texture,
                ));
                geometry.merge(context.geometry.quad_fill(
                    position.extend_uniform(outline_width + pixel_scale as f32),
                    outline_width,
                    bg_color,
                ));
            }
            HighlightMode::Color(highlight) => {
                if self.state.hovered {
                    fg_color = theme.get_color(highlight);
                }
                geometry.merge(context.geometry.texture_pp_at(
                    self.state.position.center(),
                    fg_color,
                    pixel_scale,
                    &self.texture,
                ));
            }
        }

        geometry
    }
}

fn editor_event_time(id: EditorEventIdx, level_editor: &LevelEditor) -> Option<Time> {
    match id {
        EditorEventIdx::Event(idx) => {
            let event = level_editor.level.events.get(idx)?;
            Some(event.time)
        }
        EditorEventIdx::Waypoint(light_id, waypoint_id) => {
            let event = level_editor.level.events.get(light_id.event)?;
            if let Event::Light(light) = &event.event {
                let time = light.movement.get_time(waypoint_id)?;
                Some(event.time + time)
            } else {
                None
            }
        }
        EditorEventIdx::Timing(idx) => Some(level_editor.level.timing.points.get(idx)?.time),
    }
}
