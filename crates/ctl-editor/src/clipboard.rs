use super::*;

pub struct Clipboard {
    data: Option<ClipboardItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClipboardItem {
    Events(Time, Vec<TimedEvent>),
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
}
