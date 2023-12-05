use crate::rolag3::floor::{rofiz::{rofiz_object::{RofizObjectMovement, Transformation}, rofiz_state::{RofizState, RofizObjectRef}}, draw::Color};

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
    engine_power: f64, // intuitively, equal to the max speed in tiles/s
    tire_friction: f64, // intuitively, proportional to how quickly the unit reaches its max speed
    velocity_x: f64,
    velocity_y: f64,

    hp: f64,
    last_damaged_time: f64,

    budebs: Vec<Budeb>,
}

impl StandardUnitCommon {
    const MASS: f64 = 1.0;
    const GRAVITY: f64 = 1.0;
    const EPSILON: f64 = 1e-20;

    pub fn new(ro_ref: RofizObjectRef, hp: f64, engine_power: f64, tire_friction: f64) -> Self {
        Self {
            ro_ref,
            engine_power,
            tire_friction,
            velocity_x: 0.0,
            velocity_y: 0.0,

            hp,
            last_damaged_time: -100.0,

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
        let velocity_norm = f64::hypot(self.velocity_x, self.velocity_y);
        if velocity_norm < Self::EPSILON {
            self.velocity_x = 0.0;
            self.velocity_y = 0.0;
            return;
        }

        let f_engine = self.tire_friction * self.engine_power / f64::max(velocity_norm, 0.1);
        self.decelerate(tick_length, f_engine);
    }

    fn decelerate(&mut self, tick_length: f64, force: f64) {
        let velocity_norm = f64::hypot(self.velocity_x, self.velocity_y);
        if velocity_norm < Self::EPSILON {
            self.velocity_x = 0.0;
            self.velocity_y = 0.0;
            return;
        }

        let decel = tick_length * force / Self::MASS;

        let decel_x = decel * self.velocity_x / velocity_norm;
        if f64::abs(decel_x) > f64::abs(self.velocity_x) {
            self.velocity_x = 0.0;
        } else {
            self.velocity_x -= decel_x;
        }

        let decel_y = decel * self.velocity_y / velocity_norm;
        if f64::abs(decel_y) > f64::abs(self.velocity_y) {
            self.velocity_y = 0.0;
        } else {
            self.velocity_y -= decel_y;
        }
    }

    pub fn accelerate_ro_xy(&mut self, tick_length: f64, x: f64, y: f64) {
        let input_norm = f64::hypot(x, y);
        if input_norm < Self::EPSILON {
            return;
        }
        let velocity_norm = f64::hypot(self.velocity_x, self.velocity_y);
        let f_engine = self.tire_friction * self.engine_power / f64::max(velocity_norm, 0.1);
        let accel = tick_length * f_engine / Self::MASS;
        self.velocity_x += accel * x / input_norm;
        self.velocity_y += accel * y / input_norm;
    }

    pub fn reset_velocity(&mut self) {
        self.velocity_x = 0.0;
        self.velocity_y = 0.0;
    }

    pub fn process(&mut self, rofiz: &mut RofizState, tick_length: f64) {
        // this simulates friction.
        // TODO: friction is calculated with slightly different velocities than the acceleration calculations. 
        // Friction is computed with the post-acceleration velocity. This should only make a small difference in
        // practice, but I'm just making a note in case there are bugs.
        self.decelerate(tick_length, self.tire_friction * Self::MASS * Self::GRAVITY);

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

        let mut move_fallbacks = vec![Transformation::new(dx, dy, dtheta)];

        let dxy_r = f64::hypot(dx, dy);
        let dxy_theta = f64::atan2(dy, dx);
        for i in 1..3 {
            for j in [-1, 1] {
                let angle = (j * i) as f64 / 3.0 * std::f64::consts::PI / 2.0;
                let mag_adj = f64::cos(angle);
                let new_dx = mag_adj * dxy_r * f64::cos(dxy_theta + angle);
                let new_dy = mag_adj * dxy_r * f64::sin(dxy_theta + angle);
                move_fallbacks.push(Transformation::new(new_dx, new_dy, dtheta));
            }
        }
        let movement = RofizObjectMovement::_MoveWithFallbacks(move_fallbacks);
        rofiz.move_object(&self.ro_ref, movement);
    }

    pub fn apply_budeb(&mut self, budeb: Budeb) {
        self.budebs.push(budeb);
    }

    pub fn take_damage(&mut self, room_time: f64, damage: f64) -> TakeDamageResponse {
        if damage < 0.0 {
            panic!("damage < 0. Expected positive damage.");
        }
        let damage_taken: f64;
        if self.hp < damage {
            damage_taken = self.hp;
            self.hp = 0.0
        } else {
            damage_taken = damage;
            self.hp -= damage;
        }
        self.last_damaged_time = room_time;
        TakeDamageResponse { 
            dead: self.hp <= 0.0,
            damage_taken,
         }
    }

    pub fn get_draw_color(&self, room_time: f64, original_color: Color) -> Color {
        lerp_no_alpha(((1.0 - 4.0 * f64::min(0.25, room_time - self.last_damaged_time)) / 1.5) as f32, 
            original_color,
            Color::new(1.0, 1.0, 1.0, 0.0))
    }
}

pub struct TakeDamageResponse {
    pub dead: bool,
    pub damage_taken: f64,
}

fn lerp_no_alpha(v: f32, a: Color, b: Color) -> Color {
    if v < 0.0 || v > 1.0 {
        panic!("lerp got v={}", v);
    }
    Color::new(
        a.r*(1.0-v) + b.r*v,
        a.g*(1.0-v) + b.g*v,
        a.b*(1.0-v) + b.b*v,
        a.a)
}