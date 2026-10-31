// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.
use super::*;

impl ClipperBase {
    /// `clipper.engine.cpp:637–742,862–866`: preserve the input and minima walks.
    pub fn add_paths(&mut self, paths: &Paths64, polytype: PathType, is_open: bool) {
        self.has_open_paths |= is_open;
        self.minima_list_sorted = false;
        self.vertex_arena.reserve(paths.iter().map(Vec::len).sum());
        for path in paths {
            self.add_input_path(path, polytype, is_open);
        }
    }

    // AddPaths_:647–676, including the unadjusted count after closing-duplicate removal.
    fn add_input_path(&mut self, path: &Path64, polytype: PathType, is_open: bool) {
        let start = self.vertex_arena.len();
        let mut previous: Option<usize> = None;
        let mut count = 0;
        for &point in path {
            match previous {
                Some(previous) if self.vertex_arena[previous].pt == point => continue,
                Some(previous) => self.vertex_arena[previous].next = self.vertex_arena.len(),
                None => {}
            }
            let mut vertex = Vertex::new(point);
            vertex.prev = previous.unwrap_or(NONE);
            previous = Some(self.vertex_arena.len());
            self.vertex_arena.push(vertex);
            count += 1;
        }
        let Some(mut previous) = previous else {
            return;
        };
        if self.vertex_arena[previous].prev == NONE {
            self.vertex_arena.truncate(start);
            return;
        }
        if !is_open && self.vertex_arena[previous].pt == self.vertex_arena[start].pt {
            previous = self.vertex_arena[previous].prev;
        }
        self.vertex_arena[previous].next = start;
        self.vertex_arena[start].prev = previous;
        if count < 2 || (count == 2 && !is_open) {
            return;
        }
        let Some(direction) = self.initial_input_direction(start, polytype, is_open) else {
            return;
        };
        self.mark_input_extrema(start, direction, polytype, is_open);
    }

    // AddPaths_:678–702, first nonhorizontal direction and open-start flags.
    fn initial_input_direction(
        &mut self,
        start: usize,
        polytype: PathType,
        is_open: bool,
    ) -> Option<bool> {
        if is_open {
            let mut current = self.vertex_arena[start].next;
            while current != start
                && self.vertex_arena[current].pt.y == self.vertex_arena[start].pt.y
            {
                current = self.vertex_arena[current].next;
            }
            let going_up = self.vertex_arena[current].pt.y <= self.vertex_arena[start].pt.y;
            if going_up {
                self.vertex_arena[start].flags = VertexFlags::OPEN_START;
                self.add_loc_min(start, polytype, true);
            } else {
                self.vertex_arena[start].flags = VertexFlags::OPEN_START | VertexFlags::LOCAL_MAX;
            }
            return Some(going_up);
        }
        let mut previous = self.vertex_arena[start].prev;
        while previous != start && self.vertex_arena[previous].pt.y == self.vertex_arena[start].pt.y
        {
            previous = self.vertex_arena[previous].prev;
        }
        if previous == start {
            return None;
        }
        Some(self.vertex_arena[previous].pt.y > self.vertex_arena[start].pt.y)
    }

    // AddPaths_:704–742, same linked walk and final open/closed extremum.
    fn mark_input_extrema(
        &mut self,
        start: usize,
        mut going_up: bool,
        polytype: PathType,
        is_open: bool,
    ) {
        let original_direction = going_up;
        let mut previous = start;
        let mut current = self.vertex_arena[start].next;
        while current != start {
            if self.vertex_arena[current].pt.y > self.vertex_arena[previous].pt.y && going_up {
                self.vertex_arena[previous].flags |= VertexFlags::LOCAL_MAX;
                going_up = false;
            } else if self.vertex_arena[current].pt.y < self.vertex_arena[previous].pt.y
                && !going_up
            {
                going_up = true;
                self.add_loc_min(previous, polytype, is_open);
            }
            previous = current;
            current = self.vertex_arena[current].next;
        }
        if is_open {
            self.vertex_arena[previous].flags |= VertexFlags::OPEN_END;
            if going_up {
                self.vertex_arena[previous].flags |= VertexFlags::LOCAL_MAX;
            } else {
                self.add_loc_min(previous, polytype, true);
            }
        } else if going_up != original_direction {
            if original_direction {
                self.add_loc_min(previous, polytype, false);
            } else {
                self.vertex_arena[previous].flags |= VertexFlags::LOCAL_MAX;
            }
        }
    }

    // clipper.engine.cpp:75–83,812–818: stable source comparator, no added keys.
    pub fn sort_minima_list(&mut self) {
        if self.minima_list_sorted {
            return;
        }
        let vertices = &self.vertex_arena;
        self.minima_list.sort_by(|a, b| {
            let a = vertices[a.vertex].pt;
            let b = vertices[b.vertex].pt;
            if a.y != b.y {
                b.y.cmp(&a.y)
            } else {
                a.x.cmp(&b.x)
            }
        });
        self.minima_list_sorted = true;
    }
}
