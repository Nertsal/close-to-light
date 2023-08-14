use super::*;

impl Model {
    pub(super) fn process_events(&mut self, delta_time: Time) {
        let mut to_process = Vec::new();
        for event in &mut self.queued_events {
            event.delay -= delta_time;
            if event.delay <= Time::ZERO {
                to_process.push(event.event.clone());
            }
        }
        self.queued_events.retain(|e| e.delay > Time::ZERO);

        for event in to_process {
            self.process_event(event);
        }
    }

    pub fn process_event(&mut self, event: Event) {
        match event {
            Event::Light(event) => {
                self.telegraphs.push(
                    event
                        .light
                        .instantiate(self.level.beat_time())
                        .into_telegraph(event.telegraph, self.level.beat_time()),
                );
            }
        }
    }
}
