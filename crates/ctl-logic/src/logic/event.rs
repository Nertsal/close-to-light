use super::*;

impl Model {
    pub fn handle_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::Rhythm { perfect } => {
                // Collect rhythm
                if let Some((event, light)) = self.player.closest_light.and_then(|id| {
                    self.level_state
                        .lights
                        .iter()
                        .find(|light| light.event_id == Some(id))
                        .map(|light| (id, light))
                }) {
                    self.last_rhythm = (event, light.closest_waypoint.1);
                }

                let position = self.player.collider.position;
                self.rhythms.push(Rhythm {
                    position,
                    time: Bounded::new_zero(TIME_IN_FLOAT_TIME / 2),
                    perfect,
                });
            }
        }
    }

    pub fn handle_level_event(&mut self, event: &TimedEvent) {
        match event.event {
            Event::Light(_) | Event::PaletteSwap => {}
            Event::RgbSplit(duration) => {
                self.vfx.rgb_split.time_left = time_to_seconds(duration);
            }
        }
    }
}
