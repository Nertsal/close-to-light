use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Movement {
    /// Time (in beats) to spend fading into the initial position.
    pub fade_in: Time,
    /// Time (in beats) to spend fading out of the last keyframe.
    pub fade_out: Time,
    pub initial: Transform,
    pub key_frames: VecDeque<MoveFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoveFrame {
    /// How long (in beats) should the interpolation from the last frame to that frame last.
    pub lerp_time: Time,
    pub transform: Transform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WaypointId {
    Initial,
    Frame(usize),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Transform {
    pub translation: vec2<Coord>,
    pub rotation: Angle<Coord>,
    pub scale: Coord,
}

impl MoveFrame {
    pub fn scale(lerp_time: impl Float, scale: impl Float) -> Self {
        Self {
            lerp_time: lerp_time.as_r32(),
            transform: Transform::scale(scale),
        }
    }
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            translation: vec2::ZERO,
            rotation: Angle::ZERO,
            scale: Coord::ONE,
        }
    }

    pub fn scale(scale: impl Float) -> Self {
        Self {
            scale: scale.as_r32(),
            ..Self::identity()
        }
    }

    pub fn lerp(&self, target: &Self, t: Time) -> Self {
        Self {
            translation: self.translation + (target.translation - self.translation) * t,
            rotation: self.rotation + self.rotation.angle_to(target.rotation) * t,
            scale: self.scale + (target.scale - self.scale) * t,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            fade_in: r32(1.0),
            fade_out: r32(1.0),
            initial: default(),
            key_frames: default(),
        }
    }
}

impl Movement {
    /// Iterate over frames with corrected (accumulated) transforms.
    pub fn frames_iter(&self) -> impl Iterator<Item = &MoveFrame> {
        self.key_frames.iter()
    }

    /// Iterate over all key transformations (including initial)
    /// together with their start times.
    pub fn timed_positions(&self) -> impl Iterator<Item = (WaypointId, Transform, Time)> + '_ {
        std::iter::once((WaypointId::Initial, self.initial, self.fade_in))
            .chain(
                self.frames_iter()
                    .enumerate()
                    .map(|(i, frame)| (WaypointId::Frame(i), frame.transform, frame.lerp_time)),
            )
            .scan(Time::ZERO, |time, (i, trans, duration)| {
                *time += duration;
                Some((i, trans, *time))
            })
    }

    pub fn get_frame_mut(&mut self, id: WaypointId) -> Option<&mut Transform> {
        match id {
            WaypointId::Initial => Some(&mut self.initial),
            WaypointId::Frame(i) => self.key_frames.get_mut(i).map(|frame| &mut frame.transform),
        }
    }

    pub fn get_time(&self, id: WaypointId) -> Option<Time> {
        let i = match id {
            WaypointId::Initial => 0,
            WaypointId::Frame(i) => i + 1,
        };
        self.timed_positions().nth(i).map(|(_, _, time)| time)
    }

    /// Whether currently at a waypoint with specified precision.
    pub fn is_at_waypoint(&self, time: Time, precision: Time) -> bool {
        self.timed_positions()
            .any(|(_, _, key_time)| (key_time - time).abs() <= precision)
    }

    /// Get the transform at the given time.
    pub fn get(&self, mut time: Time) -> Transform {
        let mut from = self.initial;

        let lerp = |from: Transform, to, time, duration| {
            let t = if duration > Time::ZERO {
                time / duration
            } else {
                Time::ONE
            };
            let t = crate::util::smoothstep(t);
            from.lerp(&to, t)
        };

        // Fade in
        if time <= self.fade_in {
            return lerp(
                Transform {
                    scale: Coord::ZERO,
                    ..from
                },
                from,
                time,
                self.fade_in,
            );
        }
        time -= self.fade_in;

        for frame in self.frames_iter() {
            if time <= frame.lerp_time {
                return lerp(from, frame.transform, time, frame.lerp_time);
            }
            time -= frame.lerp_time;
            from = frame.transform;
        }

        // Fade out
        let target = Transform {
            scale: Coord::ZERO,
            ..from
        };
        if time <= self.fade_out {
            lerp(from, target, time, self.fade_out)
        } else {
            target
        }
    }

    /// Returns the total duration of the movement including fade in/out.
    pub fn total_duration(&self) -> Time {
        self.fade_in + self.movement_duration() + self.fade_out
    }

    /// Returns the duration of the movement excluding fade in/out.
    pub fn movement_duration(&self) -> Time {
        self.key_frames
            .iter()
            .map(|frame| frame.lerp_time)
            .fold(Time::ZERO, Time::add)
    }
}
