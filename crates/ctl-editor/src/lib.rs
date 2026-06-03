mod action;
mod clipboard;
mod config;
mod grid;
mod group;
mod history;
mod interpolation_cache;
mod level;
mod state;
#[cfg(test)]
mod tests;
pub mod ui;

pub use self::{
    action::*,
    clipboard::*,
    config::*,
    grid::*,
    group::*,
    history::*,
    interpolation_cache::*,
    level::*,
    state::{EditingState, *},
};

use ctl_logic::*;
use ctl_ui::widget::ConfirmPopup;
use ctl_util::{Change, SecondOrderState, TimeInterpolation};

use itertools::Itertools;

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    Noop,
    ExitUnsaved,
    ChangeLevelUnsaved(usize),
    DeleteDiff(usize),
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
