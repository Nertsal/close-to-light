use super::*;

use crate::leaderboard::Leaderboard;

pub struct ProfileWidget {
    pub state: WidgetState,
    pub offline: TextWidget,
    pub register: RegisterWidget,
    pub logged: LoggedWidget,
}

pub struct RegisterWidget {
    pub state: WidgetState,
}

pub struct LoggedWidget {
    pub state: WidgetState,
}

impl ProfileWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            offline: TextWidget::new("Offline"),
            register: RegisterWidget {
                state: WidgetState::new(),
            },
            logged: LoggedWidget {
                state: WidgetState::new(),
            },
        }
    }
}

impl StatefulWidget for ProfileWidget {
    type State = Leaderboard;

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);

        let margin = context.layout_size * 0.5;
        let main = position.extend_uniform(-margin);

        let (off, reg, log) = match (state.client().is_some(), state.user.is_some()) {
            (false, _) => (true, false, false),
            (true, false) => (false, true, false),
            (true, true) => (false, false, true),
        };
        if off {
            self.offline.show()
        } else {
            self.offline.hide()
        }
        if reg {
            self.register.show()
        } else {
            self.register.hide()
        }
        if log {
            self.logged.show()
        } else {
            self.logged.hide()
        }

        self.offline.update(main, context);
        self.register.update(main, context, state);
        self.logged.update(main, context, state);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
        self.offline.walk_states_mut(f);
        self.register.walk_states_mut(f);
        self.logged.walk_states_mut(f);
    }
}

impl StatefulWidget for RegisterWidget {
    type State = Leaderboard;

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}

impl StatefulWidget for LoggedWidget {
    type State = Leaderboard;

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
