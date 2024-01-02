use std::{cell::RefCell, rc::Rc, collections::VecDeque};

use rand::{rngs::StdRng, Rng};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, RoomObjectCollection, RoomObjectId}, wall::basic_wall::BasicWall, cosmetic::ground1::new_ground1, tiles::damage_tile::new_damage_tile}, rofiz::rofiz_state::RofizState, room::Room, draw::Color}, util::disjoint_set_union::DisjointSetUnion};

pub fn make_room_maze1(rng: &mut StdRng, room_object_id_counter: &mut RoomObjectId, x: u32, y: u32) -> Room {
    let maze_w = 24;
    let maze_h = 24;
    let maze = make_rectangular_maze(rng, maze_w, maze_h, 10);
    let mut rofiz = RofizState::new();
    let mut new_floor_object_ctx = NewRoomObjectContext::new(&mut rofiz, room_object_id_counter, 0.0, rng);
    let mut room_objects = RoomObjectCollection::new();

    let room_w = 5 * maze_w + 1;
    let room_h = 5 * maze_h + 1;

    for i in 0..room_w {
        let wall = BasicWall::new(&mut new_floor_object_ctx, i as u32, 0, Color::new(0.1, 0.2, 0.3, 1.0));
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, i as u32, room_h as u32 - 1, Color::new(0.1, 0.2, 0.3, 1.0));
        room_objects.add(Rc::new(RefCell::new(wall)));
    }
    
    for i in 1..(room_h-1) {
        let wall = BasicWall::new(&mut new_floor_object_ctx, 0, i as u32, Color::new(0.1, 0.2, 0.3, 1.0));
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, (room_w - 1) as u32, i as u32, Color::new(0.1, 0.2, 0.3, 1.0));
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    let ground = new_ground1(&mut new_floor_object_ctx, Color::new(0.02, 0.0, 0.0, 1.0), 1, 1, room_w as u32 - 2, room_h as u32 - 2);
    room_objects.add(Rc::new(RefCell::new(ground)));

    let maze_wall_array = maze.get_maze_wall_array(4);
    for x in 0..(maze_wall_array.len()) {
        for y in 0..(maze_wall_array[0].len()) {
            if maze_wall_array[x][y] {
                let tile = new_damage_tile(&mut new_floor_object_ctx,(x + 1) as u32, (y + 1) as u32);
                room_objects.add(Rc::new(RefCell::new(tile)));
            }
        }
    }

    Room {
        upper_left_x: x,
        upper_left_y: y,
        width: room_w as u32,
        height: room_h as u32,
        tiles: Vec::new(),
        room_objects,
        rofiz,
        room_time: 0.0,
        room_cleared_at_time: None,
        minimap_texture: None,
        ttc: 20.0,
        connection_candidates: Vec::new(),
        is_hallway: false,
    }
}

