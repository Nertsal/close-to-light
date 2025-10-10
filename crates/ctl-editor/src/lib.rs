mod action;
mod clipboard;
mod config;
mod grid;
mod group;
mod history;
mod level;
mod state;
pub mod ui;

pub use self::{
    action::*,
    clipboard::*,
    config::*,
    grid::*,
    group::*,
    history::*,
    level::*,
    state::{EditingState, *},
};

use ctl_logic::*;
use ctl_ui::widget::ConfirmPopup;
use ctl_util::{SecondOrderDynamics, SecondOrderState};

use itertools::Itertools;

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    ExitUnsaved,
    ChangeLevelUnsaved(usize),
    DeleteLevel(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTab {
    Edit,
    Config,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScrollSpeed {
    Slow,
    Normal,
    Fast,
}

pub struct TimeInterpolation {
    state: SecondOrderState<FloatTime>,
    pub value: Time,
    pub target: Time,
}

impl Default for TimeInterpolation {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeInterpolation {
    pub fn new() -> Self {
        let time = Time::ZERO;
        Self {
            state: SecondOrderState::new(SecondOrderDynamics::new(
                3.0,
                1.0,
                0.0,
                time_to_seconds(time),
            )),
            value: time,
            target: time,
        }
    }

    pub fn update(&mut self, delta_time: FloatTime) {
        self.state.update(delta_time.as_f32());
        if (self.state.current - self.state.target).abs().as_f32() < 0.002 {
            // Skip the final step for better precision on dependent visuals
            self.state.current = self.state.target;
        }
        self.value = seconds_to_time(self.state.current);
    }

    pub fn scroll_time(&mut self, change: Change<Time>) {
        change.apply(&mut self.target);
        self.state.target = time_to_seconds(self.target);
    }

    pub fn snap_to(&mut self, time: Time) {
        self.value = time;
        self.target = time;
        let time = time_to_seconds(self.value);
        self.state.current = time;
        self.state.target = time;
    }
}
