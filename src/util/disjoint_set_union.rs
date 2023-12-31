pub struct DisjointSetUnion {
    p: Box<[usize]>,
}

impl DisjointSetUnion {
    pub fn new(n: usize) -> Self {
        Self {
            p: (0..n).collect()
        }
    }

    // returns true if a union of two disjoint sets actually happened
    pub fn union(&mut self, a: usize, b: usize) -> bool { 
        let ar = self.find(a);
        let br = self.find(b);
        self.p[ar] = br;
        return ar != br
    }

    pub fn find(&mut self, a: usize) -> usize {
        let mut cur = a;
        while cur != self.p[cur] {
            let next = self.p[cur];
            self.p[cur] = self.p[next]; 
            cur = next;
        }
        cur
    }
}