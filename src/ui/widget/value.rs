use super::*;

use crate::{ui::layout::AreaOps, util::SecondOrderState};

use ctl_client::core::{prelude::Interpolatable, types::Name};
use geng_utils::conversions::Vec2RealConversions;

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

impl<T: Float + Interpolatable> ValueWidget<T> {
    pub fn new(text: impl Into<Name>, value: T, control: ValueControl<T>, scroll_by: T) -> Self {
        Self {
            state: WidgetState::new(),
            value_text: InputWidget::new(text).format(InputFormat::Float),
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

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext, state: &mut T) -> bool {
        self.update_impl(position, context, *state, state)
    }

    pub fn update_dynamic(
        &mut self,
        position: Aabb2<f32>,
        context: &UiContext,
        state: &mut SecondOrderState<T>,
    ) -> bool {
        self.update_impl(position, context, state.current, &mut state.target)
    }

    fn update_impl(
        &mut self,
        position: Aabb2<f32>,
        context: &UiContext,
        value: T,
        state: &mut T,
    ) -> bool {
        self.value = value;
        let mut target = *state;
        self.state.update(position, context);
        let mut main = position;

        let control_height = context.font_size * 0.2;
        let mut control = main.cut_bottom(control_height);

        if let ValueControl::Circle { .. } = self.control {
            let texture = context.context.assets.atlas.value_knob();
            let size = texture.size().as_f32() * context.geometry.pixel_scale;
            control = crate::ui::layout::align_aabb(size, control, vec2(0.5, 0.5));
        }

        self.control_state.update(control, context);

        // Drag value
        let mut controlling = self.control_state.pressed;
        if controlling {
            match self.control {
                ValueControl::Slider { min, max } => {
                    let pos = (context.cursor.position - self.control_state.position.center())
                        / self.control_state.position.size();
                    let t = (pos.x + 0.5).clamp(0.0, 1.0);
                    target = min + (max - min) * T::from_f32(t);
                }
                ValueControl::Circle { period, .. } => {
                    let pos = context.cursor.position - self.control_state.position.center();
                    let last_pos =
                        context.cursor.last_position - self.control_state.position.center();
                    let delta = last_pos.arg().angle_to(pos.arg()).as_radians();
                    let delta = T::from_f32(delta / std::f32::consts::TAU) * period;
                    target += delta;
                }
            }
        } else if self.state.hovered && context.cursor.scroll != 0.0 {
            // Scroll value
            controlling = true;
            let delta = T::from_f32(context.cursor.scroll.signum()) * self.scroll_by;
            target += delta;
        }
        context.update_focus(controlling);

        self.value_text.update(main, context);
        if self.value_text.editing {
            if let Ok(typed_value) = self.value_text.raw.parse::<f32>() {
                controlling = true;
                target = T::from_f32(typed_value);
            } // TODO: check error
        }

        // Check bounds
        match self.control {
            ValueControl::Slider { min, max } => target = target.clamp_range(min..=max),
            ValueControl::Circle { .. } => {}
        }

        // TODO: better formatting with decimal points
        let precision = T::from_f32(100.0);
        target = (target * precision).round() / precision;

        if !self.value_text.editing {
            self.value_text.sync(&format!("{}", target), context);
        }

        *state = target;
        controlling
    }
}

impl<T: 'static + Float> Widget for ValueWidget<T> {
    simple_widget_state!();
    fn draw(&self, context: &UiContext) -> Geometry {
        let theme = context.theme();

        let mut fg_color = theme.light;
        if self.state.hovered {
            fg_color = theme.highlight;
        }

        let mut geometry = self.value_text.draw_colored(context, fg_color);

        let quad = self.control_state.position;
        let width = quad.height() * 0.05;
        let quad = quad.extend_uniform(-width);
        match self.control {
            ValueControl::Slider { min, max } => {
                let t = (self.value - min) / (max - min);
                let mut fill = quad;
                let fill = fill.cut_left(fill.width() * t.as_f32());

                let tick = |t: f32| quad.align_pos(vec2(t, 0.5));

                geometry.merge(context.geometry.texture_pp(
                    tick(0.0),
                    theme.highlight,
                    0.5,
                    &context.context.assets.atlas.timeline_tick_smol(),
                ));
                geometry.merge(context.geometry.texture_pp(
                    tick(t.as_f32()),
                    theme.highlight,
                    0.5,
                    &context.context.assets.atlas.timeline_tick_tiny(),
                ));
                geometry.merge(context.geometry.texture_pp(
                    tick(1.0),
                    theme.light,
                    0.5,
                    &context.context.assets.atlas.timeline_tick_smol(),
                ));

                geometry.merge(context.geometry.quad(fill, theme.highlight));
                geometry.merge(context.geometry.quad(quad, theme.light));
            }
            ValueControl::Circle { zero_angle, period } => {
                let angle =
                    Angle::from_radians((self.value / period).as_f32() * std::f32::consts::TAU);
                let angle = zero_angle + angle;

                let texture = context.context.assets.atlas.value_knob();
                geometry.merge(context.geometry.texture(
                    quad,
                    mat3::rotate_around(quad.center(), angle),
                    theme.light,
                    &texture,
                ));
            }
        }

        geometry
    }
}
