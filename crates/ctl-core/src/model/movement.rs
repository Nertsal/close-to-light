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
    /// Interpolation to use when moving towards this frame.
    pub interpolation: MoveInterpolation,
    /// Whether to start a new curve starting from this frame.
    /// Is set to `None`, the curve will either continue the previous type,
    /// or continue as linear in the case of bezier.
    pub change_curve: Option<TrajectoryInterpolation>,
    pub transform: Transform,
}

/// Controls the speed of the light when moving between keyframes.
/// Default is Smoothstep.
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MoveInterpolation {
    Linear,
    #[default]
    Smoothstep,
    EaseIn,
    EaseOut,
}

impl MoveInterpolation {
    /// Applies the interpolation function to a value between 0 and 1.
    pub fn apply(&self, t: Time) -> Time {
        match self {
            Self::Linear => t,
            Self::Smoothstep => smoothstep(t),
            Self::EaseIn => ease_in(t),
            Self::EaseOut => ease_out(t),
        }
    }
}

/// Controls the overall trajectory of the light based on the keyframes.
/// Default is Linear.
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrajectoryInterpolation {
    /// Connects keyframes in a straight line.
    #[default]
    Linear,
    /// Connects keyframes via a smooth cubic Cardinal spline.
    Spline { tension: R32 },
    /// Connects keyframes via a Bezier curve.
    Bezier,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum WaypointId {
    Initial,
    Frame(usize),
}

impl WaypointId {
    pub fn prev(self) -> Option<Self> {
        match self {
            Self::Initial => None,
            Self::Frame(0) => Some(Self::Initial),
            Self::Frame(i) => Some(Self::Frame(i - 1)),
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Initial => Self::Frame(0),
            Self::Frame(i) => Self::Frame(i + 1),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Transform {
    pub translation: vec2<Coord>,
    pub rotation: Angle<Coord>,
    pub scale: Coord,
}

impl Interpolatable for Transform {
    fn add(self, other: Self) -> Self {
        Self {
            translation: self.translation + other.translation,
            rotation: self.rotation + other.rotation,
            scale: self.scale + other.scale,
        }
    }

    fn sub(self, other: Self) -> Self {
        Self {
            translation: self.translation - other.translation,
            rotation: self.rotation - other.rotation,
            scale: self.scale - other.scale,
        }
    }

    fn scale(self, factor: f32) -> Self {
        let factor = r32(factor);
        Self {
            translation: self.translation * factor,
            rotation: self.rotation * factor,
            scale: self.scale * factor,
        }
    }

    fn length_sqr(self) -> f32 {
        self.translation.length_sqr() + self.rotation.length_sqr() + self.scale.length_sqr()
    }
}

impl MoveFrame {
    pub fn scale(lerp_time: impl Float, scale: impl Float) -> Self {
        Self {
            lerp_time: lerp_time.as_r32(),
            interpolation: MoveInterpolation::default(),
            change_curve: None,
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
            initial: Transform::default(),
            key_frames: VecDeque::new(),
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

    pub fn get_frame(&self, id: WaypointId) -> Option<Transform> {
        match id {
            WaypointId::Initial => Some(self.initial),
            WaypointId::Frame(i) => self.key_frames.get(i).map(|frame| frame.transform),
        }
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

    /// Find the temporaly closest waypoint (in past or future).
    pub fn closest_waypoint(&self, time: Time) -> (WaypointId, Transform, Time) {
        self.timed_positions()
            .min_by_key(|(_, _, key_time)| (*key_time - time).abs())
            .expect("Light has no waypoints") // NOTE: Can unwrap because there is always at least one waypoint - initial
    }

    /// Get the transform at the given time.
    pub fn get(&self, mut time: Time) -> Transform {
        let mut from = self.initial;

        let lerp = |from: Transform, to, time, duration, interpolation: MoveInterpolation| {
            let t = if duration > Time::ZERO {
                time / duration
            } else {
                Time::ONE
            };
            let t = interpolation.apply(t);
            from.lerp(&to, t)
        };

        // Fade in
        if time <= self.fade_in {
            let interpolation = MoveInterpolation::Smoothstep; // TODO: customize?
            return lerp(
                Transform {
                    scale: Coord::ZERO,
                    ..from
                },
                from,
                time,
                self.fade_in,
                interpolation,
            );
        }
        time -= self.fade_in;

        // TODO: bake only once before starting the level, then cache
        let interpolation = self.bake();

        // Find the target frame
        for (i, frame) in self.frames_iter().enumerate() {
            if time <= frame.lerp_time {
                // Apply frame's move interpolation
                let time = if frame.lerp_time > Time::ZERO {
                    time / frame.lerp_time
                } else {
                    Time::ONE
                };
                let time = frame.interpolation.apply(time);
                return interpolation.get(i, time).unwrap_or(from);
            }
            time -= frame.lerp_time;
            from = frame.transform;
        }

        // Fade out
        if time > Time::ZERO && time <= self.fade_out {
            let target = Transform {
                scale: Coord::ZERO,
                ..from
            };
            let interpolation = MoveInterpolation::Smoothstep; // TODO: customize?
            return lerp(from, target, time, self.fade_out, interpolation);
        }

        from // Default
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
            .fold(Time::ZERO, Add::add)
    }

    pub fn change_fade_out(&mut self, target: Time) {
        self.fade_out = target.clamp(r32(0.25), r32(25.0));
    }

    pub fn change_fade_in(&mut self, target: Time) {
        self.fade_in = target.clamp(r32(0.25), r32(25.0));
    }

    /// Bakes the interpolation path based on the keypoints.
    pub fn bake(&self) -> Interpolation<Transform> {
        let points = std::iter::once((self.initial, None)).chain(
            self.frames_iter()
                .map(|frame| (frame.transform, frame.change_curve)),
        );

        let mk_segment = |curve, segment: &[_]| match curve {
            TrajectoryInterpolation::Linear => InterpolationSegment::linear(segment),
            TrajectoryInterpolation::Spline { tension } => {
                InterpolationSegment::spline(segment, tension.as_f32())
            }
            TrajectoryInterpolation::Bezier => InterpolationSegment::bezier(segment),
        };

        let mut segments = vec![];
        let mut current_curve = TrajectoryInterpolation::Linear;
        let mut current_segment = vec![]; // TODO: smallvec
        for (point, curve) in points {
            current_segment.push(point);
            if let Some(new_curve) = curve {
                if !current_segment.is_empty() {
                    segments.push(mk_segment(current_curve, &current_segment));
                }
                current_segment = vec![point];
                current_curve = new_curve;
            }
        }

        if !current_segment.is_empty() {
            segments.push(mk_segment(current_curve, &current_segment));
        }

        Interpolation { segments }
    }
}

fn smoothstep<T: Float>(t: T) -> T {
    T::from_f32(3.0) * t * t - T::from_f32(2.0) * t * t * t
}

fn ease_in<T: Float>(t: T) -> T {
    t * t * t
}

fn ease_out<T: Float>(t: T) -> T {
    let t = T::ONE - t;
    T::ONE - t * t * t
}
