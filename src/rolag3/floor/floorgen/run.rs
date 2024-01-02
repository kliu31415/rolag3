use std::collections::{VecDeque, HashSet, HashMap};

use image::ImageBuffer;
use rand::Rng;
use rand::{distributions::WeightedIndex, rngs::StdRng};
use rand::distributions::Distribution;

use crate::rolag3::floor::floorgen::steiner::{GraphEdge, compute_approx_steiner_tree};
use crate::rolag3::floor::room::RoomConnectionInfo;
use crate::rolag3::floor::room_object::room_object_def::RoomObjectId;
use crate::rolag3::floor::rooms::hallway1::{HallwayGridCell, make_hallway1};
use crate::rolag3::floor::{floor_def::RoomId, room::Room};

pub struct GenFloorArgs<'a> {
    pub grid_w: u32,
    pub grid_h: u32,

    pub ttc_min: f64,
    pub ttc_max: f64,

    pub gen_initial_room_fn: GenFloorRoomFn,
    pub gen_normal_room_fns: Vec<GenFloorRoomFn>,

    pub rng: &'a mut StdRng,
    pub room_object_id_counter: &'a mut RoomObjectId,

    pub save_debug_data: bool,
}

impl<'a> GenFloorArgs<'a> {

}

pub struct GenFloorRoomFn {
    pub weight: f64,
    pub func: Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse>,
}

pub struct GenFloorRoomContext<'a> {
    pub rng: &'a mut StdRng,
    pub room_object_id_counter: &'a mut RoomObjectId,
}

