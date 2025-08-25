mod action;
mod handle_event;

use crate::{
    prelude::*,
    render::{editor::EditorRender, post::PostRender},
};

pub use ctl_editor::{ui::*, *};
use ctl_local::Leaderboard;
use ctl_logic::{PlayGroup, PlayLevel};
use ctl_ui::UiContext;
use ctl_util::{SecondOrderDynamics, SecondOrderState};

pub struct EditorState {
    context: Context,
    transition: Option<geng::state::Transition>,
    /// Stop music on the next `update` frame. Used when returning from F5 play to stop music.
    stop_music_next_frame: bool,
    render: EditorRender,
    post_render: PostRender,
    editor: Editor,
    framebuffer_size: vec2<usize>,
    delta_time: FloatTime,
    ui: EditorUi,
    ui_focused: bool,
    ui_context: UiContext,
}

impl EditorState {
    pub fn new_group(context: Context, config: EditorConfig, group: PlayGroup) -> Self {
        Self {
            transition: None,
            stop_music_next_frame: true,
            render: EditorRender::new(context.clone()),
            post_render: PostRender::new(context.clone()),
            framebuffer_size: vec2(1, 1),
            delta_time: r32(0.1),
            ui: EditorUi::new(context.clone()),
            ui_focused: false,
            ui_context: UiContext::new(context.clone()),
            editor: Editor {
                context: context.clone(),
                real_time: FloatTime::ZERO,
                render_options: RenderOptions {
                    show_grid: true,
                    hide_ui: false,
                },
                cursor_world_pos: vec2::ZERO,
                cursor_world_pos_snapped: vec2::ZERO,
                drag: None,

                confirm_popup: None,

                tab: EditorTab::Config,
                exit: false,

                grid: Grid::new_with(config.grid.clone()),
                view_zoom: SecondOrderState::new(SecondOrderDynamics::new(3.0, 1.0, 1.0, 1.0)),
                visualize_beat: true,
                show_only_selected: false,
                snap_to_grid: true,
                music_timer: FloatTime::ZERO,

                group,
                level_edit: None,
                config,
            },
            context,
        }
    }

    pub fn new_level(context: Context, config: EditorConfig, level: PlayLevel) -> Self {
        let mut editor = Self::new_group(context.clone(), config, level.group.clone());
        editor.editor.tab = EditorTab::Edit;
        let options = context.get_options();
        let model = Model::empty(context.clone(), options, level.clone());
        editor.editor.level_edit = Some(LevelEditor::new(context, model, level, true, false));
        editor
    }

    fn snap_pos_grid(&self, pos: vec2<Coord>) -> vec2<Coord> {
        self.editor.grid.snap_pos(pos)
    }

    // TODO: scale snap
    // fn snap_distance_grid(&self, distance: Coord) -> Coord {
    //     self.snap_pos_grid(vec2(distance, Coord::ZERO)).x
    // }

