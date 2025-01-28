use super::*;

use crate::ui::layout::AreaOps;

use ctl_client::core::types::{BeatTime, Name, Time};
use num_rational::Ratio;

pub struct BeatValueWidget {
    pub state: WidgetState,
    pub value_text: InputWidget,
    pub control_state: WidgetState,
    pub range: RangeInclusive<BeatTime>,
    pub value: BeatTime,
    pub scroll_by: BeatTime,
}

impl BeatValueWidget {
    pub fn new(
        text: impl Into<Name>,
        value: BeatTime,
        range: RangeInclusive<BeatTime>,
        scroll_by: BeatTime,
    ) -> Self {
        Self {
            state: WidgetState::new(),
            value_text: InputWidget::new(text).format(InputFormat::Ratio),
            control_state: WidgetState::new(),
            range,
            value,
            scroll_by,
        }
    }

    pub fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &UiContext,
        state: &mut BeatTime,
    ) -> bool {
        self.value = *state;
        let mut target = *state;
        self.state.update(position, context);
        let mut main = position;

        let control_height = context.font_size * 0.2;
        let control = main.cut_bottom(control_height);
        self.control_state.update(control, context);
        let (min, max) = (*self.range.start(), *self.range.end());

        // Drag value
        let mut controlling = self.control_state.pressed;
        if controlling {
            // (0,0) in the center, range -0.5..=0.5
            let convert = |pos| {
                (pos - self.control_state.position.center()) / self.control_state.position.size()
            };
            let pos = convert(context.cursor.position);
            let t = (pos.x + 0.5).clamp(0.0, 1.0);
            let steps = (max - min).units() as f32 / self.scroll_by.units() as f32 * t;
            target = min + self.scroll_by * steps.round() as Time;
        } else if self.state.hovered && context.cursor.scroll != 0.0 {
            // Scroll value
            controlling = true;
            let delta = self.scroll_by * context.cursor.scroll.signum() as Time;
            target += delta;
        }

        self.value_text.update(main, context);
        if self.value_text.editing {
            // TODO: handle errors
            if let Some((num, den)) = self.value_text.raw.split_once('/') {
                if let Ok(num) = num.parse::<Time>() {
                    if let Ok(den) = den.parse::<Time>() {
                        if 16 % den == 0 {
                            let units = num * (16 / den);
                            target = BeatTime::from_units(units);
                        }
                    }
                }
            }
        }

        // Check bounds
        target = target.clamp_range(min..=max);

        if !self.value_text.editing {
            let value = Ratio::new_raw(target.units(), BeatTime::UNITS_PER_BEAT).reduced();
            self.value_text
                .sync(&format!("{}/{}", value.numer(), value.denom()), context);
        }

        *state = target;
        controlling
    }
}

impl Widget for BeatValueWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let theme = context.theme();
        let mut geometry = self.value_text.draw(context);

        let quad = self.control_state.position;
        let width = quad.height() * 0.05;
        let quad = quad.extend_uniform(-width);

        {
            let (min, max) = (*self.range.start(), *self.range.end());
            let t = (self.value - min).units() as f32 / (max - min).units() as f32;
            let mut fill = quad;
            let fill = fill.cut_left(fill.width() * t);

            let tick = |t: f32| quad.align_pos(vec2(t, 0.5));

            geometry.merge(context.geometry.texture_pp(
                tick(0.0),
                theme.highlight,
                0.5,
                &context.context.assets.atlas.timeline_tick_smol(),
            ));
            geometry.merge(context.geometry.texture_pp(
                tick(t),
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

        geometry
    }
}
