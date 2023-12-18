use crate::rolag3::floor::{rofiz::{rofiz_object::{RofizObjectMovement, Transformation}, rofiz_state::{RofizState, RofizObjectRef}}, draw::Color};

#[derive(Debug, Clone, Copy)]
pub enum Budeb {
    SpeedMult(BudebMaxSpeed),
    TimeSpeedMult(BudebTimeSpeedMult)
}

#[derive(Debug, Clone, Copy)]
pub struct BudebMaxSpeed {
    pub time_til_expiry: f64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct BudebTimeSpeedMult {
    pub time_til_expiry: f64,
    pub multiplier: f64,
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
    angular_power: f64,
    angular_traction: f64,
    velocity_cap: f64, // prevents accel tiles from making the player too fast
    min_effective_velocity: f64, // determines how much force is applied at rest and low velocities

    velocity_x: f64,
    velocity_y: f64,
    velocity_theta: f64,

    act1_started: bool,
    unit_tick_length: f64,
    translate: TranslateMove,
    rotate: RotateMove,
    external_forces: Vec<PolarForce>,

    prev_position: Option<Transformation>,
    prev_desired_movement: Option<Transformation>,

    max_hp: f64,
    hp: f64,
    last_damaged_time: f64,

    budebs: Vec<Budeb>,
}

pub struct PolarForce {
    pub r: f64,
    pub theta: f64,
}

#[derive(Debug)]
pub enum TranslateMove {
    Nop,
    Accelerate{ax: f64, ay: f64}, // ax and ay will be normalized to engine power and traction
    Decelerate,
    ResetVelocity,
}

#[derive(Debug)]
pub enum RotateMove {
    Nop,
    Accelerate{atheta: f64}, // atheta will be normalized to engine power and traction
    Decelerate,
    ResetVelocity,
}

impl StandardUnitCommon {
    const MASS: f64 = 1.0;
    const GRAVITY: f64 = 1.0;
    const EPSILON: f64 = 1e-20;

