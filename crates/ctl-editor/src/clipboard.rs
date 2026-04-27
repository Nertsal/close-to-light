use super::*;

pub struct Clipboard {
    data: Option<ClipboardItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClipboardItem {
    Events {
        time: Time,
        events: Vec<ClipboardEvent<TimedEvent>>,
        timing: Vec<ClipboardEvent<TimingPoint>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipboardEvent<E> {
    pub event: E,
    pub beat_aligned: bool,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard {
    pub fn new() -> Self {
        Self { data: None }
    }

    pub fn clear(&mut self) {
        self.data = None;
    }

    pub fn copy(&mut self, item: ClipboardItem) {
        self.data = Some(item);
    }

    pub fn paste(&mut self) -> Option<ClipboardItem> {
        self.data.clone()
    }

    pub fn get(&self) -> &Option<ClipboardItem> {
        &self.data
    }
}