pub struct GenFloorRoomResponse {
    pub room: Room,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum FloorGenGridCell {
    Empty,
    Blocked,
    PlaceRoomHere,
    Room(RoomId),
}

pub struct GenFloorResult {
    pub rooms: Vec<Room>,
    pub floor_w: u32,
    pub floor_h: u32,
    pub connections: Vec<Vec<RoomConnectionInfo>>,
}

const HALLWAY_DIST: i32 = 5;

#[inline(never)]
pub fn gen_floor(mut args: GenFloorArgs) -> GenFloorResult {
    assert!(args.ttc_min >= 0.0);
    assert!(args.ttc_min <= args.ttc_max, "expected ttc_min({}) =< ttc_max({})", args.ttc_min, args.ttc_max);
    if args.ttc_min * 1.01 > args.ttc_max {
        log::warn!("args.ttc_min({}) * 1.01 > args.ttc_max({}). Min and max too close may cause issues", args.ttc_min, args.ttc_max);
    }
    if args.ttc_max > 1e4 {
        log::warn!("args.ttc_max({}) > 1e4. Generating a floor this big may cause issues.", args.ttc_max);
    }

    assert!(args.grid_w > 0);
    assert!(args.grid_h > 0);
    if args.grid_w < 100 {
        log::warn!("grid_w({}) is small, which may cause difficulties in floor generation", args.grid_w);
    }
    if args.grid_h < 100 {
        log::warn!("grid_h({}) is small, which may cause difficulties in floor generation", args.grid_h);
    }
    if args.grid_w * args.grid_h > 2000000 {
        log::warn!("grid_w({}) * grid_h({}) is large. This may cause slow floor generation", args.grid_w, args.grid_h);
    }

    let save_debug_path = if args.save_debug_data {
        let path = format!("tmp/gen-floor-{}", rand::thread_rng().gen_range(0..u64::MAX));
        std::fs::create_dir_all(path.clone()).expect(&format!("unable to recursively create folders in path {}", path));
        log::debug!("saving gen_floor() debug info to path: {}", path);
        Some(path)
    } else {
        None
    };

    let (starting_room, normal_room_candidates) = gen_room_candidates(&mut args);

    let mut rooms = pick_rooms(&mut args, starting_room, normal_room_candidates);

    let mut grid = place_rooms(&mut args, &mut rooms);

    if let Some(ref path) = save_debug_path {
        let img = ImageBuffer::from_fn(args.grid_w, args.grid_h, |x, y| {
            match grid[x as usize][y as usize] {
                FloorGenGridCell::Empty => image::Rgb([255u8, 255u8, 255u8]),
                FloorGenGridCell::Blocked => image::Rgb([255u8, 0u8, 0u8]),
                FloorGenGridCell::PlaceRoomHere => image::Rgb([0u8, 255u8, 0u8]),
                FloorGenGridCell::Room(id) => {
                    if id == 0 {
                        image::Rgb([127u8 * ((x % 2) as u8), 0u8, 255u8])
                    } else {
                        image::Rgb([0u8, 0u8, 255u8])
                    }
                }
            }
        });
        log::debug!("saving rooms on grid image");
        img.save(format!("{}/rooms-on-grid.png", path)).expect("unable to save rooms on grid image");
    }

    // now generate hallways that link together rooms
    // Hallways are 5 away from rooms. Find hallway tiles using a BFS.
    let (hallway_grid, mst_vertexes, graph_edges) = make_hallway_candidate_grid(&mut args, &rooms, &grid);

    if let Some(ref path) = save_debug_path {
        let req_set = mst_vertexes.keys().map(|v| *v).collect::<HashSet<_>>();
        let img = ImageBuffer::from_fn(args.grid_w, args.grid_h, |x, y| {
            if req_set.contains(&(x, y)) {
                image::Rgb([0u8, 255u8, 255u8]) 
            } else if hallway_grid[x as usize][y as usize] {
                image::Rgb([0u8, 0u8, 0u8])
            } else {
                image::Rgb([255u8, 255u8, 255u8])
            }
        });
        log::debug!("saving gen floor hallway candidates image");
        img.save(format!("{}/hallway-candidates.png", path)).expect("unable to save hallway candidate image");
    }

    log::trace!("start compute steiner tree");
    let mut steiner_tree = compute_approx_steiner_tree(&mst_vertexes.keys().cloned().collect(), &graph_edges);
    log::trace!("end compute steiner tree");
    // remove steiner tree edges (A, B) where A and B are both room cells. These are dummy edges used to simulate the
    // fact that as long as one room cell is in the steiner tree, the entire room is.
    steiner_tree.0.retain(|((x1, y1), (x2, y2))| 
        !matches!(grid[*x1 as usize][*y1 as usize], FloorGenGridCell::Room(_)) 
        || !matches!(grid[*x2 as usize][*y2 as usize], FloorGenGridCell::Room(_)));
    
    if let Some(ref path) = save_debug_path {
        let mut st_set = HashSet::new();
        steiner_tree.0.iter().for_each(|(a, b)| {
            st_set.insert(*a);
            st_set.insert(*b);
        });
        let req_set = mst_vertexes.keys().map(|v| *v).collect::<HashSet<_>>();
        let img = ImageBuffer::from_fn(args.grid_w, args.grid_h, |x, y| {
            if req_set.contains(&(x, y)) {
                image::Rgb([0u8, 255u8, 255u8]) 
            } else if st_set.contains(&(x, y)) {
                image::Rgb([0u8, 0u8, 0u8])
            } else if let FloorGenGridCell::Room(id) = grid[x as usize][y as usize] {
                if id == 0 {
                    image::Rgb([127u8 * ((x % 2) as u8), 0u8, 255u8])
                } else {
                    image::Rgb([0u8, 0u8, 255u8])
                }
            } else {
                image::Rgb([255u8, 255u8, 255u8])
            }
        });
        log::debug!("saving gen floor steiner tree image");
        img.save(format!("{}/steiner-tree.png", path)).expect("unable to save steiner tree image");
    }

    let smeared_steiner_hallway_grid = compute_smeared_steiner_hallway_grid(&mut args, &grid, &steiner_tree.0);
    
    if let Some(ref path) = save_debug_path {
        let mut st_set = HashSet::new();
        steiner_tree.0.iter().for_each(|(a, b)| {
            st_set.insert(*a);
            st_set.insert(*b);
        });
        let req_set = mst_vertexes.keys().map(|v| *v).collect::<HashSet<_>>();
        let img = ImageBuffer::from_fn(args.grid_w, args.grid_h, |x, y| {
            if req_set.contains(&(x, y)) {
                image::Rgb([0u8, 255u8, 255u8]) 
            } else if smeared_steiner_hallway_grid[x as usize][y as usize] == HallwayGridCell::Hallway {
                image::Rgb([255u8, 255u8, 0u8])
            } else if smeared_steiner_hallway_grid[x as usize][y as usize] == HallwayGridCell::Wall {
                image::Rgb([0u8, 0u8, 0u8])
            } else if let FloorGenGridCell::Room(id) = grid[x as usize][y as usize] {
                if id == 0 {
                    image::Rgb([127u8 * ((x % 2) as u8), 0u8, 255u8])
                } else {
                    image::Rgb([0u8, 0u8, 255u8])
                }
            } else {
                image::Rgb([255u8, 255u8, 255u8])
            }
        });
        log::debug!("saving gen floor finalized image");
        img.save(format!("{}/floor-final.png", path)).expect("unable to save floor finalized image");
    }

    for x in 0..smeared_steiner_hallway_grid.len() {
        for y in 0..smeared_steiner_hallway_grid[x].len() {
            match smeared_steiner_hallway_grid[x][y] {
                HallwayGridCell::Empty => {},
                HallwayGridCell::Hallway | HallwayGridCell::Wall => {
                    assert!(!matches!(grid[x][y], FloorGenGridCell::Room(_)));
                }
            }
        }
    }

    let mut hallway_rooms = compute_hallway_rooms(
        &mut grid, 
        args.grid_w, 
        args.grid_h, 
        &smeared_steiner_hallway_grid, 
        args.rng, 
        args.room_object_id_counter, 
        rooms.len());
    rooms.append(&mut hallway_rooms);

    let rci = get_rci(&rooms, &grid, &steiner_tree.0, &mst_vertexes);

    GenFloorResult {
        rooms,
        floor_w: args.grid_w, // TODO: use the smallest bounding box for floor_w/floor_h
        floor_h: args.grid_h,
        connections: rci,
    }
}

#[inline(never)]
fn gen_room_candidates(args: &mut GenFloorArgs) -> (Room, Vec<Room>) {
    let normal_room_weights = WeightedIndex::new(args.gen_normal_room_fns.iter().map(|x| x.weight)).expect("failed to unwrap WeightedIndex made from room gen weights");
    let mut gen_room_ctx = GenFloorRoomContext {
        rng: args.rng,
        room_object_id_counter: args.room_object_id_counter,
    };
    let starting_room = (args.gen_initial_room_fn.func)(&mut gen_room_ctx).room;
    assert!(starting_room.ttc <= args.ttc_max);
    let mut normal_room_candidates = Vec::new();
    let num_nrc = 100;
    while normal_room_candidates.len() < num_nrc {
        let idx = normal_room_weights.sample(args.rng);
        let mut gen_room_ctx = GenFloorRoomContext {
            rng: args.rng,
            room_object_id_counter: args.room_object_id_counter,
        };
        let f = &mut args.gen_normal_room_fns[idx];
        normal_room_candidates.push((f.func)(&mut gen_room_ctx).room);
    }
    (starting_room, normal_room_candidates)
}

#[inline(never)]
fn pick_rooms(args: &mut GenFloorArgs, starting_room: Room, normal_room_candidates: Vec<Room>) -> Vec<Room> {
    let mut tries1 = 0;
    loop {
        tries1 += 1;
        assert!(tries1 < 10000);
        let mut chosen_nrcs = HashSet::new();
        let mut ttc = starting_room.ttc;
        let mut tries2 = 0;
        while ttc < args.ttc_min {
            tries2 += 1;
            assert!(tries2 < 10000);
            let nrc_idx = args.rng.gen_range(0..normal_room_candidates.len());
            if !chosen_nrcs.contains(&nrc_idx) {
                chosen_nrcs.insert(nrc_idx);
                ttc += normal_room_candidates[nrc_idx].ttc;
            }
        }
        
        if ttc <= args.ttc_max {
            let mut rooms = Vec::new();
            rooms.push(starting_room);
            let mut r_cnt = 0;
            normal_room_candidates.into_iter().filter(|_| {
                let res = chosen_nrcs.contains(&r_cnt);
                r_cnt += 1;
                res
            }).for_each(|r| rooms.push(r));
            return rooms;
        }
    }
}

#[inline(never)]
fn place_rooms(args: &mut GenFloorArgs, rooms: &mut Vec<Room>) -> Vec<Vec<FloorGenGridCell>> {
    let mut grid = vec![vec![FloorGenGridCell::Empty; args.grid_h as usize]; args.grid_w as usize];
    grid[(args.grid_w as usize) / 2][(args.grid_h as usize) / 2] = FloorGenGridCell::PlaceRoomHere;

    let mut room_place_candidates = Vec::new();
    let mut psum_blocked = vec![vec![0; args.grid_h as usize]; args.grid_w as usize];
    let mut psum_place_room_here = vec![vec![0; args.grid_h as usize]; args.grid_w as usize];
    for (i, room) in rooms.iter_mut().enumerate() {
        assert!(room.ttc >= 0.0);
        // Rooms need to be at least 5x5, because a connection is 5 tiles wide (including the walls; the inside is 3)
        assert!(room.width >= 5);
        assert!(room.height >= 5);

        // don't place rooms within "buffer" of the edge of the grid. This makes logic for connecting rooms to hallways
        // easier by ensuring a room can connect to a hallway on all 4 sides
        let buffer = 10;
        if room.width + buffer*2 > args.grid_w || room.height + buffer*2 > args.grid_h {
            continue;
        }

        room_place_candidates.clear();

        psum_blocked.iter_mut().for_each(|v| v.fill(0));
        psum_place_room_here.iter_mut().for_each(|v| v.fill(0));
        for x in 0..(args.grid_w as usize){
            for y in 0..(args.grid_h as usize) {
                (psum_blocked[x][y], psum_place_room_here[x][y]) = match grid[x][y] {
                    FloorGenGridCell::Empty => (0, 0),
                    FloorGenGridCell::Blocked | FloorGenGridCell::Room(_) => (1, 0),
                    FloorGenGridCell::PlaceRoomHere => (0, 1),
                };
                if x > 0 {
                    psum_blocked[x][y] += psum_blocked[x-1][y];
                    psum_place_room_here[x][y] += psum_place_room_here[x-1][y];
                    if y > 0 {
                        psum_blocked[x][y] -= psum_blocked[x-1][y-1];
                        psum_place_room_here[x][y] -= psum_place_room_here[x-1][y-1];
                    }
                }
                if y > 0 {
                    psum_blocked[x][y] += psum_blocked[x][y-1];
                    psum_place_room_here[x][y] += psum_place_room_here[x][y-1];
                }
            }
        }

        let xmax = args.grid_w - room.width - buffer;
        let ymax = args.grid_h - room.height - buffer;
        for x1 in buffer as usize..=xmax as usize {
            for y1 in buffer as usize..=ymax as usize {
                let x2 = x1 + (room.width as usize) - 1;
                let y2 = y1 + (room.height as usize) - 1;
                let blocked = psum_blocked[x2][y2] - psum_blocked[x1-1][y2] - psum_blocked[x2][y1-1] + psum_blocked[x1-1][y1-1];
                let prh = psum_place_room_here[x2][y2] - psum_place_room_here[x1-1][y2] - psum_place_room_here[x2][y1-1] + psum_place_room_here[x1-1][y1-1];

                if blocked == 0 && prh > 0 {
                    room_place_candidates.push((x1 as u32, y1 as u32));
                }
            }
        }

        /*
        for x in buffer..=xmax {
            let mut blocked_count = 0;
            let mut place_room_here_count = 0;
            for i in x..(x+room.width) {
                for j in 0..(room.height) {
                    match grid[i as usize][j as usize] {
                        FloorGenGridCell::Empty => {},
                        FloorGenGridCell::Blocked | FloorGenGridCell::Room(_)  => blocked_count += 1,
                        FloorGenGridCell::PlaceRoomHere => place_room_here_count += 1,
                    }
                }
            }

            for y in buffer..=ymax {
                if place_room_here_count > 0 && blocked_count == 0 {
                    room_place_candidates.push((x, y));
                }
                
                if y != ymax { 
                    // move down one row by removing the top row and adding the next row as the new bottom row
                    for i in x..(x + room.width) {
                        match grid[i as usize][y as usize] {
                            FloorGenGridCell::Empty => {},
                            FloorGenGridCell::Blocked | FloorGenGridCell::Room(_) => blocked_count -= 1,
                            FloorGenGridCell::PlaceRoomHere => place_room_here_count -= 1,
                        }
                    }

                    for i in x..(x + room.width) {
                        match grid[i as usize][(y + room.height) as usize] {
                            FloorGenGridCell::Empty => {},
                            FloorGenGridCell::Blocked | FloorGenGridCell::Room(_) => blocked_count += 1,
                            FloorGenGridCell::PlaceRoomHere => place_room_here_count += 1,
                        }
                    }
                }
            }
        }
        */

        log::trace!("found {} candidate locations to place room #{}", room_place_candidates.len(), i);

        if room_place_candidates.is_empty() {
            panic!("unable to find place to place room. This should ideally be handled more gracefully.");
        }

        let position = room_place_candidates[args.rng.gen_range(0..room_place_candidates.len())];
        room.upper_left_x = position.0;
        room.upper_left_y = position.1;
        // room.id = rooms.len()

        let surround = HALLWAY_DIST * 2;
        let xmin = i32::max(room.upper_left_x as i32 - surround, 0) as u32;
        let xmax = i32::min((room.upper_left_x + room.width) as i32 + surround, args.grid_w as i32) as u32;
        let ymin = i32::max(room.upper_left_y as i32 - surround, 0) as u32;
        let ymax = i32::min((room.upper_left_y + room.height) as i32 + surround, args.grid_h as i32) as u32;
        for x in xmin..xmax {
            for y in ymin..ymax {
                if x==xmin || x+1==xmax || y==ymin || y+1==ymax {
                    if grid[x as usize][y as usize] == FloorGenGridCell::Empty {
                        grid[x as usize][y as usize] = FloorGenGridCell::PlaceRoomHere;
                    }
                    continue;
                }

                if x>=room.upper_left_x && x<room.upper_left_x + room.width && 
                   y>=room.upper_left_y && y<room.upper_left_y + room.height {
                    assert!(grid[x as usize][y as usize] == FloorGenGridCell::Empty || grid[x as usize][y as usize] == FloorGenGridCell::PlaceRoomHere, "Unexpected grid[x][y]={:?}", grid[x as usize][y as usize]);
                    grid[x as usize][y as usize] = FloorGenGridCell::Room(i);
                    continue;
                }

                grid[x as usize][y as usize] = FloorGenGridCell::Blocked;
            }
        }
    }

    return grid;
}

#[inline(never)]
fn make_hallway_candidate_grid(
    args: &mut GenFloorArgs, 
    rooms: &Vec<Room>, 
    grid: &Vec<Vec<FloorGenGridCell>>
) -> (Vec<Vec<bool>>, HashMap<(u32, u32), RoomId>, Vec<GraphEdge<(u32, u32)>>) {
    let mut q = VecDeque::new();
    let mut dist = vec![vec![None; args.grid_h as usize]; args.grid_w as usize];
    for i in 0..(args.grid_w as i32) {
        for j in 0..(args.grid_h as i32) {
            if let FloorGenGridCell::Room(_) = grid[i as usize][j as usize] {
                q.push_back((i, j));
                dist[i as usize][j as usize] = Some(0);
            }
        }
    }

    let mut hallway_grid = vec![vec![false; args.grid_h as usize]; args.grid_w as usize];
    while !q.is_empty() {
        let cur = q.pop_front().unwrap();
        if dist[cur.0 as usize][cur.1 as usize].unwrap() == HALLWAY_DIST {
            hallway_grid[cur.0 as usize][cur.1 as usize] = true;
        }
        for dx in -1..=1 {
            for dy in -1..=1 {
                let next = (cur.0 + dx, cur.1 + dy);
                if next.0<0 || next.0>=(args.grid_w as i32) || next.1<0 || next.1>=(args.grid_h as i32) {
                    continue;
                }
                if dist[next.0 as usize][next.1 as usize].is_some() {
                    continue;
                }
                dist[next.0 as usize][next.1 as usize] = Some(dist[cur.0 as usize][cur.1 as usize].unwrap() + 1);
                q.push_back(next);
            }
        }
    }

    // add some candidate connections from each room to the surrounding hallways
    let mut graph_edges = Vec::new();
    let mut req_vertexes_to_room_id = HashMap::new();
    for (i, room) in rooms.iter().enumerate() {
        let mut connections = Vec::new();
        // these assertions are to be extra-safe. They should never trigger, because room w/h is checked for earlier.
        assert!(room.width >= 5);
        assert!(room.height >= 5);
        let mut candidates = room.connection_candidates.clone();
        assert_ne!(candidates.len(), 0);
        loop {
            let candidate = candidates[args.rng.gen_range(0..candidates.len())];
            connections.push(candidate);
            candidates.retain(|(x, y, _)| candidate.0.abs_diff(*x) >= 7 || candidate.1.abs_diff(*y) >= 7);
            if candidates.is_empty() {
                break;
            }
            if connections.len() >= 6 { // magic number of 6 connection candidates per room
                break;
            }
        }
    
        assert!(!connections.is_empty(), "room {} has no connections to hallways and is isolated", i);

        for (cx, cy, _) in connections.iter() {
            let (dx, dy) = if *cx == 0 { // Left
                (-1, 0)
            } else if *cy == 0 { // Up
                (0, -1)
            } else if cx + 1 == room.width { // Right
                (1, 0)
            } else if cy + 1 == room.height { // Down
                (0, 1)
            } else {
                panic!("room connection ({}, {}) is not in any cardinal direction", *cx, *cy);
            };
            let key = (room.upper_left_x + cx, room.upper_left_y + cy);
            assert!(!req_vertexes_to_room_id.contains_key(&key), "cannot assign the same connection tile to multiple rooms");
            req_vertexes_to_room_id.insert(key, i);
            (0..=HALLWAY_DIST).into_iter().for_each(|i| {
                let x = (room.upper_left_x + cx) as i32 + i * dx;
                let y = (room.upper_left_y + cy) as i32 + i * dy;
                hallway_grid[x as usize][y as usize] = true;
            });
        }
        let world_coord_connections = connections.iter().map(|(x, y, _)| (x + room.upper_left_x, y + room.upper_left_y));
        for ((x1, y1), (x2, y2)) in world_coord_connections.clone().skip(1).zip(world_coord_connections) {
            // add 0-cost edges between all connection tiles in the same room.
            // Only one connection vertex per room needs to be added to the Steiner tree. 0-cost edges simulates that.
            graph_edges.push(GraphEdge {
                a: (x1, y1),
                b: (x2, y2),
                weight: 0,
            });
        }
    }

    // generate an (undirected) edge list of the hallway graph
    for x in 0..args.grid_w {
        for y in 0..args.grid_h {
            if !hallway_grid[x as usize][y as usize] {
                continue;
            }
            if x + 1 != args.grid_w {
                if hallway_grid[(x+1) as usize][y as usize] {
                    graph_edges.push(GraphEdge {
                        a: (x, y),
                        b: (x+1, y),
                        weight: 1,
                    });
                }
            }
            if y + 1 != args.grid_h {
                if hallway_grid[x as usize][(y+1) as usize] {
                    graph_edges.push(GraphEdge {
                        a: (x, y),
                        b: (x, y+1),
                        weight: 1,
                    });
                }
            }
        }
    }

    (hallway_grid, req_vertexes_to_room_id, graph_edges)
}

#[inline(never)]
fn compute_smeared_steiner_hallway_grid(
    args: &mut GenFloorArgs,
    grid: &Vec<Vec<FloorGenGridCell>>,
    steiner_tree: &Vec<((u32, u32), (u32, u32))>,
) -> Vec<Vec<HallwayGridCell>> {
    let mut steiner_hallway_grid = vec![vec![HallwayGridCell::Empty; args.grid_h as usize]; args.grid_w as usize];
    for ((x1, y1), (x2, y2)) in steiner_tree.iter() {
        if !matches!(grid[*x1 as usize][*y1 as usize], FloorGenGridCell::Room(_)) {
            steiner_hallway_grid[*x1 as usize][*y1 as usize] = HallwayGridCell::Hallway;
        }
        if !matches!(grid[*x2 as usize][*y2 as usize], FloorGenGridCell::Room(_)) {
            steiner_hallway_grid[*x2 as usize][*y2 as usize] = HallwayGridCell::Hallway;
        }
    }

    // smear the hallway to make it 3 wide
    let mut smeared_steiner_hallway_grid = vec![vec![HallwayGridCell::Empty; args.grid_h as usize]; args.grid_w as usize];
    for x in 0..(args.grid_w as i32) {
        for y in 0..(args.grid_h as i32) {
            if steiner_hallway_grid[x as usize][y as usize] != HallwayGridCell::Hallway {
                continue;
            }
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let i = x + dx;
                    let j = y + dy;
                    if i < 0 || j < 0 || i==(args.grid_w as i32) || j==(args.grid_h as i32) {
                        continue;
                    }
                    if !matches!(grid[i as usize][j as usize], FloorGenGridCell::Room(_)) {
                        smeared_steiner_hallway_grid[i as usize][j as usize] = HallwayGridCell::Hallway;
                    }
                }
            }
        }
    }

    // smear the hallway again to generate walls bounding the hallway
    for x in 0..(args.grid_w as i32) {
        for y in 0..(args.grid_h as i32) {
            if smeared_steiner_hallway_grid[x as usize][y as usize] != HallwayGridCell::Hallway {
                continue;
            }
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let i = x + dx;
                    let j = y + dy;
                    if i < 0 || j < 0 || i==(args.grid_w as i32) || j==(args.grid_h as i32) {
                        continue;
                    }
                    if !matches!(grid[i as usize][j as usize], FloorGenGridCell::Room(_)) &&
                       !matches!(smeared_steiner_hallway_grid[i as usize][j as usize], HallwayGridCell::Hallway) {
                        smeared_steiner_hallway_grid[i as usize][j as usize] = HallwayGridCell::Wall;
                    }
                }
            }
        }
    }

    smeared_steiner_hallway_grid
}