    pub fn new(ro_ref: RofizObjectRef, hp: f64, engine_power: f64, tire_friction: f64, angular_power: f64, angular_traction: f64, velocity_cap: f64, min_effective_velocity: f64) -> Self {
        Self {
            ro_ref,
            engine_power,
            tire_friction,
            angular_power,
            angular_traction,
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_theta: 0.0,
            velocity_cap,
            min_effective_velocity,

            act1_started: false,
            unit_tick_length: 0.0,
            translate: TranslateMove::Nop,
            rotate: RotateMove::Nop,
            external_forces: Vec::new(),

            prev_position: None,
            prev_desired_movement: None,

            max_hp: hp,
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

    fn decelerate_xy(&mut self, tick_length: f64, force: f64) {
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

    fn decelerate_theta(&mut self, tick_length: f64, force: f64) {
        if f64::abs(self.velocity_theta) < Self::EPSILON {
            self.velocity_theta = 0.0;
            return;
        }

        let decel = tick_length * force / Self::MASS;

        if f64::abs(decel) > f64::abs(self.velocity_theta) {
            self.velocity_theta = 0.0;
        } else {
            self.velocity_theta -= decel;
        }
    }

    pub fn start_act1(&mut self, room_tick_length: f64) {
        assert!(!self.act1_started, "cannot call standard_unit_common::start_act1() twice for standard_unit_common");
        self.act1_started = true;

        let mut max_time_speed_mult = 1.0;
        let mut min_time_speed_mult = 1.0;

        let mut expired_budeb_idx = Vec::new();
        for (i, budeb) in self.budebs.iter_mut().enumerate() {
            match budeb {
                Budeb::TimeSpeedMult(v) => {
                    v.time_til_expiry -= room_tick_length; // note we use room tick length here, not unit tick length
                    if v.time_til_expiry < 0.0 {
                        expired_budeb_idx.push(i);
                    } else {
                        max_time_speed_mult = f64::max(max_time_speed_mult, v.multiplier);
                        min_time_speed_mult = f64::min(min_time_speed_mult, v.multiplier);
                    }
                },
                _ => {},
            }
        }
        for i in expired_budeb_idx.iter().rev() {
            self.budebs.remove(*i);
        }

        let time_speed_mult = max_time_speed_mult * min_time_speed_mult;
        if !(0.5..2.0).contains(&time_speed_mult) {
            log::warn!("time_speed_mult({}) is outside of range [0.5, 2.0]", time_speed_mult);
        }
        self.unit_tick_length = room_tick_length * time_speed_mult;
    }

    pub fn set_translate_move(&mut self, translate: TranslateMove) {
        assert!(self.act1_started, "cannot call standard_unit_common::set_translate_move() before act1 starts");
        self.translate = translate;
    }

    pub fn set_rotate_move(&mut self, rotate: RotateMove) {
        assert!(self.act1_started, "cannot call standard_unit_common::set_rotate_move() before act1 starts");
        self.rotate = rotate;
    }

    pub fn add_external_forces(&mut self, mut f: Vec<PolarForce>) {
        assert!(self.act1_started, "cannot call standard_unit_common::add_external_forces() before act1 starts");
        self.external_forces.append(&mut f);
    }

    pub fn get_unit_tick_len(&self) -> f64 {
        assert!(self.act1_started, "cannot call standard_unit_common::get_unit_tick_len() before act1 starts");
        self.unit_tick_length
    }

    pub fn end_act1(&mut self, rofiz: &mut RofizState) {
        assert!(self.act1_started, "cannot call standard_unit_common::end_act1() before act1 has started");

        let tick_length = self.unit_tick_length;
        let min_velocity = self.min_effective_velocity;

        match self.translate {
            TranslateMove::Nop => {},
            TranslateMove::Accelerate { ax, ay, .. } => (||{
                let axay_norm = f64::hypot(ax, ay);
                if axay_norm < Self::EPSILON {
                    return;
                }
                let velocity_norm = f64::hypot(self.velocity_x, self.velocity_y);
                let f_engine = self.tire_friction * self.engine_power / f64::max(velocity_norm, min_velocity);
                let accel = tick_length * f_engine / Self::MASS;
                self.velocity_x += accel * ax / axay_norm;
                self.velocity_y += accel * ay / axay_norm;
            })(),
            TranslateMove::Decelerate => (||{
                let velocity_norm = f64::hypot(self.velocity_x, self.velocity_y);
                if velocity_norm < Self::EPSILON {
                    self.velocity_x = 0.0;
                    self.velocity_y = 0.0;
                    return;
                }
        
                let f_engine = self.tire_friction * self.engine_power / f64::max(velocity_norm, min_velocity);
                self.decelerate_xy(tick_length, f_engine);
            })(),
            TranslateMove::ResetVelocity => {
                self.velocity_x = 0.0;
                self.velocity_y = 0.0;
            }
        }

        match self.rotate {
            RotateMove::Nop => {}
            RotateMove::Accelerate { atheta } => (||{
                if f64::abs(atheta) < Self::EPSILON {
                    return;
                }
                let f_angular = self.angular_traction * self.angular_power / f64::max(self.velocity_theta, min_velocity);
                let accel = tick_length * f_angular / Self::MASS;
                self.velocity_theta += accel;
            })(),
            RotateMove::Decelerate => (||{
                if f64::abs(self.velocity_theta) < Self::EPSILON {
                    self.velocity_theta = 0.0;
                    return;
                }
        
                let f_angular = self.angular_traction * self.angular_power / f64::max(self.velocity_theta, min_velocity);
                self.decelerate_theta(tick_length, f_angular);
            })(),
            RotateMove::ResetVelocity => {
                self.velocity_theta = 0.0;
            }
        }

        if !self.external_forces.is_empty() {
            let fx: f64 = self.external_forces.iter().map(|x| x.r * f64::cos(x.theta)).sum();
            let fy: f64 = self.external_forces.iter().map(|x| x.r * f64::sin(x.theta)).sum();
            let ax = fx * tick_length / Self::MASS;
            let ay = fy * tick_length / Self::MASS;
            self.velocity_x += ax;
            self.velocity_y += ay;
        }

        let position = rofiz.get_movable_object_xform(&self.ro_ref);

        if let Some(prev_position) = self.prev_position {
            let prev_desired_movement = self.prev_desired_movement.unwrap();
            let actual_movement = position.sub(&prev_position);
            let accel_xform = actual_movement.sub(&prev_desired_movement);
            let accel_norm = f64::hypot(accel_xform.dx, accel_xform.dy);
            if accel_norm > 1e-10 {
                let prev_velocity = f64::hypot(prev_desired_movement.dx, prev_desired_movement.dy);
                self.velocity_x += 10.0 * prev_velocity * tick_length * accel_xform.dx / accel_norm;
                self.velocity_y += 10.0 * prev_velocity * tick_length * accel_xform.dy / accel_norm;
            }

            self.prev_position = None;
            self.prev_desired_movement = None;
        }

        // this simulates friction.
        // TODO: friction is calculated with slightly different velocities than the acceleration calculations. 
        // Friction is computed with the post-acceleration velocity. This should only make a small difference in
        // practice, but I'm just making a note in case there are bugs.
        self.decelerate_xy(tick_length, self.tire_friction * Self::MASS * Self::GRAVITY);

        // we have to cap the underlying velocity. We can't just compute a separate scaled velocity while leaving the
        // underlying the same. The reason is because if the player's velocity is 10000, and we compute a separate
        // scaled velocity capped to 150, the player will retain a velocity of 150 for a very long time.
        let vnorm = f64::hypot(self.velocity_x, self.velocity_y);
        if vnorm > self.velocity_cap {
            let scale = self.velocity_cap / vnorm;
            self.velocity_x *= scale;
            self.velocity_y *= scale;
        }

        let mut max_speed_mult = 1.0;
        let mut min_speed_mult = 1.0;

        let mut expired_budeb_idx = Vec::new();
        for (i, budeb) in self.budebs.iter_mut().enumerate() {
            match budeb {
                Budeb::SpeedMult(v) => {
                    v.time_til_expiry -= tick_length;
                    if v.time_til_expiry < 0.0 {
                        expired_budeb_idx.push(i);
                    } else {
                        max_speed_mult = f64::max(max_speed_mult, v.multiplier);
                        min_speed_mult = f64::min(min_speed_mult, v.multiplier);
                    }
                },
                Budeb::TimeSpeedMult(_) => {}, // handled in start_act1()
            }
        }
        for i in expired_budeb_idx.iter().rev() {
            self.budebs.remove(*i);
        }

        let dx = self.velocity_x * tick_length * max_speed_mult * min_speed_mult;
        let dy = self.velocity_y * tick_length * max_speed_mult * min_speed_mult;
        let dtheta = self.velocity_theta * tick_length * max_speed_mult * min_speed_mult;

        let mut movement_and_fallbacks = vec![Transformation::new(dx, dy, dtheta)];

        let dxy_r = f64::hypot(dx, dy);
        let dxy_theta = f64::atan2(dy, dx);
        for i in 1..3 {
            for j in [-1, 1] {
                let angle = (j * i) as f64 / 3.0 * std::f64::consts::PI / 2.0;
                let mag_adj = f64::cos(angle);
                let new_dx = mag_adj * dxy_r * f64::cos(dxy_theta + angle);
                let new_dy = mag_adj * dxy_r * f64::sin(dxy_theta + angle);
                movement_and_fallbacks.push(Transformation::new(new_dx, new_dy, f64::abs(i as f64) / 3.0 * dtheta));
            }
        }
        let movement = RofizObjectMovement::MoveWithFallbacks(movement_and_fallbacks.clone());
        self.prev_position = Some(position);
        self.prev_desired_movement = Some(Transformation::new(dx, dy, dtheta));

        self.act1_started = false;
        self.translate = TranslateMove::Nop;
        self.rotate = RotateMove::Nop;
        self.external_forces.clear();

        rofiz.move_object(&self.ro_ref, movement);
    }

    pub fn apply_budeb(&mut self, budeb: &Budeb) {
        self.budebs.push(*budeb);
    }

    pub fn take_damage(&mut self, room_time: f64, damage: f64) -> TakeDamageResponse {
        if damage < 0.0 {
            panic!("damage < 0. Expected positive damage.");
        }
        
        if damage == 0.0 {
            return TakeDamageResponse {
                dead: false,
                damage_taken: 0.0,
            };
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

    pub fn get_cur_hp(&self) -> f64 {
        self.hp
    }

    pub fn get_max_hp(&self) -> f64 {
        self.max_hp
    }
}

pub struct TakeDamageResponse {
    pub dead: bool,
    pub damage_taken: f64,
}

fn lerp_no_alpha(v: f32, a: Color, b: Color) -> Color {
    if !(0.0..=1.0).contains(&v) {
        panic!("lerp got v={}", v);
    }
    Color::new(
        a.r*(1.0-v) + b.r*v,
        a.g*(1.0-v) + b.g*v,
        a.b*(1.0-v) + b.b*v,
        a.a)
}