// TODO: maybe make this algorithm assign random weights to graph edges and run kruskal's on the graph, selecting the 
// next-least-weight edge every step.
// Right now, this algorithm randomly selects an edge every step. The complexity of this is hard to analyze.
fn make_rectangular_maze(rng: &mut StdRng, w: usize, h: usize, additional_edges: u32) -> RectangularGraph {
    let num_cells = w * h;
    if num_cells > 10000 {
        log::warn!("generating unusually large maze (w={}, h={})", w, h);
    }
    let mut dsu = DisjointSetUnion::new(num_cells);
    let mut unions_to_go = num_cells - 1;
    let mut graph = RectangularGraph::new(w, h);
    while unions_to_go > 0 {
        let x1 = rng.gen_range(0..w) as i32;
        let y1 = rng.gen_range(0..h) as i32;
        let direction = RectGraphDir::from_idx(rng.gen_range(0..4));
        let dxy = direction.to_dxy();
        let x2 = x1 + dxy.0;
        let y2 = y1 + dxy.1;
        if x2<0 || y2<0 || x2>=(w as i32) || y2>=(h as i32) {
            continue;
        }
        let dsu1_id = (y1 * (w as i32) + x1) as usize;
        let dsu2_id = (y2 * (w as i32) + x2) as usize;
        if dsu.union(dsu1_id, dsu2_id) {
            graph.add_connection(x1, y1, direction);
            unions_to_go -= 1;
        }
    }

    for _ in 0..additional_edges {
        let mut candidates = Vec::new();
        let num_candidates_to_gen = 5;
        let mut num_tries = 0;
        while candidates.len() < num_candidates_to_gen {
            num_tries += 1;
            if num_tries > 0 && num_tries % (num_candidates_to_gen * 100) == 0{
                log::warn!("number of tries({}) to add a new rect graph edge is very large. Maze size ({}, {}), additional edges={}.", num_tries, w, h, additional_edges);
            }
            let x1 = rng.gen_range(0..w) as i32;
            let y1 = rng.gen_range(0..h) as i32;
            let direction = RectGraphDir::from_idx(rng.gen_range(0..4));
            if graph.get_has_connection(x1 as usize, y1 as usize)[direction.to_idx()] {
                continue;
            }
            let dxy = direction.to_dxy();
            let x2 = x1 + dxy.0;
            let y2 = y1 + dxy.1;
            if x2<0 || y2<0 || x2>=(w as i32) || y2>=(h as i32) {
                continue;
            }
            let dist = graph.get_distance(x1, y1, x2, y2);
            candidates.push((dist, (x1, y1), direction));
        }
        candidates.sort_by_key(|x| x.0);
        let to_add = candidates.last().unwrap();
        graph.add_connection(to_add.1.0, to_add.1.1, to_add.2);
    }

    graph
}

// undirected graph
struct RectangularGraph {
    // up, right, down, left
    has_connection: Box<[Box<[[bool; 4]]>]>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RectGraphDir {
    Up,
    Right,
    Down,
    Left,
}

impl RectGraphDir {
    fn from_idx(idx: usize) -> Self {
        match idx {
            0 => RectGraphDir::Up,
            1 => RectGraphDir::Right,
            2 => RectGraphDir::Down,
            3 => RectGraphDir::Left,
            _ => panic!("unexpected arg in from_idx({})", idx),
        }
    }

    fn to_idx(&self) -> usize {
        match self {
            RectGraphDir::Up => 0,
            RectGraphDir::Right => 1,
            RectGraphDir::Down => 2,
            RectGraphDir::Left => 3,
        }
    }

    fn to_dxy(&self) -> (i32, i32)  {
        match self {
            RectGraphDir::Up => (0, -1),
            RectGraphDir::Right => (1, 0),
            RectGraphDir::Down => (0, 1),
            RectGraphDir::Left => (-1, 0),
        }
    }

