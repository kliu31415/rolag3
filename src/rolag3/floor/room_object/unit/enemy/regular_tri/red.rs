use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, HandleCollisionResponse, Team}, damage::DamageColor, unit::{standard_unit1::{StandardUnit1, StandardUnit1BuilderReq, StandardUnit1Builder, HandleCollisionLogic, SuAct1Context, SuDrawContext, SuHandleCollisionContext}, standard_unit_common::{RotateMove, TranslateMove, Budeb, BudebMaxSpeed, BudebExpiry}}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::{shape::{Shape, Polygon}, util::{regular_polygon, get_inner_polygon}}};

/* RegtriRed randomly rotates and translates in the direction of one of its vertices. When it's damaged, it moves
   faster temporarily.
*/

const BORDER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);
const INNER_COLOR: Color = Color::new(0.5, 0.0, 0.0, 1.0);
const EXCITED_COLOR: Color = Color::new(5.0, 0.1, 0.1, 1.0);

pub struct RegtriRed {
    border: Polygon,
    inner: Polygon,
    translate_dir: i64,
    rotate_dir: i64,
    should_reset_velocity: bool,
    hp_last_tick: Option<f64>,
    excitement: f64,
}

pub fn new_regtri_red(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, 2.0 * std::f64::consts::PI * ctx.get_randf64());
    let border = regular_polygon(3, 1.5);
    let inner = get_inner_polygon(0.1, &border);
    let border = Polygon::new(border);
    let inner = Polygon::new(inner);
    let shape = Shape::Polygon(border.clone());
    let us_data = RegtriRed {
        border,
        inner,
        translate_dir: ctx.get_randi64(0..3),
        rotate_dir: 2 * ctx.get_randi64(0..1) - 1,
        should_reset_velocity: false,
        hp_last_tick: None,
        excitement: 0.0,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Red,
        hp: 15000.0,
        engine_power: 12.0,
        tire_traction: 50.0,
    }).angular_power(0.5)
        .angular_traction(30.0)
        .act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .handle_collision_logic(HandleCollisionLogic::CustomFn(Box::new(handle_collision)))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<RegtriRed>().unwrap();

    let response = Act1Response::new();
    let tick_len = ctx.su_ctx.su_common.get_unit_tick_len();

    match us_data.hp_last_tick {
        Some(prev) => {
            let cur = ctx.su_ctx.su_common.get_cur_hp();
            us_data.excitement *= f64::powf(0.1, tick_len);
            us_data.excitement += f64::max(0.0, prev - cur);
            us_data.hp_last_tick = Some(cur);
        }
        None => us_data.hp_last_tick = Some(ctx.su_ctx.su_common.get_cur_hp()),
    }
    assert!(us_data.excitement >= 0.0);

    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let theta = xform.dtheta + (us_data.translate_dir as f64) * 2.0/3.0 * std::f64::consts::PI;
    ctx.su_ctx.su_common.set_translate_move(TranslateMove::Accelerate { ax: f64::cos(theta), ay: f64::sin(theta)});
    ctx.su_ctx.su_common.set_rotate_move(RotateMove::Accelerate { atheta: us_data.rotate_dir as f64 });
    let speed_mult = f64::cbrt(1.0 + us_data.excitement);
    log::warn!("excitement={}", us_data.excitement);
    ctx.su_ctx.su_common.apply_budeb(&Budeb::SpeedMult(BudebMaxSpeed::new(speed_mult, BudebExpiry::OneTick)));

    if us_data.should_reset_velocity {
        us_data.should_reset_velocity = false;
        ctx.su_ctx.su_common.set_translate_move(TranslateMove::ResetVelocity);
        ctx.su_ctx.su_common.set_rotate_move(RotateMove::ResetVelocity);
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<RegtriRed>().unwrap();
    assert!(us_data.border.vertexes.len() == 3);
    assert!(us_data.inner.vertexes.len() == 3);
    let inner_color_t = us_data.excitement / (1.0 + us_data.excitement);
    let inner_color = Color::lerp(INNER_COLOR, EXCITED_COLOR, inner_color_t as f32);
    let border_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), BORDER_COLOR);
    let inner_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), inner_color);
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let border = xform.get_transformed_polygon(&us_data.border).vertexes;
    let inner = xform.get_transformed_polygon(&us_data.inner).vertexes;
    let dop_border = ctx.draw_ctx.do_tri_fan_border(border_color, &border, &inner);
    let dop_inner = ctx.draw_ctx.do_tri_fan(inner_color, &inner);

    let dop = ctx.draw_ctx.dop_group(vec![dop_border, dop_inner].into());
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop);
}

fn handle_collision(ctx: &mut SuHandleCollisionContext) -> HandleCollisionResponse {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<RegtriRed>().unwrap();
    if !ctx.hc_ctx.get_other().borrow().is_spectral() {
        us_data.should_reset_velocity = true;
        us_data.translate_dir = ctx.hc_ctx.get_randi64(0..3);
        us_data.rotate_dir = 2 * ctx.hc_ctx.get_randi64(0..1) - 1;
    }
    HandleCollisionResponse::new()
}