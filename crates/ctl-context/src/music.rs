use std::collections::VecDeque;

use super::*;

use ctl_core::types::time_to_seconds;
use ctl_util::{SecondOrderState, Task};
use geng::prelude::futures::channel::mpsc::{Receiver, Sender};

pub struct MusicManager {
    inner: RefCell<MusicManagerImpl>,
}

struct MusicManagerImpl {
    geng: Geng,
    volume: SecondOrderState<f32>,
    playing: Option<PlayingMusic>,
}

enum PlayingMusic {
    Static(Music),
    Streaming(MusicStreaming),
}

impl MusicManager {
    pub fn new(geng: Geng) -> Self {
        Self {
            inner: RefCell::new(MusicManagerImpl {
                geng,
                volume: SecondOrderState::new(3.0, 1.0, 0.0, 0.5),
                playing: None,
            }),
        }
    }

    pub fn current_static(&self) -> Option<Rc<LocalMusic>> {
        self.inner
            .borrow()
            .playing
            .as_ref()
            .and_then(|music| match music {
                PlayingMusic::Static(music) => Some(music.local.clone()),
                PlayingMusic::Streaming(_) => None,
            })
    }

    pub fn set_volume(&self, volume: f32) {
        let mut inner = self.inner.borrow_mut();
        inner.volume.snap_to(volume, 1.0 / 60.0);
        if let Some(music) = &mut inner.playing {
            match music {
                PlayingMusic::Static(music) => music.set_volume(volume),
                PlayingMusic::Streaming(music) => music.set_volume(volume),
            }
        }
    }

    pub fn fade_to_volume(&self, volume: f32) {
        let mut inner = self.inner.borrow_mut();
        inner.volume.target = volume;
    }

    pub fn set_speed(&self, speed: f32) {
        let mut inner = self.inner.borrow_mut();
        if let Some(music) = &mut inner.playing {
            match music {
                PlayingMusic::Static(music) => {
                    if let Some(effect) = &mut music.effect {
                        effect.set_speed(speed);
                    }
                }
                PlayingMusic::Streaming(music) => todo!(),
            }
        }
    }

    pub fn update(&self, delta_time: f32) {
        let mut inner = self.inner.borrow_mut();
        inner.volume.update(delta_time);
        let volume = inner.volume.current;
        if let Some(music) = &mut inner.playing {
            match music {
                PlayingMusic::Static(music) => music.set_volume(volume),
                PlayingMusic::Streaming(music) => {
                    music.poll();
                    music.set_volume(volume)
                }
            }
        }
    }

    pub fn stop(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(music) = &mut inner.playing {
            match music {
                PlayingMusic::Static(music) => music.stop(),
                PlayingMusic::Streaming(music) => music.stop(),
            }
        }
    }

    pub fn is_playing_static(&self) -> Option<Id> {
        self.inner
            .borrow()
            .playing
            .as_ref()
            .and_then(|music| match music {
                PlayingMusic::Static(music) => {
                    music.effect.is_some().then_some(music.local.meta.id)
                }
                PlayingMusic::Streaming(_) => None,
            })
    }

    pub fn switch(&self, music: &Rc<LocalMusic>, looped: bool) {
        if self.inner.borrow().playing.as_ref().is_none_or(|playing| {
            if let PlayingMusic::Static(playing) = playing {
                playing.effect.is_none() || !Rc::ptr_eq(&playing.local.sound, &music.sound)
            } else {
                true
            }
        }) {
            self.play(music, looped);
        }
    }

    // pub fn restart_from(&self, time: Duration) {
    //     let mut inner = self.inner.borrow_mut();
    //     if let Some(music) = &mut inner.playing {
    //         music.play_from(time);
    //     }
    // }

    pub fn play(&self, music: &Rc<LocalMusic>, looped: bool) {
        self.play_from(music, Duration::from_secs_f64(0.0), looped)
    }

    pub fn play_from(&self, music: &Rc<LocalMusic>, time: Duration, looped: bool) {
        let mut inner = self.inner.borrow_mut();
        let mut music = Music::new(inner.geng.clone(), music.clone());
        music.set_volume(inner.volume.current);
        music.play_from(time, looped);
        inner.playing = Some(PlayingMusic::Static(music));
    }

    pub fn play_from_time(&self, music: &Rc<LocalMusic>, time: Time, looped: bool) {
        let time = time_to_seconds(time);
        let time = Duration::from_secs_f64(time.as_f32().into());
        self.play_from(music, time, looped)
    }

