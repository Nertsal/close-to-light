use super::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Movement {
    pub key_frames: VecDeque<MoveFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoveFrame {
    /// How long (in beats) should the interpolation from the last frame to that frame last.
    pub lerp_time: Time,
    pub transform: Transform,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

impl Movement {
    /// Iterate over frames with corrected (accumulated) transforms.
    pub fn frames_iter(&self) -> impl Iterator<Item = MoveFrame> + '_ {
        self.key_frames
            .iter()
            .scan(Transform::identity(), |trans, frame| {
                *trans = Transform {
                    translation: trans.translation + frame.transform.translation,
                    rotation: trans.rotation + frame.transform.rotation,
                    scale: frame.transform.scale,
                };
                Some(MoveFrame {
                    transform: *trans,
                    ..*frame
                })
            })
    }

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
