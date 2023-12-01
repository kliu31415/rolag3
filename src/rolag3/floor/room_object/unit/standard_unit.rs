use crate::rolag3::floor::rofiz::{rofiz_object::{RofizObjectMovement, Transformation}, rofiz_state::{RofizState, RofizObjectRef}};

use super::Unit;

pub trait StandardUnit: Unit {

}

pub enum Budeb {
    MaxSpeed(BudebMaxSpeed),
}

pub struct BudebMaxSpeed {
    time_til_expiry: f64,
    multiplier: f64,
}

impl BudebMaxSpeed {
    pub fn new(time_til_expiry: f64, multiplier: f64) -> Self {
        Self { time_til_expiry, multiplier }
    }
}

pub struct StandardUnitCommon {
    ro_ref: RofizObjectRef,
    max_speed: f64,
    max_accel: Option<f64>,
    velocity_x: f64,
    velocity_y: f64,
    budebs: Vec<Budeb>,
}

impl StandardUnitCommon {
    const EPSILON: f64 = 1e-20;

    pub fn new(ro_ref: RofizObjectRef, max_speed: f64, accel: Option<f64>) -> Self {
        Self {
            ro_ref,
            max_speed,
            max_accel: accel,
            velocity_x: 0.0,
            velocity_y: 0.0,
            budebs: Vec::new(),
        }
    }

    pub fn get_velocity_x(&self) -> f64 {
        self.velocity_x
    }

    pub fn get_velocity_y(&self) -> f64 {
        self.velocity_y
    }


    pub fn get_ro_ref(&self) -> &RofizObjectRef {
        &self.ro_ref
    }

    pub fn decelerate_ro_xy(&mut self, tick_length: f64) {
        if self.max_accel.is_none() {
            self.velocity_x = 0.0;
            self.velocity_y = 0.0;
            return;
        }

        let velocity_norm = f64::hypot(self.velocity_x, self.velocity_y);
        if velocity_norm < Self::EPSILON {
            self.velocity_x = 0.0;
            self.velocity_y = 0.0;
            return;
        }

        let decel_x = self.max_accel.unwrap() * tick_length * self.velocity_x / velocity_norm;
        if f64::abs(decel_x) > f64::abs(self.velocity_x) {
            self.velocity_x = 0.0;
        } else {
            self.velocity_x -= decel_x;
        }

        let decel_y = self.max_accel.unwrap() * tick_length * self.velocity_y / velocity_norm;
        if f64::abs(decel_y) > f64::abs(self.velocity_y) {
            self.velocity_y = 0.0;
        } else {
            self.velocity_y -= decel_y;
        }
    }

    pub fn accelerate_ro_xy(&mut self, tick_length: f64, x: f64, y: f64) {
        if self.max_accel.is_none() {
            self.velocity_x = x;
            self.velocity_y = y;
            self.clamp_velocity();
            return;
        }

        let norm = f64::hypot(x, y);
        if norm < Self::EPSILON {
            return;
        }

        self.velocity_x += self.max_accel.unwrap() * tick_length * x / norm;
        self.velocity_y += self.max_accel.unwrap() * tick_length * y / norm;
        self.clamp_velocity();
    }

    pub fn reset_velocity(&mut self) {
        self.velocity_x = 0.0;
        self.velocity_y = 0.0;
    }

    pub fn process(&mut self, rofiz: &mut RofizState, tick_length: f64) {
        let mut max_speed_mult = 1.0;
        let mut min_speed_mult = 1.0;

        let mut expired_budeb_idx = Vec::new();
        for (i, budeb) in self.budebs.iter_mut().enumerate() {
            match budeb {
                Budeb::MaxSpeed(v) => {
                    v.time_til_expiry -= tick_length;
                    if v.time_til_expiry < 0.0 {
                        expired_budeb_idx.push(i);
                    }
                    max_speed_mult = f64::max(max_speed_mult, v.multiplier);
                    min_speed_mult = f64::min(min_speed_mult, v.multiplier);
                },
            }
        }
        for i in expired_budeb_idx.iter().rev() {
            self.budebs.remove(*i);
        }

        let dx = self.velocity_x * tick_length * max_speed_mult * min_speed_mult;
        let dy = self.velocity_y * tick_length * max_speed_mult * min_speed_mult;
        let dtheta = 0.0;

        let movement = RofizObjectMovement::Move(Transformation::new(dx, dy, dtheta));
        rofiz.move_object(&self.ro_ref, movement);
    }

    pub fn apply_budeb(&mut self, budeb: Budeb) {
        self.budebs.push(budeb);
    }

    fn clamp_velocity(&mut self) {
        let velocity_norm = f64::hypot(self.velocity_x, self.velocity_y);
        if velocity_norm < Self::EPSILON {
            return;
        }
        if velocity_norm > self.max_speed {
            let adjustment = self.max_speed / velocity_norm;
            self.velocity_x *= adjustment;
            self.velocity_y *= adjustment;
        }
    }
}