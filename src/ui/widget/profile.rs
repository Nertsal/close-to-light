use ctl_client::core::auth::Credentials;

use super::*;

use crate::{leaderboard::Leaderboard, ui::layout::AreaOps};

pub struct ProfileWidget {
    pub state: WidgetState,
    pub window: UiWindow<()>,
    pub offline: TextWidget,
    pub register: RegisterWidget,
    pub logged: LoggedWidget,
}

pub struct RegisterWidget {
    pub state: WidgetState,
    pub username: InputWidget,
    pub password: InputWidget,
    pub login: ButtonWidget,
    pub register: ButtonWidget,
}

pub struct LoggedWidget {
    pub state: WidgetState,
    pub username: TextWidget,
    pub logout: ButtonWidget,
}

impl ProfileWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            window: UiWindow::new((), 0.3),
            offline: TextWidget::new("Offline"),
            register: RegisterWidget {
                state: WidgetState::new(),
                username: InputWidget::new("Username", false),
                password: InputWidget::new("Password", true),
                login: ButtonWidget::new("Login"),
                register: ButtonWidget::new("Register"),
            },
            logged: LoggedWidget {
                state: WidgetState::new(),
                username: TextWidget::new("<username>"),
                logout: ButtonWidget::new("Logout"),
            },
        }
    }
}

impl StatefulWidget for ProfileWidget {
    type State = Leaderboard;

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);
        self.window.update(context.delta_time);

        let margin = context.layout_size * 0.5;
        let main = position.extend_uniform(-margin);

        let (off, reg, log) = match (state.client().is_some(), state.user.is_some()) {
            (false, _) => (true, false, false),
            (true, false) => (false, true, false),
            (true, true) => (false, false, true),
        };
        if off {
            self.offline.show();
            self.offline.update(main, context);
        } else {
            self.offline.hide();
        }
        if reg {
            self.register.show();
            self.register.update(main, context, state);
        } else {
            self.register.hide();
        }
        if log {
            self.logged.show();
            self.logged.update(main, context, state);
        } else {
            self.logged.hide();
        }
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

        let main = position;

        let rows = main.split_rows(3);
        self.username.update(rows[0], context);
        self.password.update(rows[1], context);

        let cols = rows[2].split_columns(2);
        self.login.update(cols[0], context);
        self.register.update(cols[1], context);

        let creds = Credentials {
            username: self.username.text.text.clone(),
            password: self.password.text.text.clone(),
        };
        if self.login.text.state.clicked {
            state.login(creds);
        } else if self.register.text.state.clicked {
            state.register(creds);
        }
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}

impl StatefulWidget for LoggedWidget {
    type State = Leaderboard;

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);

        if let Some(user) = &state.user {
            self.username.text = user.to_owned();
        }

        let main = position;

        let rows = main.split_rows(2);
        self.username.update(rows[0], context);
        self.logout.update(rows[1], context);

        if self.logout.text.state.clicked {
            state.logout();
        }
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
