use super::types::EndPoint;

pub(super) struct Queue {
    heap: Vec<usize>,
}

impl Queue {
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

    pub(super) fn push(&mut self, endpoint: usize, endpoints: &mut [EndPoint]) {
        let index = self.heap.len();
        self.heap.push(endpoint);
        endpoints[endpoint].heap_index = index;
        self.update_up(0, index, endpoints);
    }

    pub(super) fn pop(&mut self, endpoints: &mut [EndPoint]) -> usize {
        self.remove(0, endpoints)
    }

    pub(super) fn remove(&mut self, index: usize, endpoints: &mut [EndPoint]) -> usize {
        let removed = self.heap[index];
        endpoints[removed].heap_index = usize::MAX;
        if index + 1 == self.heap.len() {
            self.heap.pop();
            return removed;
        }
        let replacement = self.heap.pop().expect("queue is nonempty");
        self.heap[index] = replacement;
        endpoints[replacement].heap_index = index;
        self.update_down(index, self.heap.len() - 1, endpoints);
        self.update_up(0, index, endpoints);
        removed
    }

    pub(super) fn update(&mut self, index: usize, endpoints: &mut [EndPoint]) {
        let endpoint = self.remove(index, endpoints);
        self.push(endpoint, endpoints);
    }

    fn update_up(&mut self, top: usize, bottom: usize, endpoints: &mut [EndPoint]) {
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
            if endpoints[parent].distance >= endpoints[child].distance {
                self.heap.swap(parent_index, child_index);
                endpoints[parent].heap_index = child_index;
                endpoints[child].heap_index = parent_index;
            }
            child_index = parent_index;
        }
    }

    fn update_down(&mut self, top: usize, bottom: usize, endpoints: &mut [EndPoint]) {
        let mut parent_index = top;
        loop {
            let mut child_index = (parent_index << 1) + 1;
            if child_index > bottom {
                break;
            }
            child_index = self.lower_child(child_index, bottom, endpoints);
            let parent = self.heap[parent_index];
            let child = self.heap[child_index];
            if endpoints[parent].distance < endpoints[child].distance {
                return;
            }
            self.heap.swap(parent_index, child_index);
            endpoints[parent].heap_index = child_index;
            endpoints[child].heap_index = parent_index;
            parent_index = child_index;
        }
    }

    fn lower_child(&self, left_index: usize, bottom: usize, endpoints: &[EndPoint]) -> usize {
        if left_index == bottom {
            return left_index;
        }
        let left = self.heap[left_index];
        let right = self.heap[left_index + 1];
        if endpoints[left].distance < endpoints[right].distance {
            left_index
        } else {
            left_index + 1
        }
    }
}
