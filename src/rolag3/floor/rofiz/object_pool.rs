use std::collections::VecDeque;

use crate::util::time::now_unix;

use super::rofiz_object::{RofizObjBasicWall, RofizObjMovable};

use rayon::prelude::*;

const INITIAL_POOL_SIZE: usize = 16;
const MAX_LOAD_FACTOR: f64 = 0.7;

pub struct RofizObjPool {
    pub basic_walls: Vec<Option<RofizObjBasicWall>>,
    pub movable: Vec<Option<RofizObjMovable>>,
    pub bw_occupancy: usize,
    pub mo_occupancy: usize,

    pub timings: VecDeque<f64>,
}

#[derive(Debug, Clone, Copy)]
pub struct RofizObjPoolRef {
    pub idx: u16,
    pub is_bw: bool,
}

impl RofizObjPool {
    pub fn new() -> Self {
        let mut basic_walls = Vec::new();
        basic_walls.reserve(INITIAL_POOL_SIZE);
        for _ in 0..INITIAL_POOL_SIZE {
            basic_walls.push(None);
        }
        let mut movable = Vec::new();
        movable.reserve(INITIAL_POOL_SIZE);
        for _ in 0..INITIAL_POOL_SIZE {
            movable.push(None);
        }
        Self {
            basic_walls,
            movable,
            timings: VecDeque::new(),
            bw_occupancy: 0,
            mo_occupancy: 0,
        }
    }

    pub fn get_mo_mo_mut(&mut self, a: &RofizObjPoolRef, b: &RofizObjPoolRef) -> (&mut RofizObjMovable, &mut RofizObjMovable) {
        assert!(!a.is_bw);
        assert!(!b.is_bw);
        assert!(a.idx != b.idx);
        if a.idx < b.idx {
            let (l, r) = self.movable.split_at_mut(b.idx as usize);
            return (l[a.idx as usize].as_mut().unwrap(), r[0].as_mut().unwrap())
        } else {
            let (l, r) = self.movable.split_at_mut(a.idx as usize);
            return (r[0].as_mut().unwrap(), l[b.idx as usize].as_mut().unwrap())
        }
    }

    pub fn get_mo(&self, a: &RofizObjPoolRef) -> &RofizObjMovable {
        assert!(!a.is_bw);
        self.movable[a.idx as usize].as_ref().unwrap()
    }

    pub fn get_mo_mut(&mut self, a: &RofizObjPoolRef) -> &mut RofizObjMovable {
        assert!(!a.is_bw);
        self.movable[a.idx as usize].as_mut().unwrap()
    }

    pub fn _get_mo_bw_mut(&mut self, a: &RofizObjPoolRef, b: &RofizObjPoolRef) -> (&mut RofizObjMovable, &mut RofizObjBasicWall) {
        assert!(!a.is_bw);
        assert!(b.is_bw);
        (self.movable[a.idx as usize].as_mut().unwrap(), self.basic_walls[b.idx as usize].as_mut().unwrap())
    }

    pub fn get_bw(&mut self, a: &RofizObjPoolRef) -> &RofizObjBasicWall {
        assert!(a.is_bw);
        self.basic_walls[a.idx as usize].as_ref().unwrap()
    }

    pub fn add_bw(&mut self, bw: RofizObjBasicWall) -> RofizObjPoolRef {
        self.bw_occupancy += 1;
        if self.bw_occupancy as f64 > self.basic_walls.len() as f64 * MAX_LOAD_FACTOR {
            let new_size = self.basic_walls.len() * 2;
            self.basic_walls.reserve_exact(new_size);
            while self.basic_walls.len() < new_size {
                self.basic_walls.push(None);
            }
        }

        let mut tries = 0;
        let mut cur = bw.id;
        loop {
            tries += 1;
            if tries == 1000 {
                log::warn!("unable to add basic wall to rofiz pool after 1000 tries");
            }
            if cur >= self.basic_walls.len() {
                cur %= self.basic_walls.len();
            }
            if self.basic_walls[cur].is_none() {
                self.basic_walls[cur] = Some(bw);
                return RofizObjPoolRef { idx: cur as u16, is_bw: true };
            }
            cur += 1;
        }
    }

    pub fn add_mo(&mut self, mo: RofizObjMovable) -> RofizObjPoolRef {
        self.mo_occupancy += 1;
        if self.mo_occupancy as f64 > self.movable.len() as f64 * MAX_LOAD_FACTOR {
            let new_size = self.movable.len() * 2;
            self.movable.reserve_exact(new_size);
            while self.movable.len() < new_size {
                self.movable.push(None);
            }
        }

        let mut tries = 0;
        let mut cur = mo.id;
        loop {
            tries += 1;
            if tries == 1000 {
                log::warn!("unable to add movable to rofiz pool after 1000 tries");
            }
            if cur >= self.movable.len() {
                cur %= self.movable.len();
            }
            if self.movable[cur].is_none() {
                self.movable[cur] = Some(mo);
                return RofizObjPoolRef { idx: cur as u16, is_bw: false };
            }
            cur += 1;
        }
    }

    pub fn del_mo(&mut self, a: &RofizObjPoolRef) {
        assert!(!a.is_bw);
        self.mo_occupancy -= 1;
        self.movable[a.idx as usize] = None;
    }

    pub fn del_bw(&mut self, a: &RofizObjPoolRef) {
        assert!(a.is_bw);
        self.bw_occupancy -= 1;
        self.basic_walls[a.idx as usize] = None;
    }
    
    pub fn start_moafc(&mut self) {
        let begin = now_unix();
        self.movable.par_iter_mut().for_each(|x| {
            match x {
                None => {},
                Some(ref mut v) => v.start_moafc(),
            }
        });
        let end = now_unix();
        self.timings.push_back(end - begin);
        /*if self.timings.len() > 200 {
            while self.timings.len() > 100 {
                self.timings.pop_front();
            }
            let numerator: f64 = self.timings.iter().sum();
            log::warn!("start_moafc_time={}", numerator / 100.0);
        }*/
    }
}