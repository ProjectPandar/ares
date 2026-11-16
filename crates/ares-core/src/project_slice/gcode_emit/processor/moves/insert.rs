//! Back-to-front actual-speed insertion (`GCodeProcessor.cpp:8141-8174`).

pub(in crate::project_slice::gcode_emit::processor) fn insert<T: Copy>(
    moves: &mut Vec<T>,
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
        while read > id {
            read -= 1;
            write -= 1;
            moves[write] = moves[read];
        }
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
