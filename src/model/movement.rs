use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movement<T: Float = Time> {
    pub key_frames: VecDeque<MoveFrame<T>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveFrame<T: Float = Time> {
    /// How long should the interpolation from the last frame to that frame last.
    pub lerp_time: T,
    pub transform: Transform,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct Transform {
    pub translation: vec2<Coord>,
    pub rotation: Angle<Coord>,
    pub scale: Coord,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            translation: vec2::ZERO,
            rotation: Angle::ZERO,
            scale: Coord::ONE,
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

impl<T: Float> MoveFrame<T> {
    pub fn with_beat_time(self, beat_time: Time) -> MoveFrame<Time> {
        MoveFrame {
            lerp_time: Time::new(self.lerp_time.as_f32()) * beat_time,
            transform: self.transform,
        }
    }
}

impl<T: Float> Movement<T> {
    pub fn with_beat_time(self, beat_time: Time) -> Movement<Time> {
        Movement {
            key_frames: self
                .key_frames
                .into_iter()
                .map(|m| m.with_beat_time(beat_time))
                .collect(),
        }
    }
}

impl Movement {
    /// Get the transform at the given time.
    pub fn get(&self, mut time: Time) -> Transform {
        let mut from = Transform::identity();
        for frame in &self.key_frames {
            // Translation and rotation are accumulating
            let target = Transform {
                translation: from.translation + frame.transform.translation,
                rotation: from.rotation + frame.transform.rotation,
                ..frame.transform
            };

            if time <= frame.lerp_time {
                let t = if frame.lerp_time > Time::ZERO {
                    time / frame.lerp_time
                } else {
                    Time::ONE
                };
                let t = crate::util::smoothstep(t);
                return from.lerp(&target, t);
            }
            time -= frame.lerp_time;

            from = target;
        }
        from
    }

    /// Get the transform at the end of the movement.
    pub fn get_finish(&self) -> Transform {
        let mut result = Transform::identity();
        for frame in &self.key_frames {
            result = Transform {
                // Translation and rotation are accumulating
                translation: result.translation + frame.transform.translation,
                rotation: result.rotation + frame.transform.rotation,
                ..frame.transform
            };
        }
        result
    }

    /// Returns the total duration of the movement.
    pub fn duration(&self) -> Time {
        self.key_frames
            .iter()
            .map(|frame| frame.lerp_time)
            .fold(Time::ZERO, Time::add)
    }
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            key_frames: VecDeque::new(),
        }
    }
}
