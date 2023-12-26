use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response, HandleCollisionResponse}, unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, SuHandleCollisionContext, HandleCollisionLogic}, standard_unit_common::TranslateMove}, damage::DamageColor}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::shape::Shape};

/* Blue square moves erratically. It fires no projectiles.
*/

const SIDE_LEN: f32 = 1.5;
const INNER_COLOR: Color = Color::new(0.1, 0.1, 1.0, 1.0);

pub struct SquareBlue {
    movement: Option<Movement>,
}

struct Movement {
    velocity_x: f64,
    velocity_y: f64,
}

pub fn new_square_blue(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_square(-SIDE_LEN/2.0, -SIDE_LEN/2.0, SIDE_LEN);
    let us_data = SquareBlue { movement: None};

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Blue,
        hp: 10.0,
        engine_power: 15.0,
        tire_traction: 70.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .handle_collision_logic(HandleCollisionLogic::CustomFn(Box::new(handle_collision)))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareBlue>().unwrap();
    let tick_len = ctx.su_ctx.su_common.get_unit_tick_len();
    match us_data.movement {
        Some(ref m) => {
            ctx.su_ctx.su_common.set_translate_move(TranslateMove::Accelerate { ax: m.velocity_x, ay: m.velocity_y});
            if ctx.act1_ctx.get_randf64() < 0.5 * tick_len {
                us_data.movement = None;
            }
        },
        None => {
            let angle = std::f64::consts::FRAC_PI_2 * (ctx.act1_ctx.get_randi64(0..4) as f64);
            let velocity_x = f64::cos(angle);
            let velocity_y = f64::sin(angle);
            us_data.movement = Some(Movement {velocity_x, velocity_y});
        }
    }
    Act1Response::new()
}

fn draw(ctx: &mut SuDrawContext) {
    let inner_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), INNER_COLOR);
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let x = xform.dx as f32 - SIDE_LEN / 2.0;
    let y = xform.dy as f32 - SIDE_LEN / 2.0;
    let w = SIDE_LEN;
    let h = SIDE_LEN;
    let dop = ctx.draw_ctx.do_rect(inner_color, x, y, w, h);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop);
}

fn handle_collision(ctx: &mut SuHandleCollisionContext) -> HandleCollisionResponse {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareBlue>().unwrap();
    if !ctx.hc_ctx.get_other().borrow().is_spectral() {
        us_data.movement = None;
    }
    HandleCollisionResponse::new()
}