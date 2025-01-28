use itertools::Itertools;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Movement {
    /// Time (in milliseconds) to spend fading into the initial position.
    pub fade_in: Time,
    /// Time (in milliseconds) to spend fading out of the last keyframe.
    pub fade_out: Time,
    pub initial: Transform,
    #[serde(default)]
    pub interpolation: MoveInterpolation,
    #[serde(default)]
    pub curve: TrajectoryInterpolation,
    pub key_frames: VecDeque<MoveFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoveFrame {
    /// How long (in beats) should the interpolation from the last frame to this frame last.
    pub lerp_time: Time,
    /// Interpolation to use when moving away from this frame to the next.
    #[serde(default)]
    pub interpolation: MoveInterpolation,
    /// Whether to start a new curve going towards from this frame further.
    /// If set to `None`, the curve will continue as the previous type.
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
    pub fn apply(&self, t: FloatTime) -> FloatTime {
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
            lerp_time: seconds_to_time(FloatTime::new(lerp_time.as_f32())),
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

    pub fn lerp(&self, target: &Self, t: FloatTime) -> Self {
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
            fade_in: TIME_IN_FLOAT_TIME,
            fade_out: TIME_IN_FLOAT_TIME,
            initial: Transform::default(),
            interpolation: MoveInterpolation::default(),
            curve: TrajectoryInterpolation::default(),
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

    pub fn get_interpolation(
        &self,
        id: WaypointId,
    ) -> Option<(MoveInterpolation, Option<TrajectoryInterpolation>)> {
        match id {
            WaypointId::Initial => Some((self.interpolation, Some(self.curve))),
            WaypointId::Frame(i) => self
                .key_frames
                .get(i)
                .map(|frame| (frame.interpolation, frame.change_curve)),
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
                FloatTime::new(time as f32 / duration as f32)
            } else {
                FloatTime::ONE
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
        let mut move_interp = self.interpolation;
        for (i, frame) in self.frames_iter().enumerate() {
            if time <= frame.lerp_time {
                // Apply frame's move interpolation
                let time = if frame.lerp_time > Time::ZERO {
                    FloatTime::new(time as f32 / frame.lerp_time as f32)
                } else {
                    FloatTime::ONE
                };
                let time = move_interp.apply(time);
                return interpolation.get(i, time).unwrap_or(from);
            }
            time -= frame.lerp_time;
            from = frame.transform;
            move_interp = frame.interpolation;
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

    /// Returns the total distance that a light will travel following this movement.
    pub fn total_distance(&self) -> Coord {
        // TODO: cached bake
        let interpolation = self.bake();
        interpolation
            .get_path(5)
            .tuple_windows()
            .map(|(a, b)| (b.translation - a.translation).len())
            .fold(Coord::ZERO, |a, b| a + b)
    }

    pub fn change_fade_out(&mut self, target: Time) {
        self.fade_out = target.clamp(TIME_IN_FLOAT_TIME / 10, TIME_IN_FLOAT_TIME * 50);
    }

    pub fn change_fade_in(&mut self, target: Time) {
        self.fade_in = target.clamp(TIME_IN_FLOAT_TIME / 10, TIME_IN_FLOAT_TIME * 50);
    }

    /// Bakes the interpolation path based on the keypoints.
    pub fn bake(&self) -> Interpolation<Transform> {
        bake_movement(
            self.initial,
            self.curve,
            self.frames_iter()
                .map(|frame| (frame.transform, frame.change_curve)),
        )
    }

    fn modify_transforms(&mut self, mut f: impl FnMut(&mut Transform)) {
        f(&mut self.initial);
        for frame in &mut self.key_frames {
            f(&mut frame.transform);
        }
    }

    pub fn rotate_around(&mut self, anchor: vec2<Coord>, delta: Angle<Coord>) {
        self.modify_transforms(|transform: &mut Transform| {
            transform.translation = anchor + (transform.translation - anchor).rotate(delta);
            transform.rotation += delta;
        })
    }

    pub fn flip_horizontal(&mut self, anchor: vec2<Coord>) {
        self.modify_transforms(|transform: &mut Transform| {
            transform.translation =
                anchor + (transform.translation - anchor) * vec2(-Coord::ONE, Coord::ONE);
            transform.rotation = Angle::from_degrees(r32(180.0)) - transform.rotation;
        })
    }

    pub fn flip_vertical(&mut self, anchor: vec2<Coord>) {
        self.modify_transforms(|transform: &mut Transform| {
            transform.translation =
                anchor + (transform.translation - anchor) * vec2(Coord::ONE, -Coord::ONE);
            transform.rotation = -transform.rotation;
        })
    }
}

pub fn bake_movement<T: 'static + Interpolatable>(
    initial: T,
    initial_curve: TrajectoryInterpolation,
    keyframes: impl IntoIterator<Item = (T, Option<TrajectoryInterpolation>)>,
) -> Interpolation<T> {
    let points = std::iter::once((initial, None)).chain(keyframes);

    let mk_segment = |curve, segment: &[T]| match curve {
        TrajectoryInterpolation::Linear => InterpolationSegment::linear(segment),
        TrajectoryInterpolation::Spline { tension } => {
            InterpolationSegment::spline(segment, tension.as_f32())
        }
        TrajectoryInterpolation::Bezier => InterpolationSegment::bezier(segment),
    };

    let mut segments = vec![];
    let mut current_curve = initial_curve;
    let mut current_segment = vec![]; // TODO: smallvec
    for (point, curve) in points {
        if let Some(new_curve) = curve {
            if !current_segment.is_empty() {
                current_segment.push(point.clone());
                segments.push(mk_segment(current_curve, &current_segment));
            }
            current_segment = vec![point];
            current_curve = new_curve;
        } else {
            current_segment.push(point);
        }
    }

    if !current_segment.is_empty() {
        segments.push(mk_segment(current_curve, &current_segment));
    }

    Interpolation { segments }
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
