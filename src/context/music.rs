use super::*;

pub struct MusicManager {
    inner: RefCell<MusicManagerImpl>,
}

struct MusicManagerImpl {
    volume: f32,
    playing: Option<Music>,
}

impl MusicManager {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(MusicManagerImpl {
                volume: 0.5,
                playing: None,
            }),
        }
    }

    pub fn current(&self) -> Option<Rc<LocalMusic>> {
        self.inner
            .borrow()
            .playing
            .as_ref()
            .map(|music| music.local.clone())
    }

    pub fn set_volume(&self, volume: f32) {
        let mut inner = self.inner.borrow_mut();
        inner.volume = volume;
        if let Some(music) = &mut inner.playing {
            music.set_volume(volume);
        }
    }

    pub fn set_speed(&self, speed: f32) {
        let mut inner = self.inner.borrow_mut();
        if let Some(music) = &mut inner.playing {
            if let Some(effect) = &mut music.effect {
                effect.set_speed(speed);
            }
        }
    }

    pub fn stop(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(music) = &mut inner.playing {
            music.stop();
        }
    }

    pub fn is_playing(&self) -> Option<Id> {
        self.inner
            .borrow()
            .playing
            .as_ref()
            .filter(|music| music.effect.is_some())
            .map(|music| music.local.meta.id)
    }

    pub fn switch(&self, music: &Rc<LocalMusic>) {
        if self.inner.borrow().playing.as_ref().is_none_or(|playing| {
            playing.effect.is_none() || !Rc::ptr_eq(&playing.local.sound, &music.sound)
        }) {
            self.play(music);
        }
    }

    // pub fn restart_from(&self, time: Duration) {
    //     let mut inner = self.inner.borrow_mut();
    //     if let Some(music) = &mut inner.playing {
    //         music.play_from(time);
    //     }
    // }

    pub fn play(&self, music: &Rc<LocalMusic>) {
        self.play_from(music, Duration::from_secs_f64(0.0))
    }

    pub fn play_from(&self, music: &Rc<LocalMusic>, time: Duration) {
        let mut inner = self.inner.borrow_mut();
        let mut music = Music::new(music.clone());
        music.set_volume(inner.volume);
        music.play_from(time);
        inner.playing = Some(music);
    }

    pub fn play_from_beat(&self, music: &Rc<LocalMusic>, beat: Time) {
        let time = Duration::from_secs_f64(
            beat as f64 * ctl_client::core::types::TIME_IN_FLOAT_TIME as f64,
        );
        self.play_from(music, time)
    }
}

pub struct Music {
    local: Rc<LocalMusic>,
    effect: Option<geng::SoundEffect>,
    volume: f32,
}

impl Drop for Music {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Debug for Music {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Music")
            .field("meta", &self.local.meta)
            // .field("effect", &self.effect)
            .field("volume", &self.volume)
            .finish()
    }
}

impl Clone for Music {
    fn clone(&self) -> Self {
        let mut m = Self::new(self.local.clone());
        m.set_volume(self.volume);
        m
    }
}

impl Music {
    pub fn new(local: Rc<LocalMusic>) -> Self {
        Self {
            local,
            volume: 0.5,
            effect: None,
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        self.volume = volume;
        if let Some(effect) = &mut self.effect {
            effect.set_volume(volume);
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut effect) = self.effect.take() {
            effect.stop();
        }
    }

    pub fn play_from(&mut self, time: time::Duration) {
        self.stop();
        let mut effect = self.local.sound.effect();
        effect.set_volume(self.volume);
        effect.play_from(time);
        self.effect = Some(effect);
    }
}