    pub fn play_streaming(&self, mut music: MusicStreaming) {
        let mut inner = self.inner.borrow_mut();
        music.set_volume(inner.volume.current);
        music.start();
        inner.playing = Some(PlayingMusic::Streaming(music));
    }
}

pub struct MusicStreaming {
    geng: Geng,
    /// The stream processing task the sends processed chunks.
    stream: Option<Task<()>>,
    /// The actual playback task that is playing the sound chunks.
    playback: Option<Task<()>>,
    /// Handle to control playback.
    playback_handle: Sender<MusicControl>,
}

struct MusicStreamingBuffer {
    recv_control: Receiver<MusicControl>,
    recv_stream: Receiver<geng::Sound>,
    buffer: VecDeque<geng::Sound>,
}

enum MusicControl {
    Start,
    Stop,
}

impl MusicStreaming {
    pub fn new_speed(geng: &Geng, music: &Rc<LocalMusic>, start_time: Time, speed: f32) -> Self {
        let (mut send_stream, recv_stream) = futures::channel::mpsc::channel(0);
        let stream = {
            let music = Rc::clone(music);
            let geng = geng.clone();
            let start_time =
                time::Duration::from_secs_f64(time_to_seconds(start_time).as_f32().into());
            async move {
                log::debug!("spawned stream processor");
                match ctl_util::change_sound_speed(
                    &music.sound,
                    dbg!(speed),
                    &geng,
                    Some(start_time),
                ) {
                    Err(err) => {
                        log::error!("Failed to change music speed: {:?}", err)
                    }
                    Ok(chunk) => {
                        let _ = send_stream.try_send(chunk);
                    }
                }
            }
        };
        let (send_control, recv_control) = futures::channel::mpsc::channel(0);
        let playback = {
            MusicStreamingBuffer {
                recv_control,
                recv_stream,
                buffer: VecDeque::new(),
            }
            .run()
        };

        Self {
            geng: geng.clone(),
            stream: Some(Task::new(geng, stream)),
            playback: Some(Task::new(geng, playback)),
            playback_handle: send_control,
        }
    }

    fn poll(&mut self) {
        if let Some(task) = self.stream.take()
            && let Err(task) = task.poll()
        {
            self.stream = Some(task);
        }
        if let Some(task) = self.playback.take()
            && let Err(task) = task.poll()
        {
            self.playback = Some(task);
        }
    }

    fn start(&mut self) {
        log::debug!("start");
        let _ = self.playback_handle.try_send(MusicControl::Start);
    }

    fn stop(&mut self) {
        let _ = self.playback_handle.try_send(MusicControl::Stop);
    }

    fn set_volume(&mut self, volume: f32) {
        // TODO
        // let _ = self.playback_handle.try_send(MusicControl::SetVolume(volume));
    }
}

impl MusicStreamingBuffer {
    async fn run(mut self) {
        log::debug!("spawned stream buffer");
        while self.update() {}
    }

    /// Returns `false` when it's safe to drop the buffer.
    fn update(&mut self) -> bool {
        match self.recv_control.try_recv() {
            Err(err) => match err {
                futures::channel::mpsc::TryRecvError::Empty => {}
                futures::channel::mpsc::TryRecvError::Closed => return false,
            },
            Ok(control) => self.control(control),
        }

        match self.recv_stream.try_recv() {
            Err(err) => match err {
                futures::channel::mpsc::TryRecvError::Empty => {}
                futures::channel::mpsc::TryRecvError::Closed => return false,
            },
            Ok(chunk) => self.add_chunk(chunk),
        }

        true
    }

    fn control(&mut self, control: MusicControl) {
        log::debug!("received control signal");
        match control {
            MusicControl::Start => {
                // TODO
                if let Some(s) = self.buffer.front() {
                    s.play();
                }
            }
            MusicControl::Stop => todo!(),
        }
    }

    fn add_chunk(&mut self, chunk: geng::Sound) {
        log::debug!("chunk processed");
        self.buffer.push_back(chunk);
        self.control(MusicControl::Start); // TODO: temporary manual start
    }
}

pub struct Music {
    geng: Geng,
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
        let mut m = Self::new(self.geng.clone(), self.local.clone());
        m.set_volume(self.volume);
        m
    }
}

impl Music {
    pub fn new(geng: Geng, local: Rc<LocalMusic>) -> Self {
        Self {
            geng,
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

    pub fn play_from(&mut self, time: time::Duration, looped: bool) {
        self.stop();
        let mut effect = self.local.sound.effect(self.geng.audio().default_type());
        effect.set_volume(self.volume);
        effect.play_from(time);
        effect.set_looped(looped);
        self.effect = Some(effect);
    }
}
