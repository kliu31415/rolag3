use crate::rolag3::floor::{floor_def::Floor, room::Room};

pub struct _GenFloorArgs {
    pub grid_w: u32,
    pub grid_h: u32,

    pub ttc_min: f64,
    pub ttc_max: f64,

    pub gen_normal_room_fns: Vec<_GenFloorRoomFn>,
}

pub struct _GenFloorRoomFn {
    weight: f64,
    func: Box<dyn Fn(&_GenFloorArgs) -> Room>,
}

pub fn _gen_floor(args: _GenFloorArgs) -> Floor {
    assert!(args.ttc_min >= 0.0);
    assert!(args.ttc_min <= args.ttc_max, "expected ttc_min({}) =< ttc_max({})", args.ttc_min, args.ttc_max);
    if args.ttc_min * 1.05 > args.ttc_max {
        log::warn!("args.ttc_min({}) * 1.05 > args.ttc_max({}). Min and max too close may cause issues", args.ttc_min, args.ttc_max);
    }
    if args.ttc_max > 1e4 {
        log::warn!("args.ttc_max({}) > 1e4. Generating a floor this big may cause issues.", args.ttc_max);
    }

    if args.grid_w < 100 {
        log::warn!("grid_w({}) is small, which may cause difficulties in floor generation", args.grid_w);
    }
    if args.grid_h < 100 {
        log::warn!("grid_h({}) is small, which may cause difficulties in floor generation", args.grid_h);
    }
    if args.grid_w * args.grid_h > 2000000 {
        log::warn!("grid_w({}) * grid_h({}) is large. This may cause slow floor generation", args.grid_w, args.grid_h);
    }

    todo!();
}