use super::*;

use ctl_core::types::time_to_seconds;
use ctl_util::SecondOrderState;

pub struct MusicManager {
    inner: RefCell<MusicManagerImpl>,
}

struct MusicManagerImpl {
    geng: Geng,
    volume: SecondOrderState<f32>,
    playing: Option<Music>,
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
            .map(|music| music.local.clone())
    }

    pub fn set_volume(&self, volume: f32) {
        let mut inner = self.inner.borrow_mut();
        inner.volume.snap_to(volume, 1.0 / 60.0);
        if let Some(music) = &mut inner.playing {
            music.set_volume(volume)
        }
    }

    pub fn fade_to_volume(&self, volume: f32) {
        let mut inner = self.inner.borrow_mut();
        inner.volume.target = volume;
    }

    pub fn set_speed(&self, speed: f32) {
        let mut inner = self.inner.borrow_mut();
        if let Some(music) = &mut inner.playing
            && let Some(MusicEffect::Static(effect)) = &mut music.effect
        {
            effect.set_speed(speed);
        }
    }

    pub fn update(&self, delta_time: f32) {
        let mut inner = self.inner.borrow_mut();
        inner.volume.update(delta_time);
        let volume = inner.volume.current;
        if let Some(music) = &mut inner.playing {
            music.set_volume(volume);
        }
    }

    pub fn stop(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(music) = &mut inner.playing {
            music.stop()
        }
    }

    pub fn is_playing_static(&self) -> Option<Id> {
        self.inner
            .borrow()
            .playing
            .as_ref()
            .and_then(|music| music.effect.is_some().then_some(music.local.meta.id))
    }

    pub fn switch(&self, music: &Rc<LocalMusic>, looped: bool) {
        if self.inner.borrow().playing.as_ref().is_none_or(|playing| {
            playing.effect.is_none() || !Rc::ptr_eq(&playing.local.sound, &music.sound)
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
        inner.playing = Some(music);
    }

    pub fn play_from_time(&self, music: &Rc<LocalMusic>, time: Time, looped: bool) {
        let time = time_to_seconds(time);
        let time = Duration::from_secs_f64(time.as_f32().into());
        self.play_from(music, time, looped)
    }

    pub fn play_from_with_speed(&self, music: &Rc<LocalMusic>, time: Duration, speed: f32) {
        let mut inner = self.inner.borrow_mut();
        let mut music = Music::new(inner.geng.clone(), music.clone());
        music.set_volume(inner.volume.current);
        music.play_from_with_speed(time, speed);
        inner.playing = Some(music);
    }

    pub fn play_from_time_with_speed(&self, music: &Rc<LocalMusic>, time: Time, speed: f32) {
        let time = time_to_seconds(time);
        let time = Duration::from_secs_f64(time.as_f32().into());
        self.play_from_with_speed(music, time, speed)
    }
}

// pub struct MusicStreaming {
//     /// The stream processing task the sends processed chunks.
//     stream: Option<Task<()>>,
//     /// The actual playback task that is playing the sound chunks.
//     playback: Option<Task<()>>,
//     /// Handle to control playback.
//     playback_handle: Sender<MusicControl>,
// }

// struct MusicStreamingBuffer {
//     geng: Geng,
//     recv_control: Receiver<MusicControl>,
//     recv_stream: Receiver<geng::Sound>,
//     buffer: VecDeque<geng::Sound>,
//     state: MusicStreamState,
//     playing: Option<(time::Duration, geng::SoundEffect)>,
//     queued: Option<(time::Duration, geng::SoundEffect)>,
// }

// enum MusicStreamState {
//     Paused,
//     Playing,
// }

// #[derive(Debug)]
// enum MusicControl {
//     Start,
//     Stop,
// }

// impl MusicStreaming {
//     pub fn new_speed(geng: &Geng, music: &Rc<LocalMusic>, start_time: Time, speed: f32) -> Self {
//         let (mut send_stream, recv_stream) = futures::channel::mpsc::channel(0);
//         let stream = {
//             let music = Rc::clone(music);
//             let geng = geng.clone();
//             let start_time =
//                 time::Duration::from_secs_f64(time_to_seconds(start_time).as_f32().into());
//             let sample_rate = music.sound.sample_rate();
//             async move {
//                 log::debug!("spawned stream processor");
//                 let iter = ctl_util::change_sound_speed_iter(&music.sound, speed, Some(start_time));
//                 for chunk in iter {
//                     match geng.audio().sound_from_buffer(chunk, sample_rate) {
//                         Err(err) => {
//                             log::error!("Failed to change music speed: {:?}", err)
//                         }
//                         Ok(chunk) => {
//                             let _ = send_stream.try_send(chunk);
//                         }
//                     }

//                     tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
//                 }
//             }
//         };
//         let (send_control, recv_control) = futures::channel::mpsc::channel(0);
//         let playback = {
//             MusicStreamingBuffer {
//                 geng: geng.clone(),
//                 recv_control,
//                 recv_stream,
//                 buffer: VecDeque::new(),
//                 state: MusicStreamState::Paused,
//                 playing: None,
//                 queued: None,
//             }
//             .run()
//         };

//         let m = Self {
//             stream: Some(Task::new(geng, stream)),
//             playback: Some(Task::new(geng, playback)),
//             playback_handle: send_control,
//         };
//         log::debug!("initialized streaming");
//         m
//     }

//     fn poll(&mut self) {
//         if let Some(task) = self.stream.take()
//             && let Err(task) = task.poll()
//         {
//             self.stream = Some(task);
//         }
//         if let Some(task) = self.playback.take()
//             && let Err(task) = task.poll()
//         {
//             self.playback = Some(task);
//         }
//     }

//     fn start(&mut self) {
//         let _ = self.playback_handle.try_send(MusicControl::Start);
//     }

//     fn stop(&mut self) {
//         let _ = self.playback_handle.try_send(MusicControl::Stop);
//     }

//     fn set_volume(&mut self, volume: f32) {
//         // TODO
//         // let _ = self.playback_handle.try_send(MusicControl::SetVolume(volume));
//     }
// }

// impl MusicStreamingBuffer {
//     async fn run(mut self) {
//         log::debug!("spawned stream buffer");
//         while self.update() {
//             tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
//         }
//         log::debug!("music stream ended");
//     }

//     /// Returns `false` when it's safe to drop the buffer.
//     fn update(&mut self) -> bool {
//         match self.recv_control.try_recv() {
//             Err(err) => match err {
//                 futures::channel::mpsc::TryRecvError::Empty => {}
//                 futures::channel::mpsc::TryRecvError::Closed => return false,
//             },
//             Ok(control) => self.control(control),
//         }

//         match self.recv_stream.try_recv() {
//             Err(err) => match err {
//                 futures::channel::mpsc::TryRecvError::Empty => {}
//                 futures::channel::mpsc::TryRecvError::Closed => {}
//             },
//             Ok(chunk) => self.add_chunk(chunk),
//         }

//         match self.state {
//             MusicStreamState::Paused => {}
//             MusicStreamState::Playing => {
//                 let next = if let Some((duration, effect)) = &mut self.playing {
//                     let time_left =
//                         duration.as_secs_f64() - effect.playback_position().as_secs_f64();
//                     (time_left <= 0.0).then_some(-time_left)
//                 } else {
//                     Some(0.0)
//                 };
//                 if let Some(offset) = next {
//                     if self.buffer.is_empty() {
//                         log::error!("empty buffer");
//                     }
//                     self.playing = self.buffer.pop_front().map(|sound| {
//                         let mut e = sound.effect(self.geng.audio().default_type());
//                         e.play_from(time::Duration::from_secs_f64(offset));
//                         (sound.duration(), e)
//                     });
//                 }

//                 match &self.playing {
//                     None => {
//                         self.playing = self.buffer.pop_front().map(|sound| {
//                             let mut e = sound.effect(self.geng.audio().default_type());
//                             e.play();
//                             (sound.duration(), e)
//                         });
//                     }
//                     Some((duration, effect)) => {
//                         let time_left =
//                             duration.as_secs_f64() - effect.playback_position().as_secs_f64();
//                         if self.queued.is_none() {
//                             self.queued = self.buffer.pop_front().map(|sound| {
//                                 let mut e = sound.effect(self.geng.audio().default_type());
//                                 e.play_at_from(
//                                     time::Duration::from_secs_f64(time_left.max(0.0)),
//                                     time::Duration::from_secs_f64(0.0),
//                                 );
//                                 (sound.duration(), e)
//                             });
//                         }
//                         if time_left <= 0.0 {
//                             self.playing = self.queued.take();
//                         }
//                     }
//                 }
//             }
//         }

//         true
//     }

//     fn control(&mut self, control: MusicControl) {
//         log::debug!("received control signal: {:?}", control);
//         match control {
//             MusicControl::Start => {
//                 self.state = MusicStreamState::Playing;
//             }
//             MusicControl::Stop => {
//                 if let Some((_, effect)) = &mut self.playing {
//                     effect.stop();
//                 }
//                 self.state = MusicStreamState::Paused;
//             }
//         }
//     }

//     fn add_chunk(&mut self, chunk: geng::Sound) {
//         self.buffer.push_back(chunk);
//         if self.buffer.len() == 1 {
//             self.control(MusicControl::Start); // TODO: temporary manual start
//         }
//     }
// }

enum MusicEffect {
    Static(geng::SoundEffect),
    #[cfg(not(target_arch = "wasm32"))]
    Stream(geng::StreamingSoundEffect),
}

pub struct Music {
    geng: Geng,
    local: Rc<LocalMusic>,
    effect: Option<MusicEffect>,
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
            match effect {
                MusicEffect::Static(effect) => effect.set_volume(volume),
                #[cfg(not(target_arch = "wasm32"))]
                MusicEffect::Stream(effect) => effect.set_volume(volume),
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut effect) = self.effect.take() {
            match &mut effect {
                MusicEffect::Static(effect) => effect.stop(),
                #[cfg(not(target_arch = "wasm32"))]
                MusicEffect::Stream(effect) => effect.stop(),
            }
        }
    }

    pub fn play_from(&mut self, time: time::Duration, looped: bool) {
        self.stop();
        let mut effect = self.local.sound.effect(self.geng.audio().default_type());
        effect.set_volume(self.volume);
        effect.play_from(time);
        effect.set_looped(looped);
        self.effect = Some(MusicEffect::Static(effect));
    }

    pub fn play_from_with_speed_pitch_shifted(&mut self, time: time::Duration, speed: f32) {
        let speed = speed.clamp(0.2, 5.0);
        self.stop();
        let mut effect = self.local.sound.effect(self.geng.audio().default_type());
        effect.set_volume(self.volume);
        effect.play_from(time);
        effect.set_speed(speed);
        self.effect = Some(MusicEffect::Static(effect));
    }

    // TODO; same as native (pitch-preserved)
    #[cfg(target_arch = "wasm32")]
    pub fn play_from_with_speed(&mut self, time: time::Duration, speed: f32) {
        self.play_from_with_speed_pitch_shifted(time, speed)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn play_from_with_speed(&mut self, time: time::Duration, speed: f32) {
        let speed = speed.clamp(0.2, 5.0);
        self.stop();

        let channels_n = self.local.sound.number_of_channels();
        let mut channels = Vec::with_capacity(channels_n as usize);
        for i in 0..channels_n {
            channels.push(self.local.sound.get_channel_data(i as u32).to_vec());
        }

        let mut effect = self
            .geng
            .audio()
            .timestretch(
                channels,
                self.local.sound.sample_rate(),
                speed,
                self.geng.audio().default_type(),
            )
            .expect("failed to timestretch audio");
        effect.set_volume(self.volume);
        effect.play_from(time);
        self.effect = Some(MusicEffect::Stream(effect));
    }
}
