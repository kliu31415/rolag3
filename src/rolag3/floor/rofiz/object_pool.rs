use std::collections::VecDeque;

use crate::util::time::now_unix;

use super::rofiz_object::{RofizObjBasicWall, RofizObjMovable};

use rayon::prelude::*;

/* The object pool uses a data structure similar to a probing hash table. 
   -When the hash table array is resized, no objects are moved. We could move objects, but we'd then have to refactor
   the object pool to use opaque indexes, which involves making the hash table array fields private, which would involve
   some refactoring to satisfy the borrow checker.
   -When an object is inserted, a random index in the less loaded half of the hash table array is selected, 
   and probing begins from there. The object is inserted into the first free location is used. The index of this
   location is a non-opaque identifier for the object.
   -Empirically, it takes <50 tries to insert a new object with a load factor of 0.7. Inserting into the less loaded
   half of the hash table helps. When a random index across the whole hash table is picked, the number of tries often
   reaches 1000+ because the more loaded half of the hash table is disproportionately heavily populated.
   -As more explanation, note that when the hash table is resized, the first half of the hash table has a load factor
   of MAX_LOAD_FACTOR, but the second half has a load factor of 0. Naturally, we should try to insert into the second
   half at this point.
   -Sometimes, when multiple RofizObjects are deleted, the second half becomes the more loaded half
 */

/* Benchmarks for how long start_moafc() takes with various values of PAR_ITER_MIN_LEN on i9-11980HK
-format is milliseconds with (<10 movable objs, ~4000 alive circular projectiles, ~4000 just-dead circular projectiles)
-1: (0.03, 0.17, 0.09)
-32: (0.0008, 0.14, 0.06)
-128: (0.0009, 0.15, 0.05)
-512: (0.001, 0.016-0.020 (high variance), 0.04)
*/

const PAR_ITER_MIN_LEN: usize = 128;

const INITIAL_POOL_SIZE: usize = 16;
const MAX_LOAD_FACTOR: f64 = 0.47;

pub struct RofizObjPool {
    pub basic_walls: Vec<Option<RofizObjBasicWall>>,
    pub movable: Vec<Option<RofizObjMovable>>,

    bw_half1_occupancy: usize,
    bw_half2_occupancy: usize,
    bw_occupancy: usize,
    mo_half1_occupancy: usize,
    mo_half2_occupancy: usize,
    mo_occupancy: usize,

    timings: VecDeque<f64>,
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
            bw_half1_occupancy: 0,
            bw_half2_occupancy: 0,
            bw_occupancy: 0,
            mo_half1_occupancy: 0,
            mo_half2_occupancy: 0,
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
            self.bw_half1_occupancy = self.bw_occupancy;
            self.bw_half2_occupancy = 0;
        }

        let offset = if self.bw_half1_occupancy < self.bw_half2_occupancy {
            self.bw_half1_occupancy += 1;
            0
        } else {
            self.bw_half2_occupancy += 1;
            self.basic_walls.len() / 2
        };
        let mut cur = offset + hsh(bw.id) % (self.basic_walls.len() / 2);

        let mut tries = 0;
        loop {
            tries += 1;
            if tries == 100 {
                log::warn!("unable to add basic wall to rofiz pool after 100 tries");
            }
            if cur >= self.basic_walls.len() {
                cur -= self.basic_walls.len();
            }
            if self.basic_walls[cur].is_none() {
                if cur < self.basic_walls.len() / 2 {
                    self.bw_half1_occupancy += 1;
                } else {
                    self.bw_half2_occupancy += 1;
                }
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
            self.mo_half1_occupancy = self.mo_occupancy;
            self.mo_half2_occupancy = 0;
        }

        let offset = if self.mo_half1_occupancy < self.mo_half2_occupancy {
            0
        } else {
            self.movable.len() / 2
        };
        let mut cur = offset + hsh(mo.id) % (self.movable.len() / 2);
        let orig_cur = cur % self.movable.len();

        let mut tries = 0;
        loop {
            tries += 1;
            if tries == 100 {
                log::warn!("unable to add movable to rofiz pool after 100 tries. id={}, Orig_cur={}, Mo Occupancy={}, Movable len={}", mo.id, orig_cur, self.mo_occupancy, self.movable.len());
            }
            if cur >= self.movable.len() {
                cur -= self.movable.len();
            }
            if self.movable[cur].is_none() {
                if cur < self.movable.len() / 2 {
                    self.mo_half1_occupancy += 1;
                } else {
                    self.mo_half2_occupancy += 1;
                }
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
        if (a.idx as usize) < self.movable.len() / 2 {
            self.mo_half1_occupancy -= 1;
        } else {
            self.mo_half2_occupancy -= 1;
        }
    }

    pub fn del_bw(&mut self, a: &RofizObjPoolRef) {
        assert!(a.is_bw);
        self.bw_occupancy -= 1;
        self.basic_walls[a.idx as usize] = None;
        if (a.idx as usize) < self.basic_walls.len() / 2 {
            self.bw_half1_occupancy -= 1;
        } else {
            self.bw_half2_occupancy -= 1;
        }
    }
    
    pub fn start_moafc(&mut self) {
        let begin: f64 = now_unix();
        self.movable.par_iter_mut().with_min_len(PAR_ITER_MIN_LEN).for_each(|x| {
            match x {
                None => {},
                Some(ref mut v) => v.start_moafc(),
            }
        });
        let end = now_unix();
        self.timings.push_back(end - begin);
        if self.timings.len() > 150 {
            while self.timings.len() > 100 {
                self.timings.pop_front();
            }
            //let numerator: f64 = self.timings.iter().sum();
            //log::warn!("start_moafc_time={}ms", 1e3 * numerator / 100.0);
        }
    }
}

fn hsh(v: usize) -> usize {
    // 373 is prime
    v * 373
}