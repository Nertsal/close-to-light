use super::widget::Widget;

use std::{cell::UnsafeCell, collections::BTreeMap, panic::Location};

use geng::prelude::*;

// Main id is Location, but if in the same frame, the same location is called multiple times,
// we spawn a new widget for each.
type Id = Location<'static>;

#[derive(Clone, Default)]
pub struct UiState(Rc<RefCell<State>>);

#[derive(Default)]
struct State {
    active: HashSet<Id>,
    // NOTE: BTreeMap to have consistent iteration order
    widgets: BTreeMap<Id, UnsafeCell<UuidCell>>, // TODO: check memory leakage
}

struct UuidCell {
    next: usize,
    widgets: Vec<Box<dyn Widget>>,
}

impl UiState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Should be called at the start of every ui frame to reset widgets.
    // NOTE: must require `&mut self` to ensure widget aliasing.
    pub fn frame_start(&mut self) {
        let mut inner = self.0.borrow_mut();
        inner.active.clear();
        for cell in inner.widgets.values_mut() {
            cell.get_mut().next = 0;
        }
    }

    #[track_caller]
    pub fn get_or_default<T: 'static + Default + Widget>(&self) -> &mut T {
        self.get_or(Default::default)
    }

    #[track_caller]
    #[allow(clippy::mut_from_ref)]
    pub fn get_or<T: 'static + Widget>(&self, default: impl FnOnce() -> T) -> &mut T {
        let mut inner = self.0.borrow_mut();
        let id = *Location::caller();
        inner.active.insert(id);

        let entry = inner.widgets.entry(id).or_insert_with(move || {
            UnsafeCell::new(UuidCell {
                next: 0,
                widgets: Vec::new(),
            })
        });
        let cell = entry.get_mut();
        if cell.widgets.len() <= cell.next {
            cell.widgets.push(Box::new(default()));
            assert!(cell.widgets.len() > cell.next);
        }
        let entry = cell
            .widgets
            .get_mut(cell.next)
            .expect("widget inserted to fit");
        cell.next += 1;

        // SAFETY: each element in `widgets` is unique per call to this function
        // guaranteed by id being `Location::caller()`.
        // Therefore, the reference can be given out for the duration until the same
        // call is reached again.
        // In case this is repeat call, we increment the counter and access the next widget.
        // This way, no collisions can only happen if the counter is reset (at the beginning of the frame)
        // but widgets are still held, which cannot happen because `frame_start` takes a mutable reference.
        let entry = unsafe { &mut *(entry as *mut Box<dyn Widget>) };
        entry
            .to_any_mut()
            .downcast_mut()
            .expect("invalid implementation of UiState::get_or")
    }

    pub fn iter_widgets(&self, mut f: impl FnMut(&dyn Widget)) {
        let inner = self.0.borrow();
        inner.active.iter().for_each(|id| {
            let w = inner
                .widgets
                .get(id)
                .expect("invalid implementation of UiState: active id is not present in widgets");
            let w = unsafe { &*(w.get()) };
            for w in &w.widgets {
                f(&**w);
            }
        })
    }
}
