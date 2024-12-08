use super::widget::Widget;

use std::{cell::UnsafeCell, panic::Location};

use geng::prelude::*;

type UiId = Location<'static>;

#[derive(Clone, Default)]
pub struct UiState(Rc<RefCell<State>>);

#[derive(Default)]
struct State {
    active: HashSet<UiId>,
    widgets: HashMap<UiId, UnsafeCell<Box<dyn Widget>>>, // TODO: check memory leakage
}

impl UiState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Should be called at the start of every ui frame to reset widgets.
    pub fn frame_start(&self) {
        self.0.borrow_mut().active.clear();
    }

    pub fn get_or_default<T: 'static + Default + Widget>(&self) -> &mut T {
        self.get_or(Default::default)
    }

    #[track_caller]
    #[allow(clippy::mut_from_ref)]
    pub fn get_or<T: 'static + Widget>(&self, default: impl FnOnce() -> T) -> &mut T {
        let &id = Location::caller();
        let mut inner = self.0.borrow_mut();
        inner.active.insert(id);

        let entry = inner
            .widgets
            .entry(id)
            .or_insert_with(move || UnsafeCell::new(Box::new(default())));
        // SAFETY: each element in `widgets` is unique per call to this function
        // guaranteed by id being `Location::caller()`.
        // Therefore, the reference can be given out for the duration until the same
        // call is reached again (which could happen inside a loop or recursion).
        let entry = unsafe { &mut *(entry.get()) };
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
            f(&**w)
        })
    }
}