fn compute_hallway_rooms(
    grid: &mut Vec<Vec<FloorGenGridCell>>, 
    grid_w: u32, 
    grid_h: u32, 
    sshg: &Vec<Vec<HallwayGridCell>>,
    rng: &mut StdRng,
    room_object_id_counter: &mut RoomObjectId,
    room_count: usize,
) -> Vec<Room> {
    let (hallway_bbs, hid_grid) = get_disjoint_hallways(&sshg);
    for x in 0..grid_w {
        for y in 0..grid_h {
            if let Some(hid) = hid_grid[x as usize][y as usize] {
                assert!(!matches!(grid[x as usize][y as usize], FloorGenGridCell::Room(_)));
                grid[x as usize][y as usize] = FloorGenGridCell::Room(hid + room_count);
            }
        }
    }

    let mut hallway_rooms = Vec::new();
    for (i, bb) in hallway_bbs.iter().enumerate() {
        let hallway = make_hallway1(&sshg, &hid_grid, i, &bb, rng, room_object_id_counter);
        hallway_rooms.push(hallway);
    }
    hallway_rooms
}

fn get_disjoint_hallways(hg: &Vec<Vec<HallwayGridCell>>) -> (Vec<BoundingBoxUsize>, Vec<Vec<Option<usize>>>) {
    assert!(!hg.is_empty());
    let mut hid_grid = vec![vec![None; hg[0].len()]; hg.len()];
    let mut hid_counter = 0;
    let mut bounding_boxes = Vec::new();
    for x in 0..hg.len() {
        for y in 0..hg[0].len() {
            if hg[x][y] != HallwayGridCell::Empty && hid_grid[x][y].is_none() {
                let mut bb = BoundingBoxUsize {
                    x1: x,
                    x2: x,
                    y1: y,
                    y2: y,
                };
                dfs_disjoint_hallways(hg, &mut hid_grid, hid_counter, x, y, &mut bb);
                bounding_boxes.push(bb);
                hid_counter += 1;
            }
        }
    }
    (bounding_boxes, hid_grid)
}

