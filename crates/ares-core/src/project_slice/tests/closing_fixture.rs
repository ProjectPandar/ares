use crate::{
    ProjectVolumeType,
    geometry::{ExPolygon, Point, Polygon},
    mesh_slicer::SlicingMode,
};

use super::super::{
    closing::{PostClosingLayer, PostClosingPrintObject, PostClosingVolume},
    layers::{PlannedLayer, PlannedPrintObject},
    task22g_oracle::encode,
};

type PointPair = (i64, i64);
type PolygonRef<'a> = &'a [PointPair];
type ExPolygonRef<'a> = (PolygonRef<'a>, &'a [PolygonRef<'a>]);

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    objects: usize,
    volumes: usize,
    layers: usize,
    contours: usize,
    holes: usize,
    points: usize,
    source_object: usize,
    transform: usize,
    planned_layers: usize,
    source_volume: usize,
    ordinal: u32,
    volume_type: u8,
    layer_records: Vec<(usize, usize, usize)>,
}

#[test]
fn task22g_parser_accepts_independent_nested_empty_vector() {
    let mut bytes = b"ARES22G\0".to_vec();
    push_u64(&mut bytes, 1);
    push_u64(&mut bytes, 7);
    push_u64(&mut bytes, 9);
    push_u64(&mut bytes, 2);
    push_u64(&mut bytes, 1);
    push_u64(&mut bytes, 11);
    bytes.extend_from_slice(&3_u32.to_le_bytes());
    bytes.push(2);
    push_u64(&mut bytes, 2);
    push_layer(&mut bytes, 0, 0, &[]);
    push_layer(
        &mut bytes,
        1,
        1,
        &[(
            &[(40, 40), (0, 40), (0, 0), (40, 0)],
            &[&[(10, 10), (10, 30), (30, 30), (30, 10)]],
        )],
    );

    assert_eq!(
        parse(&bytes),
        Snapshot {
            objects: 1,
            volumes: 1,
            layers: 2,
            contours: 1,
            holes: 1,
            points: 8,
            source_object: 7,
            transform: 9,
            planned_layers: 2,
            source_volume: 11,
            ordinal: 3,
            volume_type: 2,
            layer_records: vec![(0, 69, 86), (1, 86, bytes.len())],
        }
    );
}

#[test]
fn task22g_canonical_encoder_matches_handwritten_nested_empty_vector() {
    let nested = ExPolygon::new(
        owned_polygon(&[(40, 40), (0, 40), (0, 0), (40, 0)]),
        vec![owned_polygon(&[(10, 10), (10, 30), (30, 30), (30, 10)])],
    );
    let object = PostClosingPrintObject::new(
        PlannedPrintObject {
            source_object_index: 7,
            transform_index: 9,
            layers: vec![planned_layer(0), planned_layer(1)],
        },
        vec![PostClosingVolume::new(
            11,
            3,
            ProjectVolumeType::ParameterModifier,
            vec![
                PostClosingLayer::new(SlicingMode::Regular, Vec::new()),
                PostClosingLayer::new(SlicingMode::EvenOdd, vec![nested]),
            ],
        )],
    );

    let encoded = encode(&[object]);
    let mut expected = b"ARES22G\0".to_vec();
    for value in [1, 7, 9, 2, 1, 11] {
        push_u64(&mut expected, value);
    }
    expected.extend_from_slice(&3_u32.to_le_bytes());
    expected.push(2);
    push_u64(&mut expected, 2);
    push_layer(&mut expected, 0, 0, &[]);
    push_layer(
        &mut expected,
        1,
        1,
        &[(
            &[(40, 40), (0, 40), (0, 0), (40, 0)],
            &[&[(10, 10), (10, 30), (30, 30), (30, 10)]],
        )],
    );

    assert_eq!(encoded.len(), 255);
    assert_eq!(encoded, expected);
}

fn parse(bytes: &[u8]) -> Snapshot {
    assert!(bytes.starts_with(b"ARES22G\0"));
    let mut cursor = 8;
    let objects = read_usize(bytes, &mut cursor);
    let mut snapshot = Snapshot {
        objects,
        volumes: 0,
        layers: 0,
        contours: 0,
        holes: 0,
        points: 0,
        source_object: 0,
        transform: 0,
        planned_layers: 0,
        source_volume: 0,
        ordinal: 0,
        volume_type: 0,
        layer_records: Vec::new(),
    };
    for object in 0..objects {
        let source_object = read_usize(bytes, &mut cursor);
        let transform = read_usize(bytes, &mut cursor);
        let planned_layers = read_usize(bytes, &mut cursor);
        let volumes = read_usize(bytes, &mut cursor);
        if object == 0 {
            snapshot.source_object = source_object;
            snapshot.transform = transform;
            snapshot.planned_layers = planned_layers;
        }
        snapshot.volumes += volumes;
        for volume in 0..volumes {
            let source_volume = read_usize(bytes, &mut cursor);
            let ordinal = read_u32(bytes, &mut cursor);
            let volume_type = read_u8(bytes, &mut cursor);
            let layers = read_usize(bytes, &mut cursor);
            if object == 0 && volume == 0 {
                snapshot.source_volume = source_volume;
                snapshot.ordinal = ordinal;
                snapshot.volume_type = volume_type;
            }
            snapshot.layers += layers;
            for _ in 0..layers {
                parse_layer(bytes, &mut cursor, &mut snapshot);
            }
        }
    }
    assert_eq!(
        cursor,
        bytes.len(),
        "canonical stream must end at exact EOF"
    );
    snapshot
}

fn parse_layer(bytes: &[u8], cursor: &mut usize, snapshot: &mut Snapshot) {
    let start = *cursor;
    let layer = read_usize(bytes, cursor);
    let _mode = read_u8(bytes, cursor);
    let contours = read_usize(bytes, cursor);
    snapshot.contours += contours;
    for _ in 0..contours {
        snapshot.points += read_polygon(bytes, cursor);
        let holes = read_usize(bytes, cursor);
        snapshot.holes += holes;
        for _ in 0..holes {
            snapshot.points += read_polygon(bytes, cursor);
        }
    }
    snapshot.layer_records.push((layer, start, *cursor));
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

fn push_layer(bytes: &mut Vec<u8>, index: usize, mode: u8, expolygons: &[ExPolygonRef<'_>]) {
    push_u64(bytes, index);
    bytes.push(mode);
    push_u64(bytes, expolygons.len());
    for &(contour, holes) in expolygons {
        push_polygon(bytes, contour);
        push_u64(bytes, holes.len());
        for &hole in holes {
            push_polygon(bytes, hole);
        }
    }
}

fn push_polygon(bytes: &mut Vec<u8>, points: &[PointPair]) {
    push_u64(bytes, points.len());
    for &(x, y) in points {
        bytes.extend_from_slice(&x.to_le_bytes());
        bytes.extend_from_slice(&y.to_le_bytes());
    }
}

fn push_u64(bytes: &mut Vec<u8>, value: usize) {
    bytes.extend_from_slice(&u64::try_from(value).unwrap().to_le_bytes());
}

fn planned_layer(id: usize) -> PlannedLayer {
    PlannedLayer {
        id,
        height: 0.2,
        print_z: (id + 1) as f64 * 0.2,
        slice_z: (id as f64 + 0.5) * 0.2,
    }
}

fn owned_polygon(points: &[PointPair]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}
