use super::*;

impl Model {
    pub fn handle_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::Rhythm { perfect } => {
                self.rhythms.push(Rhythm {
                    time: Bounded::new_zero(Time::ONE),
                    perfect,
                });
            }
        }
    }
}