pub struct BoundingBoxUsize {
    pub x1: usize,
    pub x2: usize,
    pub y1: usize,
    pub y2: usize,
}

fn dfs_disjoint_hallways(
    hg: &Vec<Vec<HallwayGridCell>>, 
    hid_grid: &mut Vec<Vec<Option<usize>>>, 
    hid: usize, 
    x: usize, 
    y: usize,
    bb: &mut BoundingBoxUsize,
) {
    if hg[x][y] == HallwayGridCell::Empty {
        return;
    }
    if hid_grid[x][y].is_some() {
        return;
    }

    bb.x1 = usize::min(bb.x1, x);
    bb.x2 = usize::max(bb.x2, x);
    bb.y1 = usize::min(bb.y1, y);
    bb.y2 = usize::max(bb.y2, y);

    hid_grid[x][y] = Some(hid);

    if x > 0 {
        dfs_disjoint_hallways(hg, hid_grid, hid, x-1, y, bb);
    }
    if x+1 != hg.len() {
        dfs_disjoint_hallways(hg, hid_grid, hid, x+1, y, bb);
    }
    if y > 0 {
        dfs_disjoint_hallways(hg, hid_grid, hid, x, y-1, bb);
    }
    if y+1 != hg[0].len() {
        dfs_disjoint_hallways(hg, hid_grid, hid, x, y+1, bb);
    }
}

