use super::chain::EndPoint;

pub(super) struct MutablePriorityQueue {
    heap: Vec<usize>,
}

impl MutablePriorityQueue {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            heap: Vec::with_capacity(capacity),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub(super) fn top(&self) -> usize {
        self.heap[0]
    }

    pub(super) fn push(&mut self, item: usize, endpoints: &mut [EndPoint]) {
        let index = self.heap.len();
        self.heap.push(item);
        endpoints[item].heap_idx = index;
        self.update_heap_up(0, index, endpoints);
    }

    pub(super) fn pop(&mut self, endpoints: &mut [EndPoint]) -> usize {
        let item = self.heap[0];
        self.remove(0, endpoints);
        item
    }

    pub(super) fn remove(&mut self, index: usize, endpoints: &mut [EndPoint]) -> usize {
        let removed = self.heap[index];
        endpoints[removed].heap_idx = usize::MAX;
        if index + 1 == self.heap.len() {
            self.heap.pop();
            return removed;
        }
        let replacement = self.heap.pop().expect("queue is nonempty");
        self.heap[index] = replacement;
        endpoints[replacement].heap_idx = index;
        self.update_heap_down(index, self.heap.len() - 1, endpoints);
        self.update_heap_up(0, index, endpoints);
        removed
    }

    pub(super) fn update(&mut self, index: usize, endpoints: &mut [EndPoint]) {
        let item = self.remove(index, endpoints);
        self.push(item, endpoints);
    }

    fn update_heap_up(&mut self, top: usize, bottom: usize, endpoints: &mut [EndPoint]) {
        let mut child_index = bottom;
        loop {
            if child_index == 0 {
                break;
            }
            let parent_index = (child_index - 1) >> 1;
            if parent_index < top {
                break;
            }
            let parent = self.heap[parent_index];
            let child = self.heap[child_index];
            if endpoints[parent].distance_out >= endpoints[child].distance_out {
                self.heap.swap(parent_index, child_index);
                endpoints[parent].heap_idx = child_index;
                endpoints[child].heap_idx = parent_index;
            }
            child_index = parent_index;
        }
    }

    fn update_heap_down(&mut self, top: usize, bottom: usize, endpoints: &mut [EndPoint]) {
        let mut parent_index = top;
        loop {
            let mut child_index = (parent_index << 1) + 1;
            if child_index > bottom {
                break;
            }
            child_index = self.lower_child(child_index, bottom, endpoints);
            let parent = self.heap[parent_index];
            let child = self.heap[child_index];
            if endpoints[parent].distance_out < endpoints[child].distance_out {
                return;
            }
            self.heap.swap(parent_index, child_index);
            endpoints[parent].heap_idx = child_index;
            endpoints[child].heap_idx = parent_index;
            parent_index = child_index;
        }
    }

    fn lower_child(&self, left_index: usize, bottom: usize, endpoints: &[EndPoint]) -> usize {
        if left_index == bottom {
            return left_index;
        }
        let left = self.heap[left_index];
        let right = self.heap[left_index + 1];
        if endpoints[left].distance_out < endpoints[right].distance_out {
            left_index
        } else {
            left_index + 1
        }
    }
}
