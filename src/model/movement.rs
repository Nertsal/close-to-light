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
            rotation: self.rotation + (target.rotation - self.rotation) * t,
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
            if time <= frame.lerp_time {
                let t = if frame.lerp_time > Time::ZERO {
                    time / frame.lerp_time
                } else {
                    Time::ONE
                };
                let t = crate::util::smoothstep(t);
                return from.lerp(&frame.transform, t);
            }
            time -= frame.lerp_time;
            from = frame.transform;
        }
        from
    }
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            key_frames: VecDeque::new(),
        }
    }
}
