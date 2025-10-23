use itertools::Itertools;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Movement {
    /// The spawn waypoint that is the very first transformation of the light when it appears on the screen.
    /// Typically it is used to setup the *fade in* effect from scale zero.
    pub initial: WaypointInitial,
    pub waypoints: VecDeque<Waypoint>,
    /// The final waypoint that is the very last transformation of the light when it is visible on the screen.
    /// Typically it is used to setup the *fade out* effect to scale zero.
    pub last: Transform,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct WaypointInitial {
    /// Duration of the interpolation from this frame to the next.
    pub lerp_time: Time,
    /// Interpolation to use when moving away from this frame to the next.
    #[serde(default)]
    pub interpolation: MoveInterpolation,
    /// The initial curve going from this frame to the next.
    pub curve: TrajectoryInterpolation,
    #[serde(default)]
    pub transform: Transform,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Waypoint {
    /// Duration of the interpolation from this frame to the next.
    pub lerp_time: Time,
    /// Interpolation to use when moving away from this frame to the next.
    #[serde(default)]
    pub interpolation: MoveInterpolation,
    /// Whether to start a new curve going from this frame to the next.
    /// If set to `None`, the curve will continue as the previous type.
    pub change_curve: Option<TrajectoryInterpolation>,
    pub transform: Transform,
}

/// Controls the speed of the light when moving between keyframes.
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WaypointId {
    Initial,
    Frame(usize),
    Last,
}

impl WaypointId {
    pub fn prev(self, waypoints: usize) -> Option<Self> {
        match self {
            Self::Initial => None,
            Self::Frame(0) => Some(Self::Initial),
            Self::Frame(i) => Some(Self::Frame(i - 1)),
            Self::Last => Some(if waypoints > 0 {
                Self::Frame(waypoints - 1)
            } else {
                Self::Initial
            }),
        }
    }

    pub fn next(self, waypoints: usize) -> Option<Self> {
        match self {
            Self::Initial => Some(if waypoints > 0 {
                Self::Frame(0)
            } else {
                Self::Last
            }),
            Self::Frame(i) => Some(if i + 1 < waypoints {
                Self::Frame(i + 1)
            } else {
                Self::Last
            }),
            Self::Last => None,
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

impl WaypointInitial {
    pub fn new(lerp_time: Time, transform: Transform) -> Self {
        Self {
            lerp_time,
            interpolation: MoveInterpolation::default(),
            curve: TrajectoryInterpolation::default(),
            transform,
        }
    }
}

impl Waypoint {
    pub fn new(lerp_time: Time, transform: Transform) -> Self {
        Self {
            lerp_time,
            interpolation: MoveInterpolation::default(),
            change_curve: None,
            transform,
        }
    }

    pub fn scale(lerp_time: Time, scale: impl Float) -> Self {
        Self {
            lerp_time,
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
        Self::new(TIME_IN_FLOAT_TIME / 2, Transform::default())
    }
}

impl Movement {
    pub fn new(fade_time: Time, initial: Transform) -> Self {
        Self {
            // Fade in
            initial: WaypointInitial {
                lerp_time: fade_time,
                interpolation: MoveInterpolation::default(),
                curve: TrajectoryInterpolation::default(),
                transform: Transform {
                    scale: R32::ZERO,
                    ..initial
                },
            },
            waypoints: vec![Waypoint::new(fade_time, initial)].into(),
            // Fade out
            last: Transform {
                scale: R32::ZERO,
                ..initial
            },
        }
    }

    pub fn get_frame(&self, id: WaypointId) -> Option<Transform> {
        match id {
            WaypointId::Initial => Some(self.initial.transform),
            WaypointId::Frame(i) => self.waypoints.get(i).map(|frame| frame.transform),
            WaypointId::Last => Some(self.last),
        }
    }

    pub fn get_frame_mut(&mut self, id: WaypointId) -> Option<&mut Transform> {
        match id {
            WaypointId::Initial => Some(&mut self.initial.transform),
            WaypointId::Frame(i) => self.waypoints.get_mut(i).map(|frame| &mut frame.transform),
            WaypointId::Last => Some(&mut self.last),
        }
    }

    pub fn get_interpolation(
        &self,
        id: WaypointId,
    ) -> Option<(MoveInterpolation, Option<TrajectoryInterpolation>)> {
        match id {
            WaypointId::Initial => Some((self.initial.interpolation, Some(self.initial.curve))),
            WaypointId::Frame(i) => self
                .waypoints
                .get(i)
                .map(|frame| (frame.interpolation, frame.change_curve)),
            WaypointId::Last => None,
        }
    }

    /// Iterate over all transforms (including initial).
    pub fn transforms_iter(&self) -> impl Iterator<Item = Transform> {
        itertools::chain![
            [self.initial.transform],
            self.waypoints.iter().map(|waypoint| waypoint.transform),
            [self.last]
        ]
    }

    /// Iterate over all transforms together with their start times.
    pub fn timed_transforms(&self) -> impl Iterator<Item = (WaypointId, Transform, Time)> + '_ {
        itertools::chain![
            [(
                WaypointId::Initial,
                self.initial.transform,
                self.initial.lerp_time
            )],
            self.waypoints.iter().enumerate().map(|(i, &waypoint)| (
                WaypointId::Frame(i),
                waypoint.transform,
                waypoint.lerp_time
            )),
            [(WaypointId::Last, self.last, 0)],
        ]
        .scan(Time::ZERO, |time, (i, transform, lerp_time)| {
            let frame_time = *time;
            *time += lerp_time;
            Some((i, transform, frame_time))
        })
    }

    /// Get the start time of the waypoint.
    pub fn get_time(&self, id: WaypointId) -> Option<Time> {
        let i = match id {
            WaypointId::Initial => 0,
            WaypointId::Frame(i) => i + 1,
            WaypointId::Last => self.waypoints.len() + 1,
        };
        self.timed_transforms().nth(i).map(|(_, _, time)| time)
    }

    /// Find the temporaly closest waypoint (in past or future).
    pub fn closest_waypoint(&self, time: Time) -> (WaypointId, Transform, Time) {
        self.timed_transforms()
            .min_by_key(|(_, _, key_time)| (*key_time - time).abs())
            .expect("Light has no waypoints") // NOTE: Can unwrap because there is always at least two waypoints - initial and last
    }

    /// Get the transform at the given time.
    pub fn get(&self, mut time: Time) -> Transform {
        // TODO: bake only once before starting the level, then cache
        let curve_interpolation = self.bake();

        // Find the target frame
        let mut from = self.initial.transform;
        let interpolation = self.initial.interpolation;
        for (i, (frame, lerp_time)) in itertools::chain![
            [(self.initial.transform, self.initial.lerp_time)],
            self.waypoints
                .iter()
                .map(|waypoint| (waypoint.transform, waypoint.lerp_time))
        ]
        .enumerate()
        {
            if time <= lerp_time {
                // Apply frame's move interpolation
                let time = if lerp_time > Time::ZERO {
                    FloatTime::new(time as f32 / lerp_time as f32)
                } else {
                    FloatTime::ONE
                };
                let time = interpolation.apply(time);
                return curve_interpolation.get(i, time).unwrap_or(from);
            }
            time -= lerp_time;
            from = frame;
        }

        // Past all waypoints just return the last transform
        self.last
    }

    /// Returns the total duration of the movement.
    pub fn duration(&self) -> Time {
        // NOTE: skip last waypoint because its lerp_time is redundant
        // as there are no waypoints after it to lerp to
        self.waypoints
            .iter()
            .rev()
            .skip(1)
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

    pub fn get_fade_out(&self) -> Time {
        self.waypoints
            .back()
            .map_or(self.initial.lerp_time, |waypoint| waypoint.lerp_time)
    }

    pub fn get_fade_in(&self) -> Time {
        self.waypoints
            .front()
            .map_or(self.initial.lerp_time, |waypoint| waypoint.lerp_time)
    }

    pub fn change_fade_out(&mut self, target: Time) {
        let target = target.clamp(0, TIME_IN_FLOAT_TIME * 50);
        let value = self
            .waypoints
            .back_mut()
            .map_or(&mut self.initial.lerp_time, |waypoint| {
                &mut waypoint.lerp_time
            });
        *value = target;
    }

    pub fn change_fade_in(&mut self, target: Time) {
        let target = target.clamp(0, TIME_IN_FLOAT_TIME * 50);
        let value = self
            .waypoints
            .front_mut()
            .map_or(&mut self.initial.lerp_time, |waypoint| {
                &mut waypoint.lerp_time
            });
        *value = target;
    }

    /// Bakes the interpolation path based on the keypoints.
    pub fn bake(&self) -> Interpolation<Transform> {
        bake_movement(
            self.initial.transform,
            self.initial.curve,
            self.waypoints
                .iter()
                .map(|frame| (frame.transform, frame.change_curve)),
        )
    }

    pub fn modify_transforms(&mut self, mut f: impl FnMut(&mut Transform)) {
        f(&mut self.initial.transform);
        for frame in &mut self.waypoints {
            f(&mut frame.transform);
        }
        f(&mut self.last);
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
