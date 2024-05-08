use geng::prelude::*;
use geng_utils::bounded::Bounded;

#[derive(Debug, Clone)]
pub struct ShowTime<T> {
    pub data: T,
    pub time: Bounded<f32>,
    /// Whether the time is going up or down.
    pub going_up: bool,
}

impl<T> ShowTime<T> {
    pub fn new(time: f32, data: T) -> Self {
        Self {
            data,
            time: Bounded::new_zero(time),
            going_up: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WidgetRequest {
    Open,
    Close,
    Reload,
}

#[derive(Debug, Clone)]
pub struct UiWindow<T> {
    pub show: ShowTime<T>,
    pub request: Option<WidgetRequest>,
    /// Whether reload request should close and open the window or do nothing.
    pub reload_reopen: bool,
    /// Last processed request, can be used for outside logic.
    pub last_request: Option<WidgetRequest>,
}

impl<T> UiWindow<T> {
    pub fn new(data: T, time: f32) -> Self {
        Self {
            show: ShowTime {
                data,
                time: Bounded::new_zero(time),
                going_up: false,
            },
            request: None,
            reload_reopen: true,
            last_request: None,
        }
    }

    /// Do nothing on reload.
    pub fn reload_skip(self) -> Self {
        Self {
            reload_reopen: false,
            ..self
        }
    }

    /// Layout phase update given ui widget state.
    pub fn layout(&mut self, hovered: bool, close: bool) {
        if close {
            self.request = Some(WidgetRequest::Close);
        } else if hovered && self.show.time.is_min() {
            self.request = Some(WidgetRequest::Open);
        }
    }

    /// Updates current state based on the set request, if any.
    pub fn update(&mut self, delta_time: f32) {
        if let Some(req) = self.request {
            self.last_request = Some(req);
            match req {
                WidgetRequest::Open => {
                    if self.show.time.is_min() {
                        self.show.going_up = true;
                        self.request = None;
                    }
                }
                WidgetRequest::Close => self.show.going_up = false,
                WidgetRequest::Reload => {
                    if self.reload_reopen {
                        self.show.going_up = false;
                        self.request = Some(WidgetRequest::Open);
                    } else {
                        self.request = None;
                    }
                }
            }
        }

        let sign = if self.show.going_up { 1.0 } else { -1.0 };
        self.show.time.change(sign * delta_time);
    }
}
