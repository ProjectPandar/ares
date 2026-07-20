use super::EndPoint;

pub(in crate::geometry) struct EndPointHeap {
    heap: Vec<usize>,
}

impl EndPointHeap {
    pub(in crate::geometry) fn with_capacity(capacity: usize) -> Self {
        Self {
            heap: Vec::with_capacity(capacity),
        }
    }

    pub(in crate::geometry) fn top(&self) -> usize {
        self.heap[0]
    }

    pub(in crate::geometry) fn push(&mut self, point: usize, points: &mut [EndPoint]) {
        let index = self.heap.len();
        self.heap.push(point);
        points[point].heap_index = index;
        self.update_up(0, index, points);
    }

    pub(in crate::geometry) fn pop(&mut self, points: &mut [EndPoint]) {
        self.remove(0, points);
    }

    pub(in crate::geometry) fn remove(&mut self, index: usize, points: &mut [EndPoint]) {
        if index + 1 == self.heap.len() {
            self.heap.pop();
            return;
        }

        self.heap[index] = self.heap.pop().unwrap();
        points[self.heap[index]].heap_index = index;
        self.update_down(index, self.heap.len() - 1, points);
        self.update_up(0, index, points);
    }

    pub(in crate::geometry) fn update(&mut self, index: usize, points: &mut [EndPoint]) {
        let point = self.heap[index];
        self.remove(index, points);
        self.push(point, points);
    }

    #[cfg(test)]
    pub(in crate::geometry) fn heap(&self) -> &[usize] {
        &self.heap
    }

    fn update_up(&mut self, top: usize, bottom: usize, points: &mut [EndPoint]) {
        let mut child = bottom;
        loop {
            if child == 0 {
                break;
            }
            let parent = (child - 1) >> 1;
            if parent < top {
                break;
            }
            if !Self::less(self.heap[parent], self.heap[child], points) {
                self.swap_nodes(parent, child, points);
            }
            child = parent;
        }
    }

    fn update_down(&mut self, top: usize, bottom: usize, points: &mut [EndPoint]) {
        let mut parent = top;
        loop {
            let mut child = (parent << 1) + 1;
            if child > bottom {
                break;
            }
            if child < bottom && !Self::less(self.heap[child], self.heap[child + 1], points) {
                child += 1;
            }
            if Self::less(self.heap[parent], self.heap[child], points) {
                return;
            }
            self.swap_nodes(parent, child, points);
            parent = child;
        }
    }

    fn swap_nodes(&mut self, left: usize, right: usize, points: &mut [EndPoint]) {
        self.heap.swap(left, right);
        points[self.heap[left]].heap_index = left;
        points[self.heap[right]].heap_index = right;
    }

    fn less(left: usize, right: usize, points: &[EndPoint]) -> bool {
        points[left].distance_out < points[right].distance_out
    }
}
