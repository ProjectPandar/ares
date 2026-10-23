//! Type1 category `std::unordered_map<int,float>` iteration, GNU 64-bit reference.
//! GCC14 libstdc++: functional_hash.h::hash<int>, hashtable.h::_M_insert_bucket_begin
//! /_M_rehash_aux(true_type), hashtable_c++0x.cc::_Prime_rehash_policy.
//! Only operator[] accumulation, lookup and iteration are needed: no reserve/erase.

#[derive(Default)]
pub(super) struct Depths {
    nodes: Vec<Node>,
    buckets: Vec<Option<usize>>,
    head: Option<usize>,
    growth: usize,
}

struct Node {
    key: i32,
    depth: f32,
    next: Option<usize>,
    previous: Option<usize>,
}

// The exact _M_next_bkt(max(floor(min_bkts)+1, old*2)) sequence with load=1,
// single insertion and no reserve. The final entry covers every distinct i32 key.
// Source prime table: libstdc++-v3/src/shared/hashtable-aux.cc::__prime_list.
const GROWTH: &[u64] = &[
    13, 29, 59, 127, 257, 541, 1109, 2357, 5087, 10273, 20753, 42043, 85229, 172933, 351061,
    712697, 1447153, 2938679, 5967347, 12117689, 24607243, 49969847, 101473717, 206062531,
    418451333, 849749479, 1725587117, 3504151727, 8589934583,
];

impl Depths {
    fn bucket(&self, key: i32) -> usize {
        // Pin the reference size_t width even when the Rust target is WASM32.
        ((key as i64 as u64) % self.buckets.len() as u64) as usize
    }

    fn find(&self, key: i32) -> Option<usize> {
        if self.buckets.is_empty() {
            return None;
        }
        let bucket = self.bucket(key);
        let mut next = self.buckets[bucket];
        while let Some(index) = next {
            let node = &self.nodes[index];
            if self.bucket(node.key) != bucket {
                break;
            }
            if node.key == key {
                return Some(index);
            }
            next = node.next;
        }
        None
    }

    pub(super) fn get(&self, key: i32) -> f32 {
        self.find(key).map_or(0.0, |index| self.nodes[index].depth)
    }

    pub(super) fn add(&mut self, key: i32, depth: f32) {
        if let Some(index) = self.find(key) {
            self.nodes[index].depth += depth;
            return;
        }
        if self.nodes.len() + 1 > self.buckets.len() {
            self.buckets = vec![None; GROWTH[self.growth] as usize];
            self.growth += 1;
            let mut next = self.head.take();
            while let Some(index) = next {
                next = self.nodes[index].next;
                self.link(index);
            }
        }
        let index = self.nodes.len();
        self.nodes.push(Node {
            key,
            depth,
            next: None,
            previous: None,
        });
        self.link(index);
    }

    fn link(&mut self, index: usize) {
        let bucket = self.bucket(self.nodes[index].key);
        let (previous, next) = match self.buckets[bucket] {
            Some(first) => (self.nodes[first].previous, Some(first)),
            None => (None, self.head),
        };
        self.nodes[index].previous = previous;
        self.nodes[index].next = next;
        if let Some(previous) = previous {
            self.nodes[previous].next = Some(index);
        } else {
            self.head = Some(index);
        }
        if let Some(next) = next {
            self.nodes[next].previous = Some(index);
        }
        self.buckets[bucket] = Some(index);
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (i32, f32)> + '_ {
        let mut next = self.head;
        std::iter::from_fn(move || {
            let node = &self.nodes[next?];
            next = node.next;
            Some((node.key, node.depth))
        })
    }
}
