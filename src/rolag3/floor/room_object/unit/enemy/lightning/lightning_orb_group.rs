use std::{rc::{Weak, Rc}, cell::RefCell};

use crate::{rolag3::floor::{room_object::{room_object_def::{RoomObject, HandleCollisionResponse, HandleCollisionContext, Act1Context, Act1Response, RoomObjectMetadata, NewRoomObjectContext, RoomObjectType}, unit::standard_unit1::StandardUnit1, damage::DamageColor}, draw::{DrawContext, Color}}, gfx::renderer::DrawOp, geometry::shape::Point};

use super::{orb::{new_orb, Orb}, lightning::new_lightning, chunked_brownian_bridge::ChunkedBrownianBridge};

pub struct LightningOrbGroup {
    md: RoomObjectMetadata,
    orbs: Vec<Weak<RefCell<StandardUnit1>>>,
    lightnings: Vec<Vec<Option<Weak<RefCell<StandardUnit1>>>>>,
    lightning_thickness: f32,

    orbs_dop_cache: Vec<DrawOp>,
    orb_centers_cache: Box<[Point]>,
    cbb_quad_cache: Vec<[Point; 4]>,
}

impl RoomObject for LightningOrbGroup {
    fn get_metadata(&self) -> &RoomObjectMetadata {
        &self.md
    }

    fn act1(&mut self, _ctx: &mut Act1Context) -> Act1Response {
        Act1Response::new()
    }

    fn draw(&mut self, _ctx: &mut DrawContext) {
        // nop
    }

    fn handle_collision(&mut self, _ctx: &mut HandleCollisionContext) -> HandleCollisionResponse {
        HandleCollisionResponse::new()
    }
}

struct Empty {}

impl LightningOrbGroup {
    pub fn slave_act1(
        &mut self, 
        ctx: &mut Act1Context, 
        response: &mut Act1Response, 
        y_sd: f64, 
        orb_centers: &[(f64, f64)]
    ) {
        for (i, orb_weak) in self.orbs.iter().enumerate() {
            let orb_rc = orb_weak.upgrade().unwrap();
            let input = orb_centers[i];
            let mut output = Empty {};
            orb_rc.borrow_mut().custom_act1_fn(ctx, response, &input, &mut output);
        }
    
        for i in 0..self.lightnings.len() {
            for j in 0..i {
                let Some(ref lightning_weak) = self.lightnings[i][j] else {continue;};
                let lightning_rc = lightning_weak.upgrade().unwrap();
                let mut lightning = lightning_rc.borrow_mut();
    
                let orb1_rc = self.orbs[i].upgrade().unwrap();
                let orb1_ref = orb1_rc.borrow();
                let orb1 = orb1_ref.get_us_data().downcast_ref::<Orb>().unwrap();
                let orb1_pos = ctx.get_rofiz().get_movable_object_xform(&orb1.ro_ref);
    
                let orb2_rc = self.orbs[j].upgrade().unwrap();
                let orb2_ref = orb2_rc.borrow();
                let orb2 = orb2_ref.get_us_data().downcast_ref::<Orb>().unwrap();
                let orb2_pos = ctx.get_rofiz().get_movable_object_xform(&orb2.ro_ref);
    
                let mut output = Empty {};
                lightning.custom_act1_fn(ctx, response, &(orb1_pos, orb2_pos, y_sd), &mut output);
            }
        }
    }

