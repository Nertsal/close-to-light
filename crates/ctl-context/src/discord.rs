use std::{rc::Rc, sync::mpsc::Sender};

use geng::prelude::log;

#[derive(Clone)]
pub struct Client {
    sender: Rc<Sender<Option<String>>>,
}

impl Client {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<Option<String>>();
        std::thread::spawn(move || {
            let mut client = discord_presence::Client::new(ctl_constants::DISCORD_APP_ID);
            client.start();

            if let Err(err) = client.set_activity(|act| act.state("Jamming")) {
                log::error!("Failed to set discord rich presence: {:?}", err);
            }

            while let Ok(status) = receiver.recv() {
                let result = match status {
                    Some(status) if !status.is_empty() => {
                        client.set_activity(|act| act.state(status))
                    }
                    _ => client.set_activity(|act| act.state("Jamming")),
                };
                if let Err(err) = result {
                    log::error!("Failed to set discord rich presence: {:?}", err);
                }
            }
        });

        Self {
            sender: Rc::new(sender),
        }
    }

    pub fn set_status(&self, status: Option<&str>) {
        if let Err(err) = self.sender.send(status.map(|s| s.to_owned())) {
            log::error!("Failed to set discord rich presence: {:?}", err);
        }
    }
}