fn get_rci(
    rooms: &Vec<Room>, 
    grid: &Vec<Vec<FloorGenGridCell>>,
    st_vertexes: &Vec<((u32, u32), (u32, u32))>,
    mst_vertexes: &HashMap<(u32, u32), RoomId>,
) -> Vec<Vec<RoomConnectionInfo>> {
    let st_vertexes = st_vertexes.iter()
        .flat_map(|(a, b)| [*a, *b]);
    let mut rci = vec![Vec::new(); rooms.len()];
    for k in st_vertexes {
        if let Some(v) = mst_vertexes.get(&k) {
            let FloorGenGridCell::Room(r1id) = grid[k.0 as usize][k.1 as usize] else {
                panic!("floor gen grid[{}][{}]={:?}, but expected room as r2id", 
                k.0 as usize, 
                k.1 as usize, 
                grid[k.0 as usize][k.1 as usize]);
            };
            assert_eq!(*v, r1id, "Req vertexes map and grid disagree on room ID");
            let x1 = k.0 - rooms[r1id].upper_left_x;
            let y1 = k.1 - rooms[r1id].upper_left_y;
            let dir1_array = rooms[r1id].connection_candidates.iter()
                .filter(|(x, y, _)| *x==x1 && *y==y1)
                .map(|(_, _, d)| *d)
                .collect::<Vec<_>>();
            assert_eq!(dir1_array.len(), 1);
            let dir1 = dir1_array[0];
            let floor_x2 = k.0 as i32 + dir1.to_dxy().0;
            let floor_y2 = k.1 as i32 + dir1.to_dxy().1;
            assert!(floor_x2 >= 0);
            assert!(floor_y2 >= 0);
            let FloorGenGridCell::Room(r2id) = grid[floor_x2 as usize][floor_y2 as usize] else {
                panic!("floor gen grid[{}][{}]={:?}, but expected room as r2id", 
                    floor_x2, 
                    floor_y2, 
                    grid[floor_x2 as usize][floor_y2 as usize]);
            };
            let room_x2 = floor_x2 - (rooms[r2id].upper_left_x as i32);
            let room_y2 = floor_y2 - (rooms[r2id].upper_left_y as i32);
            assert!(room_x2 >= 0);
            assert!(room_y2 >= 0);

            let rci1 = RoomConnectionInfo {
                x: x1,
                y: y1,
                direction: dir1,
                connects_to_room_id: r2id,
                connects_to_x: room_x2 as u32,
                connects_to_y: room_y2 as u32,
            };

            let rci2 = RoomConnectionInfo {
                x: room_x2 as u32,
                y: room_y2 as u32,
                direction: dir1.inverted(),
                connects_to_room_id: r1id,
                connects_to_x: x1,
                connects_to_y: y1,
            };
            rci[r1id].push(rci1);
            rci[r2id].push(rci2);
        }
    }
    rci
}