    fn inverted(&self) -> Self {
        match self {
            RectGraphDir::Up => RectGraphDir::Down,
            RectGraphDir::Right => RectGraphDir::Left,
            RectGraphDir::Down => RectGraphDir::Up,
            RectGraphDir::Left => RectGraphDir::Right,
        }
    }
}

impl RectangularGraph {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width>0 && height>0, "cannot make RectangularGraph with 0 width({}) or height({})", width, height);
        RectangularGraph { has_connection: vec![vec![[false; 4]; height].into(); width].into() }
    }

    pub fn add_connection(&mut self, x: i32, y: i32, direction: RectGraphDir) {
        let idx = (x as usize, y as usize, direction.to_idx());
        assert!(!self.has_connection[idx.0][idx.1][idx.2], "trying to add connection that already exists. idx(1)={:?}", idx);
        self.has_connection[idx.0][idx.1][idx.2] = true;
        
        let dxy = direction.to_dxy();
        let x2 = x + dxy.0;
        let y2 = y + dxy.1;
        let dir2 = direction.inverted();
        let idx2 = (x2 as usize, y2 as usize, dir2.to_idx());
        assert!(!self.has_connection[idx2.0][idx2.1][idx2.2], "trying to add connection that already exists. idx(2)={:?}", idx);
        self.has_connection[idx2.0][idx2.1][idx2.2] = true;
    }

    pub fn get_has_connection(&self, x: usize, y: usize) -> [bool; 4] {
        self.has_connection[x][y]
    }

    pub fn get_maze_wall_array(&self, corridor_width: usize) -> Box<[Box<[bool]>]> {
        assert!(corridor_width > 0, "corridor width is 0, but must be positive");

        let maze_w = self.has_connection.len();
        let maze_h = self.has_connection[0].len();
        let array_w = maze_w * (1 + corridor_width) - 1;
        let array_h = maze_h * (1 + corridor_width) - 1;
        let mut array = vec![vec![false; array_h].into_boxed_slice(); array_w].into_boxed_slice();
        for maze_x in 0..maze_w {
            for maze_y in 0..maze_h {
                let connections = self.get_has_connection(maze_x, maze_y);

                if maze_x + 1 != maze_w && !connections[RectGraphDir::Right.to_idx()] {
                    let array_x = (1 + corridor_width) * maze_x + corridor_width;
                    let array_y_begin = (1 + corridor_width) * maze_y;
                    let array_y_end = (1 + corridor_width) * maze_y + corridor_width;
                    for array_y in array_y_begin..array_y_end {
                        array[array_x][array_y] = true;
                    }
                }

                if maze_y + 1 != maze_h && !connections[RectGraphDir::Down.to_idx()] {
                    let array_y = (1 + corridor_width) * maze_y + corridor_width;
                    let array_x_begin = (1 + corridor_width) * maze_x;
                    let array_x_end = (1 + corridor_width) * maze_x + corridor_width;
                    for array_x in array_x_begin..array_x_end {
                        array[array_x][array_y] = true;
                    }
                }

            }
        }

        for maze_x in 0..(maze_w-1) {
            for maze_y in 0..(maze_h-1) {
                let array_x = (1 + corridor_width) * maze_x + corridor_width;
                let array_y = (1 + corridor_width) * maze_y + corridor_width;
                let num_surroundings_occupied = 
                    (array[array_x-1][array_y] as u32) + 
                    (array[array_x+1][array_y] as u32) + 
                    (array[array_x][array_y-1] as u32) + 
                    (array[array_x][array_y+1] as u32);
                if num_surroundings_occupied >= 2 {
                    array[array_x][array_y] = true;
                }
            }
        }

        array
    }

    fn get_distance(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
        let mut q = VecDeque::new();
        let maze_w = self.has_connection.len();
        let maze_h = self.has_connection[0].len();
        let mut dist = vec![vec![-1; maze_h].into_boxed_slice(); maze_w].into_boxed_slice();
        q.push_back((x1, y1));
        dist[x1 as usize][y1 as usize] = 0;
        while !q.is_empty() {
            let cur = *q.front().unwrap();
            q.pop_front();
            if cur.0 == x2 && cur.1 == y2 {
                return dist[cur.0 as usize][cur.1 as usize];
            }
            for i in 0..4 {
                if !self.has_connection[cur.0 as usize][cur.1 as usize][i] {
                    continue;
                }
                let dxy = RectGraphDir::from_idx(i).to_dxy();
                let next_x = cur.0 + dxy.0;
                let next_y = cur.1 + dxy.1;
                if next_x<0 || next_y<0 || next_x>=(maze_w as i32) || next_y>=(maze_h as i32) {
                    continue;
                }
                if dist[next_x as usize][next_y as usize] != -1 {
                    continue;
                }
                dist[next_x as usize][next_y as usize] = dist[cur.0 as usize][cur.1 as usize] + 1;
                q.push_back((next_x, next_y));
            }
        }
        panic!("unable to find path from ({}, {}) to ({}, {})", x1, y1, x2, y2);
     }
}