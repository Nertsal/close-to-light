use super::widget::Widget;

use std::{cell::UnsafeCell, collections::BTreeMap, panic::Location};

use geng::prelude::{once_cell::sync::Lazy, *};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WidgetId(u64, Id, usize);

static ID_COUNTER: Lazy<Arc<Mutex<u64>>> = Lazy::new(|| Arc::new(Mutex::new(1)));

fn get_next_id() -> u64 {
    let Ok(mut counter) = ID_COUNTER.lock() else {
        return 0;
    };
    let id = *counter;
    *counter += 1;
    id
}

impl WidgetId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(get_next_id(), *Location::caller(), 0)
    }

    // #[track_caller]
    // fn state_new() -> Self {
    //     Self(get_next_id(), *Location::caller(), 0)
    // }

    fn state_root() -> Self {
        Self(0, *Location::caller(), 0)
    }
}

// Main id is Location, but if in the same frame, the same location is called multiple times,
// we spawn a new widget for each.
type Id = Location<'static>;

pub struct UiLayerHandle<'a> {
    phantom_data: PhantomData<&'a ()>,
    z_index: Rc<RefCell<i64>>,
}

impl Drop for UiLayerHandle<'_> {
    fn drop(&mut self) {
        *self.z_index.borrow_mut() -= 1;
    }
}

#[derive(Clone, Default)]
pub struct UiState(Rc<RefCell<State>>);

#[derive(Default)]
struct State {
    z_index: Rc<RefCell<i64>>,
    children: HashMap<WidgetId, Vec<WidgetId>>, // TODO: smallvec
    active: HashMap<Id, usize>,
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

    /// Start a temporary higher layer. The handle returned must be held to the duration of the whole layer.
    /// Use `let _l = context.state.start_layer();` to ensure the handle is automatically dropped only at the end of the scope.
    #[must_use]
    pub fn start_layer(&self) -> UiLayerHandle<'_> {
        let inner = self.0.borrow_mut();
        *inner.z_index.borrow_mut() += 1;
        UiLayerHandle {
            phantom_data: PhantomData,
            z_index: inner.z_index.clone(),
        }
    }

    // Manually decrement the layer counter. Can be used after processing higher layer widgets to change the layer of all future widgets.
    pub fn decrement_layer(&self) {
        let inner = self.0.borrow_mut();
        *inner.z_index.borrow_mut() -= 1;
    }

    pub fn current_layer(&self) -> i64 {
        let inner = self.0.borrow();
        *inner.z_index.borrow()
    }

    /// Should be called at the start of every ui frame to reset widgets.
    // NOTE: must require `&mut self` to ensure widget aliasing.
    pub fn frame_start(&mut self) {
        let mut inner = self.0.borrow_mut();
        *inner.z_index.borrow_mut() = 0;
        inner.children.clear();
        inner.active.clear();
        for cell in inner.widgets.values_mut() {
            cell.get_mut().next = 0;
        }
    }

    #[track_caller]
    pub fn get_root_or_default<T: 'static + Default + Widget>(&self) -> &mut T {
        self.get_or(WidgetId::state_root(), Default::default)
    }

    #[track_caller]
    #[allow(clippy::mut_from_ref)]
    pub fn get_root_or<T: 'static + Widget>(&self, default: impl FnOnce() -> T) -> &mut T {
        self.get_or(WidgetId::state_root(), default)
    }

    // #[track_caller]
    // pub fn get_or_default<T: 'static + Default + Widget>(&self, parent: WidgetId) -> &mut T {
    //     self.get_or(parent, Default::default)
    // }

    #[track_caller]
    #[allow(clippy::mut_from_ref)]
    pub fn get_or<T: 'static + Widget>(
        &self,
        parent: WidgetId,
        default: impl FnOnce() -> T,
    ) -> &mut T {
        let mut inner = self.0.borrow_mut();
        let z_index = *inner.z_index.borrow();
        let id = *Location::caller();
        let count = inner.active.entry(id).or_insert(0);
        let widget_id = WidgetId(parent.0, id, *count);
        *count += 1;

        inner.children.entry(parent).or_default().push(widget_id);

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
        // In case this is a repeat call we increment the counter and access the next widget.
        // This way, collisions can only happen if the counter is reset (at the beginning of the frame)
        // but widgets are still held, which cannot happen because `frame_start` takes a mutable reference.
        let entry = unsafe { &mut *(entry as *mut Box<dyn Widget>) };
        let widget: &mut T = entry
            .to_any_mut()
            .downcast_mut()
            .expect("invalid implementation of UiState::get_or");
        let widget_state = widget.state_mut();
        widget_state.id = widget_id;
        widget_state.z_index = z_index;
        widget
    }

    pub fn iter_widgets(
        &self,
        mut f_pre: impl FnMut(&dyn Widget),
        mut f_post: impl FnMut(&dyn Widget),
    ) {
        let inner = self.0.borrow();
        inner.iter_children(&WidgetId::state_root(), &mut f_pre, &mut f_post)
    }
}

impl State {
    fn iter_children(
        &self,
        parent: &WidgetId,
        f_pre: &mut impl FnMut(&dyn Widget),
        f_post: &mut impl FnMut(&dyn Widget),
    ) {
        if let Some(children) = self.children.get(parent) {
            for child in children {
                {
                    let cell = self.widgets.get(&child.1).expect(
                        "invalid implementation of UiState: active id is not present in widgets",
                    );
                    let cell = unsafe { &*(cell.get()) };
                    let w = cell.widgets.get(child.2).expect(
                        "invalid implementation of UiState: active id is not present in widgets",
                    );
                    f_pre(&**w);
                }

                self.iter_children(child, f_pre, f_post);

                let cell = self.widgets.get(&child.1).expect(
                    "invalid implementation of UiState: active id is not present in widgets",
                );
                let cell = unsafe { &*(cell.get()) };
                let w = cell.widgets.get(child.2).expect(
                    "invalid implementation of UiState: active id is not present in widgets",
                );
                f_post(&**w);
            }
        }
    }
}
