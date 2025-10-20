use super::*;

use ctl_core::prelude::Interpolatable;

#[derive(Debug, Clone)]
pub struct SecondOrderState<T> {
    pub target: T,
    pub current: T,
    dynamics: SecondOrderDynamics<T>,
}

impl<T: Interpolatable + Copy> SecondOrderState<T> {
    pub fn new(dynamics: SecondOrderDynamics<T>) -> Self {
        Self {
            target: dynamics.xp,
            current: dynamics.xp,
            dynamics,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.current = self.dynamics.update(delta_time, self.target);
    }

    pub fn snap_to(&mut self, value: T, delta_time: f32) {
        self.target = value;
        self.current = value;
        self.dynamics.y = value;
        self.dynamics.update(delta_time, value);
    }

    pub fn velocity(&self) -> T {
        self.dynamics.yd
    }
}

/// Second order dynamic system as described by
/// `y + k1 * dy/dt + k2 * dy^2/d^2t = x + k3 * dx/dt`.
///
/// Inspired by <https://youtu.be/KPoeNZZ6H4s>
#[derive(Debug, Clone)]
pub struct SecondOrderDynamics<T> {
    xp: T,
    y: T,
    yd: T,
    k1: f32,
    k2: f32,
    k3: f32,
}

impl<T: Interpolatable + Copy> SecondOrderDynamics<T> {
    pub fn new(frequency: f32, damping: f32, response: f32, value: T) -> Self {
        assert!(frequency > 0.0, "frequency has to be positive");

        let k1 = damping / (f32::PI * frequency);
        let k2 = (2.0 * f32::PI * frequency).sqr().recip();
        let k3 = response * damping / (2.0 * f32::PI * frequency);
        Self {
            xp: value,
            y: value,
            yd: value.sub(value), // ZERO
            k1,
            k2,
            k3,
        }
    }

    pub fn update(&mut self, delta_time: f32, value: T) -> T {
        let velocity = (value.sub(self.xp)).scale(delta_time.recip());
        self.update_with_velocity(delta_time, value, velocity)
    }

    pub fn update_with_velocity(&mut self, delta_time: f32, target: T, velocity: T) -> T {
        let k2_stable = 1.1 * (delta_time.sqr() / 4.0 + delta_time * self.k1 / 2.0);
        let k2_stable = k2_stable.max(self.k2); // Clamp to guarantee numerical stability

        // Intergrate position by velocity
        self.y = self.y.add(self.yd.scale(delta_time));
        // Integrate velocity by acceleration
        self.yd = self.yd.add(
            target
                .add(velocity.scale(self.k3))
                .add(self.y.scale(-1.0))
                .add(self.yd.scale(-self.k1))
                .scale(delta_time / k2_stable),
        );

        self.xp = target;
        self.y
    }
}
