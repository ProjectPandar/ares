//! Back-to-front actual-speed insertion (`GCodeProcessor.cpp:8141-8174`).

pub(in crate::project_slice::gcode_emit::processor) trait Buffer<T>:
    std::ops::IndexMut<usize, Output = T>
{
    fn len(&self) -> usize;
    fn resize(&mut self, len: usize, value: T);
    fn copy_backwards(&mut self, range: std::ops::Range<usize>, destination: usize);
}

impl<T: Copy> Buffer<T> for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
    fn resize(&mut self, len: usize, value: T) {
        self.resize(len, value);
    }
    fn copy_backwards(&mut self, range: std::ops::Range<usize>, destination: usize) {
        self.copy_within(range, destination);
    }
}

impl Buffer<crate::gcode_processor::MoveVertex> for crate::gcode_processor::ChunkedMoves {
    fn len(&self) -> usize {
        self.len()
    }
    fn resize(&mut self, len: usize, value: crate::gcode_processor::MoveVertex) {
        self.resize(len, value);
    }
    fn copy_backwards(&mut self, range: std::ops::Range<usize>, destination: usize) {
        self.copy_backwards(range, destination);
    }
}

pub(in crate::project_slice::gcode_emit::processor) fn insert<T: Copy>(
    moves: &mut impl Buffer<T>,
    groups: impl DoubleEndedIterator<Item = (usize, usize)> + ExactSizeIterator,
    splits: &mut Vec<T>,
) -> (Vec<(usize, usize)>, usize) {
    let inserted = splits.len();
    let mut read = moves.len();
    moves.resize(read + inserted, moves[0]);
    let mut write = moves.len();
    let mut split = splits.len();
    let mut map = Vec::with_capacity(groups.len());
    for (id, count) in groups.rev() {
        write -= read - id;
        moves.copy_backwards(id..read, write);
        read = id;
        map.push((id, write));
        for _ in 0..count {
            write -= 1;
            split -= 1;
            moves[write] = splits[split];
        }
    }
    debug_assert_eq!(read, write);
    debug_assert_eq!(split, 0);
    splits.clear();
    map.reverse();
    (map, inserted)
}
