use super::*;

use crate::ui::layout::AreaOps;

use ctl_client::core::types::Name;

pub struct ValueWidget<T> {
    pub state: WidgetState,
    pub value_text: InputWidget,
    pub control_state: WidgetState,
    pub value: T,
    pub control: ValueControl<T>,
    pub scroll_by: T,
}

#[derive(Debug, Clone)]
pub enum ValueControl<T> {
    Slider { min: T, max: T },
    Circle { zero_angle: Angle, period: T },
}

impl<T: Float> ValueWidget<T> {
    pub fn new(text: impl Into<Name>, value: T, control: ValueControl<T>, scroll_by: T) -> Self {
        Self {
            state: WidgetState::new(),
            value_text: InputWidget::new(text).vertical().format(InputFormat::Float),
            control_state: WidgetState::new(),
            value,
            control,
            scroll_by,
        }
    }

    pub fn new_range(
        text: impl Into<Name>,
        value: T,
        range: RangeInclusive<T>,
        scroll_by: T,
    ) -> Self {
        Self::new(
            text,
            value,
            ValueControl::Slider {
                min: *range.start(),
                max: *range.end(),
            },
            scroll_by,
        )
    }

    pub fn new_circle(text: impl Into<Name>, value: T, period: T, scroll_by: T) -> Self {
        Self::new(
            text,
            value,
            ValueControl::Circle {
                zero_angle: Angle::ZERO,
                period,
            },
            scroll_by,
        )
    }
}

impl<T: Float> StatefulWidget for ValueWidget<T> {
    type State<'a> = T;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &mut UiContext,
        state: &mut Self::State<'_>,
    ) {
        self.value = *state;
        self.state.update(position, context);
        let mut main = position;

        let control_width = main.height() / 2.0;
        let control = main.cut_right(control_width);
        self.control_state.update(control, context);

        // Drag value
        if self.control_state.pressed {
            // (0,0) in the center, range -0.5..=0.5
            let convert = |pos| {
                (pos - self.control_state.position.center()) / self.control_state.position.size()
            };
            let pos = convert(context.cursor.position);
            match self.control {
                ValueControl::Slider { min, max } => {
                    let t = (pos.y + 0.5).clamp(0.0, 1.0);
                    self.value = min + (max - min) * T::from_f32(t);
                }
                ValueControl::Circle { period, .. } => {
                    let last_pos = convert(context.cursor.last_position);
                    let delta = last_pos.arg().angle_to(pos.arg()).as_radians();
                    let delta = T::from_f32(delta / std::f32::consts::TAU) * period;
                    self.value += delta;
                }
            }
        } else if self.control_state.hovered && context.cursor.scroll != 0.0 {
            // Scroll value
            let delta = T::from_f32(context.cursor.scroll.signum()) * self.scroll_by;
            self.value += delta;
        }

        self.value_text.update(main, context);
        if self.value_text.editing {
            if let Ok(typed_value) = self.value_text.raw.parse::<f32>() {
                self.value = T::from_f32(typed_value);
            } // TODO: check error
        }

        // Check bounds
        match self.control {
            ValueControl::Slider { min, max } => self.value = self.value.clamp_range(min..=max),
            ValueControl::Circle { .. } => {}
        }

        // TODO: better formatting with decimal points
        let precision = T::from_f32(100.0);
        self.value = (self.value * precision).round() / precision;

        if !self.value_text.editing {
            self.value_text.sync(&format!("{}", self.value), context);
        }

        *state = self.value;
    }
}