    pub fn slave_draw(
        &mut self, 
        draw_ops: &mut Vec<DrawOp>, 
        ctx: &DrawContext,
        orb_inner_color: Color,
        orb_border_color: Color,
        orb_inner_radius: f32,
        orb_border_radius: f32,
    ) {
        for (i, orb_weak) in self.orbs.iter().enumerate() {
            let orb_rc = orb_weak.upgrade().unwrap();
            let orb_ref = orb_rc.as_ref().borrow();
            let orb = orb_ref.get_us_data().downcast_ref::<Orb>().unwrap();
            let orb_xform = ctx.get_rofiz().get_movable_object_xform(&orb.ro_ref);
    
            self.orb_centers_cache[i] = Point::new(orb_xform.dx as f32, orb_xform.dy as f32);
            let center = Point::new(orb_xform.dx as f32, orb_xform.dy as f32);
            let cc_dop = ctx.do_concentric_circle(orb_inner_color, orb_border_color, center, orb_inner_radius, orb_border_radius);
            self.orbs_dop_cache.push(cc_dop);
        }
    
        let num_orbs = self.orb_centers_cache.len();
        for i in 0..num_orbs {
            for j in 0..i {
                let Some(ref lightning_weak) = self.lightnings[i][j] else {continue;};
                let lightning_rc = lightning_weak.upgrade().unwrap();
                let mut lightning = lightning_rc.as_ref().borrow_mut();
                let mut output: (Option<ChunkedBrownianBridge>, Option<Color>) = (None, None);
                let input = Empty {};
                lightning.custom_fn(0, &input, &mut output);
                output.0.unwrap().to_quads(&mut self.cbb_quad_cache, self.lightning_thickness, self.orb_centers_cache[i], self.orb_centers_cache[j]);
                for q in self.cbb_quad_cache.drain(..) {
                    draw_ops.push(ctx.do_quad_fan(output.1.unwrap(), q));
                }
            }
        }
        draw_ops.append(&mut self.orbs_dop_cache);
    }
}

pub fn new_lightning_orb_group(
    ctx: &mut NewRoomObjectContext,
    num_orbs: usize,
    damage_colors: &[Option<DamageColor>],
    orb_xy: &[(f64, f64)] /* the initial position. Only matters for the first tick, so mostly a nop.*/,
    orb_radius: f32,
    lightning_cbb_per_s_mean: f64,
    lightning_cbb_per_s_sd: f64,
    lightning_cbb_per_s_min: f64,
    lightning_thickness: f32,
) -> (Rc<RefCell<LightningOrbGroup>>, Box<[Rc<RefCell<dyn RoomObject>>]>) {
    assert!(num_orbs > 1);
    assert_eq!(orb_xy.len(), num_orbs);
    assert_eq!(damage_colors.len(), (num_orbs - 1) * num_orbs / 2);

    let orbs = (0..num_orbs).map(|i| {
        let orb = new_orb(ctx, orb_xy[i].0, orb_xy[i].1, orb_radius);
        Rc::new(RefCell::new(orb))
    }).collect::<Vec<_>>();
    let orbs_weak = orbs.iter().map(|x| {Rc::downgrade(&x)}).collect::<Vec<_>>();

    let mut color_idxs = damage_colors.iter();
    let lightnings = (0..num_orbs).map(|i| {
        (0..i).map(|_j| {
            let damage_color_opt = *color_idxs.next().unwrap();
            let Some(damage_color) = damage_color_opt else {return None;};
            let draw_color = match damage_color {
                DamageColor::Red => Color::new(5.0, 0.1, 0.1, 1.0),
                DamageColor::Green => Color::new(0.1, 1.6, 0.1, 1.0),
                DamageColor::Blue => Color::new(0.1, 0.1, 16.0, 1.0),
                DamageColor::NotSet => panic!("lightning damage color of {:?} isn't supported", damage_color),
                DamageColor::Silver => panic!("lightning damage color of {:?} isn't supported", damage_color),
            };
            Some(Rc::new(RefCell::new(new_lightning(
                ctx, 
                damage_color,
                draw_color,
                lightning_cbb_per_s_mean,
                lightning_cbb_per_s_sd,
                lightning_cbb_per_s_min,
                lightning_thickness,
            ))))
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let lightnings_weak = lightnings.iter()
        .map(|x| 
            x.iter().map(|y| y.as_ref().and_then(|z| Some(Rc::downgrade(z)))).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    
    let orb_group = Rc::new(RefCell::new(LightningOrbGroup {
        md: RoomObjectMetadata::new(ctx, RoomObjectType::Other),
        orbs: orbs_weak,
        lightnings: lightnings_weak,
        lightning_thickness,
        orbs_dop_cache: Vec::new(),
        orb_centers_cache: vec![Point::default(); orbs.len()].into(),
        cbb_quad_cache: Vec::new(),
    }));

    (orb_group, 
        orbs.into_iter().map(|x| x as _)
        .chain(lightnings.into_iter().flatten().filter_map(|x| x).map(|x| x as _))
        .collect())
}