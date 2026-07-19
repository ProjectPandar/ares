use std::ops::RangeInclusive;

use sha2::{Digest, Sha256};

#[derive(Clone, Copy)]
pub(super) struct Expected {
    pub(super) len: usize,
    pub(super) sha256: &'static str,
    pub(super) modes: [usize; 4],
    pub(super) contours: usize,
    pub(super) holes: usize,
    pub(super) points: usize,
}

struct LayerRecord {
    index: usize,
    mode: u8,
    start: usize,
    end: usize,
}

pub(super) struct Snapshot {
    layers: Vec<LayerRecord>,
}

pub(super) fn assert_checkpoint(bytes: &[u8], magic: &[u8; 8], expected: Expected) -> Snapshot {
    assert_eq!(bytes.len(), expected.len);
    assert_eq!(sha256(bytes), expected.sha256);
    parse(bytes, magic, expected)
}

pub(super) fn assert_changed_layers(
    before: (&[u8], &Snapshot),
    after: (&[u8], &Snapshot),
    expected: RangeInclusive<usize>,
    expected_sha256: &str,
) {
    let (before, before_snapshot) = before;
    let (after, after_snapshot) = after;
    assert_eq!(before_snapshot.layers.len(), after_snapshot.layers.len());
    let changed = before_snapshot
        .layers
        .iter()
        .zip(&after_snapshot.layers)
        .filter_map(|(left, right)| {
            assert_eq!((left.index, left.mode), (right.index, right.mode));
            (record(before, left) != record(after, right)).then_some(left.index)
        })
        .collect::<Vec<_>>();
    assert_eq!(changed, expected.collect::<Vec<_>>());
    let encoded = changed
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(sha256(encoded.as_bytes()), expected_sha256);
}

pub(super) fn assert_record(
    bytes: &[u8],
    snapshot: &Snapshot,
    slot: usize,
    expected_len: usize,
    expected_sha256: &str,
) {
    let layer = &snapshot.layers[slot];
    assert_eq!(layer.index, slot);
    let record = record(bytes, layer);
    assert_eq!(record.len(), expected_len);
    assert_eq!(sha256(record), expected_sha256);
}

pub(super) fn assert_body_equal_except_magic(
    before: &[u8],
    after: &[u8],
    before_stage: u8,
    after_stage: u8,
) {
    assert_eq!(before.len(), after.len());
    assert_eq!((before[6], after[6]), (before_stage, after_stage));
    assert_eq!(&before[..6], &after[..6]);
    assert_eq!(&before[7..], &after[7..]);
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn parse(bytes: &[u8], magic: &[u8; 8], expected: Expected) -> Snapshot {
    assert_eq!(bytes.get(..8), Some(magic.as_slice()));
    let mut cursor = 8;
    let objects = read_usize(bytes, &mut cursor);
    let mut volumes = 0;
    let mut layers = Vec::new();
    let mut modes = [0; 4];
    let mut contours = 0;
    let mut holes = 0;
    let mut points = 0;
    let mut identity = (0, 0, 0, 0, 0, 0);
    for object in 0..objects {
        let source_object = read_usize(bytes, &mut cursor);
        let transform = read_usize(bytes, &mut cursor);
        let planned_layers = read_usize(bytes, &mut cursor);
        let object_volumes = read_usize(bytes, &mut cursor);
        volumes += object_volumes;
        for volume in 0..object_volumes {
            let source_volume = read_usize(bytes, &mut cursor);
            let ordinal = read_u32(bytes, &mut cursor);
            let volume_type = read_u8(bytes, &mut cursor);
            let volume_layers = read_usize(bytes, &mut cursor);
            if object == 0 && volume == 0 {
                identity = (
                    source_object,
                    transform,
                    planned_layers,
                    source_volume,
                    ordinal,
                    volume_type,
                );
            }
            for _ in 0..volume_layers {
                parse_layer(
                    bytes,
                    &mut cursor,
                    &mut layers,
                    &mut modes,
                    &mut contours,
                    &mut holes,
                    &mut points,
                );
            }
        }
    }
    assert_eq!(cursor, bytes.len(), "checkpoint must end at exact EOF");
    assert_eq!((objects, volumes, layers.len()), (1, 1, 460));
    assert_eq!(identity, (0, 0, 460, 0, 1, 0));
    assert_eq!(modes, expected.modes);
    assert_eq!(contours, expected.contours);
    assert_eq!(holes, expected.holes);
    assert_eq!(points, expected.points);
    Snapshot { layers }
}

#[allow(clippy::too_many_arguments)]
fn parse_layer(
    bytes: &[u8],
    cursor: &mut usize,
    layers: &mut Vec<LayerRecord>,
    modes: &mut [usize; 4],
    contours: &mut usize,
    holes: &mut usize,
    points: &mut usize,
) {
    let start = *cursor;
    let index = read_usize(bytes, cursor);
    let mode = read_u8(bytes, cursor);
    modes[usize::from(mode)] += 1;
    let layer_contours = read_usize(bytes, cursor);
    *contours += layer_contours;
    for _ in 0..layer_contours {
        *points += read_polygon(bytes, cursor);
        let contour_holes = read_usize(bytes, cursor);
        *holes += contour_holes;
        for _ in 0..contour_holes {
            *points += read_polygon(bytes, cursor);
        }
    }
    layers.push(LayerRecord {
        index,
        mode,
        start,
        end: *cursor,
    });
}

fn read_polygon(bytes: &[u8], cursor: &mut usize) -> usize {
    let points = read_usize(bytes, cursor);
    *cursor = cursor.checked_add(points.checked_mul(16).unwrap()).unwrap();
    assert!(*cursor <= bytes.len());
    points
}

fn read_usize(bytes: &[u8], cursor: &mut usize) -> usize {
    usize::try_from(read_u64(bytes, cursor)).unwrap()
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> u64 {
    let end = cursor.checked_add(8).unwrap();
    let value = u64::from_le_bytes(bytes[*cursor..end].try_into().unwrap());
    *cursor = end;
    value
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> u32 {
    let end = cursor.checked_add(4).unwrap();
    let value = u32::from_le_bytes(bytes[*cursor..end].try_into().unwrap());
    *cursor = end;
    value
}

fn read_u8(bytes: &[u8], cursor: &mut usize) -> u8 {
    let value = bytes[*cursor];
    *cursor += 1;
    value
}

fn record<'a>(bytes: &'a [u8], layer: &LayerRecord) -> &'a [u8] {
    &bytes[layer.start..layer.end]
}