    fn update_level_editor(&mut self, delta_time: FloatTime) {
        let Some(level_editor) = &mut self.editor.level_edit else {
            return;
        };

        self.context
            .music
            .set_volume(level_editor.model.options.volume.music());

        level_editor.real_time += delta_time;
        level_editor.current_time.update(delta_time);
        level_editor.timeline_zoom.update(delta_time.as_f32());

        if self.editor.music_timer > FloatTime::ZERO {
            self.editor.music_timer -= delta_time;
            if self.editor.music_timer <= FloatTime::ZERO {
                self.context.music.stop();
            }
        }

        // TODO: maybe config option?
        // if let Some(waypoints) = &level_editor.level_state.waypoints {
        //     if let Some(waypoint) = waypoints.selected {
        //         if let Some(event) = level_editor.level.events.get(waypoints.light.event) {
        //             if let Event::Light(light) = &event.event {
        //                 // Set current time to align with the selected waypoint
        //                 if let Some(time) = light.movement.get_time(waypoint) {
        //                     level_editor.current_time = event.time + time;
        //                 }
        //             }
        //         }
        //     }
        // }

        if level_editor.scrolling_time {
            level_editor.was_scrolling_time = true;
        } else {
            // TODO: potentially config option?
            // if level_editor.was_scrolling_time {
            //     // Stopped scrolling
            //     if let Some(music) = &level_editor.static_level.group.music {
            //         // Play some music
            //         self.context
            //             .music
            //             .play_from_time(music, level_editor.current_time.value);
            //         self.editor.music_timer = self.editor.config.playback_duration;
            //     }
            // }
            level_editor.was_scrolling_time = false;
        }

        level_editor.scrolling_time = false;

        if let EditingState::Playing {
            start_target_time,
            playing_time,
            ..
        } = &mut level_editor.state
        {
            // Scroll time forward
            *playing_time += delta_time;
            level_editor
                .current_time
                .snap_to(*start_target_time + seconds_to_time(*playing_time));
        }

        let include_cursor = !self.ui_focused
            && (self.editor.render_options.hide_ui
                || self
                    .ui
                    .game
                    .position
                    .contains(self.ui_context.cursor.position));
        level_editor.render_lights(
            include_cursor.then_some(self.editor.cursor_world_pos),
            include_cursor.then_some(self.editor.cursor_world_pos_snapped),
            self.editor.visualize_beat,
            self.editor.show_only_selected,
        );

        let pos = self.ui_context.cursor.position;
        let pos = pos - self.ui_context.screen.bottom_left();
        let pos = level_editor
            .model
            .camera
            .screen_to_world(self.ui_context.screen.size(), pos)
            .as_r32();
        self.editor.cursor_world_pos = pos;
        self.editor.cursor_world_pos_snapped = if self.editor.snap_to_grid {
            self.snap_pos_grid(pos)
        } else {
            pos
        };
    }

    /// Start playing the game from the current time.
    fn play_game(&mut self) {
        let Some(level_editor) = &self.editor.level_edit else {
            return;
        };

        let level = ctl_logic::PlayLevel {
            start_time: level_editor.current_time.target,
            level: Rc::new(LevelFull {
                meta: level_editor.static_level.level.meta.clone(),
                data: level_editor.level.clone(),
            }),
            ..level_editor.static_level.clone()
        };

        self.transition = Some(geng::state::Transition::Push(Box::new(
            crate::game::Game::new(
                self.context.clone(),
                level_editor.model.options.clone(),
                level,
                Leaderboard::new(&self.context.geng, None),
            ),
        )));
        self.stop_music_next_frame = true;
    }
}

impl geng::State for EditorState {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = FloatTime::new(delta_time as f32);
        self.delta_time = delta_time;
        self.editor.real_time += delta_time;

        self.context.local.poll();
        self.context
            .geng
            .window()
            .set_cursor_type(geng::CursorType::Default);

        if self.transition.is_none() && std::mem::take(&mut self.stop_music_next_frame) {
            self.context.music.stop();
        }

        self.ui_context.update(delta_time.as_f32());
        self.editor.view_zoom.update(delta_time.as_f32());

        for action in self.update_drag() {
            self.execute(action);
        }

        self.update_level_editor(delta_time);

        if std::mem::take(&mut self.editor.exit) {
            self.execute(EditorStateAction::Exit);
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        let actions = self.process_event(event);
        for action in actions {
            self.execute(action);
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Color::BLACK), None, None);

        self.ui_context.state.frame_start();
        self.ui_context.geometry.update(framebuffer.size());
        let (can_focus, actions) = if !self.editor.render_options.hide_ui {
            self.ui.layout(
                &self.editor,
                Aabb2::ZERO.extend_positive(framebuffer.size().as_f32()),
                &mut self.ui_context,
            )
        } else {
            (true, vec![])
        };
        self.ui_focused = !can_focus;
        self.ui_context.frame_end();
        for action in actions {
            self.execute(action);
        }

        if let Some(level_editor) = &mut self.editor.level_edit {
            level_editor.model.camera.fov =
                geng::Camera2dFov::Vertical(10.0 / self.editor.view_zoom.current);
        }

        let buffer = &mut self.post_render.begin(framebuffer.size());
        self.render
            .draw_editor(&self.editor, &self.ui, &self.ui_context, buffer);
        self.post_render
            .post_process(framebuffer, self.editor.real_time);
    }
}
