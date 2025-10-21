use super::*;

impl Model {
    pub fn handle_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::Rhythm { perfect } => {
                // Collect rhythm
                let light_rhythm = |id: usize| {
                    self.level_state
                        .lights
                        .iter()
                        .find(|light| light.event_id == Some(id))
                        .map(|light| {
                            (
                                (id, light.closest_waypoint.1),
                                self.level_state.time() + light.closest_waypoint.0,
                            )
                        })
                };
                if perfect && !self.player.perfect_waypoints.is_empty() {
                    self.recent_rhythm.extend(
                        self.player
                            .perfect_waypoints
                            .iter()
                            .flat_map(|&id| light_rhythm(id)),
                    );
                } else if let Some((light, time)) = self.player.closest_light.and_then(light_rhythm)
                {
                    self.recent_rhythm.insert(light, time);
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
}
