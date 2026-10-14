//! glu-libtess priorityq.c:92–261; priorityq-heap.c:96–256.
//! Copyright (C) 1991-2000 Silicon Graphics, Inc. All Rights Reserved.
//! SGI FREE SOFTWARE LICENSE B, Version 2.0:
//! http://oss.sgi.com/projects/FreeB/
use super::{mesh::Mesh, predicates::leq};
#[derive(Default, Clone, Copy)]
struct Handle {
    key: usize,
    node: usize,
}
#[derive(Default)]
pub(super) struct Queue {
    keys: Vec<usize>,
    order: Vec<usize>,
    size: usize,
    nodes: Vec<usize>,
    handles: Vec<Handle>,
    heap_size: usize,
    free: usize,
}
struct PqSortState {
    seed: u64,
    stack: Vec<(isize, isize)>,
}
impl Queue {
    pub fn new(mesh: &mut Mesh) -> Self {
        let mut keys = Vec::new();
        let mut v = mesh.v(super::mesh::V_HEAD).next;
        while v != super::mesh::V_HEAD {
            mesh.vertices[v].pq_handle = -(keys.len() as isize + 1);
            keys.push(v);
            v = mesh.v(v).next;
        }
        let mut queue = Self {
            order: (0..keys.len()).collect(),
            size: keys.len(),
            keys,
            nodes: vec![0; 33],
            handles: vec![Handle::default(); 33],
            heap_size: 0,
            free: 0,
        };
        queue.nodes[1] = 1;
        queue.sort(mesh);
        queue
    }
    fn sort(&mut self, mesh: &Mesh) {
        // Preserve the source LP64 quicksort seed, equal-key swaps and insertion cutoff.
        let mut state = PqSortState {
            seed: 2016473283u64,
            stack: vec![(0isize, self.size as isize - 1)],
        };
        while let Some((p, r)) = state.stack.pop() {
            self.pq_sort_range(mesh, p, r, &mut state);
        }
    }
    fn pq_sort_range(&mut self, mesh: &Mesh, mut p: isize, mut r: isize, state: &mut PqSortState) {
        while r > p + 10 {
            state.seed = state.seed.wrapping_mul(1539415821).wrapping_add(1);
            let pivot_index = p + (state.seed % ((r - p + 1) as u64)) as isize;
            let pivot = self.order[pivot_index as usize];
            self.order.swap(pivot_index as usize, p as usize);
            let (i, j) = self.pq_partition(mesh, (p, r), pivot);
            if i - p < r - j {
                state.stack.push((j + 1, r));
                r = i - 1;
            } else {
                state.stack.push((p, i - 1));
                p = j + 1;
            }
        }
        self.pq_insertion_sort(mesh, p, r);
    }
    fn pq_insertion_sort(&mut self, mesh: &Mesh, p: isize, r: isize) {
        let mut i = p + 1;
        while i <= r {
            let pivot = self.order[i as usize];
            let mut j = i;
            while j > p
                && !leq(
                    mesh.v(self.keys[pivot]),
                    mesh.v(self.keys[self.order[(j - 1) as usize]]),
                )
            {
                self.order[j as usize] = self.order[(j - 1) as usize];
                j -= 1;
            }
            self.order[j as usize] = pivot;
            i += 1;
        }
    }
    fn pq_partition(
        &mut self,
        mesh: &Mesh,
        (p, r): (isize, isize),
        pivot: usize,
    ) -> (isize, isize) {
        let (mut i, mut j) = (p - 1, r + 1);
        loop {
            i = self.pq_scan_forward(mesh, i, pivot);
            j = self.pq_scan_backward(mesh, j, pivot);
            self.order.swap(i as usize, j as usize);
            if i >= j {
                break;
            }
        }
        self.order.swap(i as usize, j as usize);
        (i, j)
    }
    fn pq_scan_forward(&self, mesh: &Mesh, mut i: isize, pivot: usize) -> isize {
        loop {
            i += 1;
            if leq(
                mesh.v(self.keys[self.order[i as usize]]),
                mesh.v(self.keys[pivot]),
            ) {
                return i;
            }
        }
    }
    fn pq_scan_backward(&self, mesh: &Mesh, mut j: isize, pivot: usize) -> isize {
        loop {
            j -= 1;
            if leq(
                mesh.v(self.keys[pivot]),
                mesh.v(self.keys[self.order[j as usize]]),
            ) {
                return j;
            }
        }
    }
    fn heap_leq(&self, mesh: &Mesh, a: usize, b: usize) -> bool {
        leq(mesh.v(self.handles[a].key), mesh.v(self.handles[b].key))
    }
    fn down(&mut self, mesh: &Mesh, mut current: usize) {
        let handle = self.nodes[current];
        loop {
            let mut child = current * 2;
            if child < self.heap_size
                && self.heap_leq(mesh, self.nodes[child + 1], self.nodes[child])
            {
                child += 1;
            }
            if child > self.heap_size || self.heap_leq(mesh, handle, self.nodes[child]) {
                self.nodes[current] = handle;
                self.handles[handle].node = current;
                break;
            }
            let next = self.nodes[child];
            self.nodes[current] = next;
            self.handles[next].node = current;
            current = child;
        }
    }
    fn up(&mut self, mesh: &Mesh, mut current: usize) {
        let handle = self.nodes[current];
        loop {
            let parent = current / 2;
            if parent == 0 || self.heap_leq(mesh, self.nodes[parent], handle) {
                self.nodes[current] = handle;
                self.handles[handle].node = current;
                break;
            }
            let next = self.nodes[parent];
            self.nodes[current] = next;
            self.handles[next].node = current;
            current = parent;
        }
    }
    pub fn insert(&mut self, mesh: &Mesh, key: usize) -> isize {
        self.heap_size += 1;
        let current = self.heap_size;
        if current * 2 >= self.nodes.len() {
            self.nodes.resize(self.nodes.len() * 2 - 1, 0);
            self.handles.resize(self.nodes.len(), Handle::default());
        }
        let handle = if self.free == 0 {
            current
        } else {
            let h = self.free;
            self.free = self.handles[h].node;
            h
        };
        self.nodes[current] = handle;
        self.handles[handle] = Handle { key, node: current };
        self.up(mesh, current);
        handle as isize
    }
    fn heap_min(&self) -> usize {
        self.handles[self.nodes[1]].key
    }
    fn extract_heap(&mut self, mesh: &Mesh) -> usize {
        let handle = self.nodes[1];
        let key = self.handles[handle].key;
        if self.heap_size > 0 {
            self.nodes[1] = self.nodes[self.heap_size];
            self.handles[self.nodes[1]].node = 1;
            self.handles[handle] = Handle {
                key: 0,
                node: self.free,
            };
            self.free = handle;
            self.heap_size -= 1;
            if self.heap_size > 0 {
                self.down(mesh, 1);
            }
        }
        key
    }
    pub fn minimum(&self, mesh: &Mesh) -> usize {
        if self.size == 0 {
            return self.heap_min();
        }
        let sorted = self.keys[self.order[self.size - 1]];
        if self.heap_size > 0 && leq(mesh.v(self.heap_min()), mesh.v(sorted)) {
            self.heap_min()
        } else {
            sorted
        }
    }
    pub fn extract(&mut self, mesh: &Mesh) -> usize {
        if self.size == 0 {
            return self.extract_heap(mesh);
        }
        let sorted = self.keys[self.order[self.size - 1]];
        if self.heap_size > 0 && leq(mesh.v(self.heap_min()), mesh.v(sorted)) {
            return self.extract_heap(mesh);
        }
        self.size -= 1;
        self.trim();
        sorted
    }
    fn trim(&mut self) {
        while self.size > 0 && self.keys[self.order[self.size - 1]] == 0 {
            self.size -= 1;
        }
    }
    pub fn delete(&mut self, mesh: &Mesh, handle: isize) {
        if handle < 0 {
            self.keys[(-(handle + 1)) as usize] = 0;
            self.trim();
            return;
        }
        let h = handle as usize;
        let current = self.handles[h].node;
        self.nodes[current] = self.nodes[self.heap_size];
        self.handles[self.nodes[current]].node = current;
        self.heap_size -= 1;
        if current <= self.heap_size {
            if current <= 1 || self.heap_leq(mesh, self.nodes[current / 2], self.nodes[current]) {
                self.down(mesh, current);
            } else {
                self.up(mesh, current);
            }
        }
        self.handles[h] = Handle {
            key: 0,
            node: self.free,
        };
        self.free = h;
    }
}
