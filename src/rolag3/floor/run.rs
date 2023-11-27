use super::map_object::Room;

pub struct RunFloorContext<'a> {
    pub num_ticks: u32,
    pub room: &'a mut Room,
}

// run_floor() should be called once per frame
pub fn run_floor(ctx: RunFloorContext) {
    for _ in 0 .. ctx.num_ticks {
        let tick_ctx = RunFloorTickContext {
            room: ctx.room,
        };
        run_floor_tick(tick_ctx);
    }
}

struct RunFloorTickContext<'a> {
    pub room: &'a mut Room,
}

fn run_floor_tick(_ctx: RunFloorTickContext) {

}
