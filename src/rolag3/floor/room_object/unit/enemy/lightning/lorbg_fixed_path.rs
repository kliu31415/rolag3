use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::rolag3::floor::{draw::{Color, DrawContext}, room_object::{room_object_def::{RoomObject, NewRoomObjectContext, RoomObjectMetadata, Act1Response, Act1Context, HandleCollisionContext, HandleCollisionResponse, RoomObjectType}, damage::DamageColor}};

use super::lightning_orb_group::{LightningOrbGroup, new_lightning_orb_group};

const ORB_BORDER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);
const ORB_INNER_COLOR: Color = Color::new(0.8, 0.8, 0.8, 1.0);
const ORB_BORDER_RADIUS: f32 = 0.4;
const ORB_INNER_RADIUS: f32 = 0.3;

const LIGHTNING_THICKNESS: f32 = 0.3;
const LIGHTNING_CBB_PER_SEC_MEAN: f64 = 12.0;
const LIGHTNING_CBB_PER_SEC_SD: f64 = 3.0;
const LIGHTNING_CBB_PER_SEC_MIN: f64 = 2.0;

pub struct LorbgFixedPath {
    md: RoomObjectMetadata,
    lo_group: Weak<RefCell<LightningOrbGroup>>,
    age: f64,
    orb_age_offsets: Box<[f64]>,
    path_segments: Box<[LogwcPathSegment]>,
    total_path_time_to_traverse: f64,
    orb_speeds: Box<[f64]>,
    cached_orb_centers: Box<[(f64, f64)]>,
}

pub struct LogwcPathSegment {
    time_to_traverse: f64,
    path_fn: Box<dyn Fn(f64) -> (f64, f64)>,
}

pub fn make_logwc_path_polygon(speed: f64, vertexes: &[(f64, f64)]) -> Box<[LogwcPathSegment]> {
    vertexes.iter().zip(vertexes[1..].iter().chain(vertexes[..1].iter()))
        .map(|(v1, v2)| (*v1, *v2))
        .map(move |(v1, v2)| {
            let dist = f64::hypot(v2.0 - v1.0, v2.1 - v1.1);
            // if dist is 0, we get a divide by 0. If it's close enough to 0, we may get weird behavior from floating
            // point imprecisions. If dist is <0.1, it's probably unintended anyway, so panic.
            assert!(dist > 0.1, "dist of {} is excessively small. This is probably unintended.", dist);
            let time_to_traverse = dist / speed;
            let path_fn = Box::new(move |t| {
                let x = v1.0 + t * speed * (v2.0 - v1.0) / dist;
                let y = v1.1 + t * speed * (v2.1 - v1.1) / dist;
                (x, y)
            });
            LogwcPathSegment {
                time_to_traverse,
                path_fn,
            }
        }).collect()
}

impl RoomObject for LorbgFixedPath {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response {
        let mut response = Act1Response::new();
        self.age += ctx.get_tick_length();
        let lo_group_rc = self.lo_group.upgrade().unwrap();
        for (i, (orb_speed, orb_age_offset)) in self.orb_speeds.iter().zip(self.orb_age_offsets.iter()).enumerate() {
            let effective_age = self.age * orb_speed + orb_age_offset;
            self.cached_orb_centers[i] = get_orb_position(effective_age, self.total_path_time_to_traverse, &self.path_segments);
        };
        lo_group_rc.borrow_mut().slave_act1(ctx, &mut response, 0.5, &self.cached_orb_centers);
        response
    }

    fn draw(&mut self, ctx: &mut DrawContext) {
        let mut dops = Vec::new();
        let lo_group_rc = self.lo_group.upgrade().unwrap();
        lo_group_rc.borrow_mut().slave_draw(&mut dops, ctx, ORB_INNER_COLOR, ORB_BORDER_COLOR, ORB_INNER_RADIUS, ORB_BORDER_RADIUS);
        ctx.add_draw_op(DrawContext::Z_UNIT, ctx.dop_group(dops.into()));
    }

    fn handle_collision(&mut self, _ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        // nop
        HandleCollisionResponse::new()
    }
}

fn get_orb_position(mut effective_age: f64, total_path_time: f64, path_segments: &[LogwcPathSegment]) -> (f64, f64) {
    effective_age %= total_path_time;
    if effective_age < 0.0 {
        effective_age += total_path_time;
    }
    for (j, path) in path_segments.iter().enumerate() {
        if j+1 == path_segments.len() || effective_age <= path.time_to_traverse {
            // if j+1 == len(), then effective_age can be slightly greater than path.time_to_traverse
            // due to accumulated floating point errors. Clamp effective_age to fix this.
            effective_age = f64::min(effective_age, path.time_to_traverse);
            return (path.path_fn)(effective_age);
        }
        effective_age -= path.time_to_traverse;
    }
    panic!("unexpectedly reached end of function");
}

pub fn new_lorbg_fixed_path(
    ctx: &mut NewRoomObjectContext,
    path_segments: Box<[LogwcPathSegment]>,
    orb_age_offsets: &[f64],
    orb_speeds: &[f64],
    lightning_colors: &[Option<DamageColor>],
) -> Box<[Rc<RefCell<dyn RoomObject>>]> {
    assert!(orb_age_offsets.len() == lightning_colors.len());
    assert!(orb_age_offsets.len() == orb_speeds.len());
    let num_orbs = orb_age_offsets.len();
    let total_path_time_to_traverse = path_segments.iter().map(|x| x.time_to_traverse).sum();
    let orb_xy = orb_age_offsets.iter()
        .map(|x| get_orb_position(*x, total_path_time_to_traverse, &path_segments))
        .collect::<Box<_>>();
    let (lo_group, lo_group_others) = new_lightning_orb_group(
        ctx, 
        num_orbs, 
        &lightning_colors, 
        &orb_xy, 
        ORB_BORDER_RADIUS, 
        LIGHTNING_CBB_PER_SEC_MEAN, 
        LIGHTNING_CBB_PER_SEC_SD, 
        LIGHTNING_CBB_PER_SEC_MIN, 
        LIGHTNING_THICKNESS,
    );
    let logfp = LorbgFixedPath {
        md: RoomObjectMetadata::new(ctx, RoomObjectType::Other),
        lo_group: Rc::downgrade(&lo_group),
        age: 0.0,
        orb_age_offsets: orb_age_offsets.into(),
        path_segments,
        total_path_time_to_traverse,
        orb_speeds: orb_speeds.into(),
        cached_orb_centers: vec![(0.0, 0.0); num_orbs].into(),
    };
    [Rc::new(RefCell::new(logfp)) as _].into_iter()
        .chain([lo_group as _]).into_iter()
        .chain(lo_group_others.into_vec().into_iter())
        .collect()
}