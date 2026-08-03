use sha2::{Digest, Sha256};

use crate::{SliceError, slice_project, task22h_browser_input_oracle, task22h_browser_oracle};

use super::support::{KsrArchive, ksr_project, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";
const SPIRAL_OFF: &str = r#""spiral_mode": "0""#;
const SPIRAL_ON: &str = r#""spiral_mode": "1""#;
const BOTTOM_LAYERS_THREE: &str = r#""bottom_shell_layers": "3""#;
const BOTTOM_LAYERS_ZERO: &str = r#""bottom_shell_layers": "0""#;
const BOTTOM_THICKNESS_ZERO: &str = r#""bottom_shell_thickness": "0""#;
const BOTTOM_THICKNESS_VECTOR: &str = r#""bottom_shell_thickness": "0.5001""#;
const FIXTURE_SHA256: &str = "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9";

#[derive(Clone, Copy)]
struct Expected {
    len: usize,
    sha256: &'static str,
    modes: [usize; 4],
    contours: usize,
    holes: usize,
    points: usize,
}

const BASE_G: Expected = Expected {
    len: 1_644_681,
    sha256: "29ffb501c54190dd4336cc1371fc5e480c5b87ac6a8184366bd072bf5cb90919",
    modes: [460, 0, 0, 0],
    contours: 2_890,
    holes: 395,
    points: 99_212,
};
const BASE_H: Expected = Expected {
    sha256: "e15967c36c0aa47a9a1a3fc31053587777359bedef796053022eaeb36ad49163",
    ..BASE_G
};

struct LayerRecord {
    index: usize,
    start: usize,
    end: usize,
}

struct Snapshot {
    objects: usize,
    volumes: usize,
    layers: Vec<LayerRecord>,
    modes: [usize; 4],
    contours: usize,
    holes: usize,
    points: usize,
    identity: (usize, usize, usize, usize, u32, u8),
}

#[test]
fn task22h_committed_project_is_exact_marker_only_identity() {
    assert_eq!(sha256(ksr_project()), FIXTURE_SHA256);
    let (g, h) = repeatable_checkpoints(ksr_project());
    let _g_snapshot = assert_checkpoint(&g, b"ARES22G\0", BASE_G);
    let h_snapshot = assert_checkpoint(&h, b"ARES22H\0", BASE_H);

    let mut marker_only = g;
    marker_only[6] = b'H';
    assert_eq!(h, marker_only);
    for (slot, len, digest) in [
        (
            0,
            14_913,
            "28fbbcc66d73c037a5dbb3c60363d83bfaeaaf1d9d8a49451594f227ea0d4fcf",
        ),
        (
            46,
            46_233,
            "8dba7c5e51c74e803903b513c5165dffb9d1c55be108e39fbccca4309a603e69",
        ),
        (
            459,
            737,
            "c8822b67958531cb4b043d338b53f7329e0b00cb4f08108306763e763cd52f80",
        ),
    ] {
        assert_record(&h, &h_snapshot, slot, len, digest);
    }
}

#[tokio::test]
async fn task22h_public_global_spiral_capability_precedes_largest_contour() {
    assert_eq!(
        slice_project(primary_mutation(), metadata()).await,
        Err(SliceError::UnsupportedProjectFeature(
            "spiral_mode".to_owned()
        ))
    );
}

fn primary_mutation() -> Vec<u8> {
    let mut archive = KsrArchive::new();
    archive.replace_unique(PROJECT_SETTINGS, SPIRAL_OFF, SPIRAL_ON);
    archive.replace_unique(PROJECT_SETTINGS, BOTTOM_LAYERS_THREE, BOTTOM_LAYERS_ZERO);
    archive.replace_unique(
        PROJECT_SETTINGS,
        BOTTOM_THICKNESS_ZERO,
        BOTTOM_THICKNESS_VECTOR,
    );
    archive.bytes()
}

fn repeatable_checkpoints(project: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let g = task22h_browser_input_oracle(project).unwrap();
    let h = task22h_browser_oracle(project).unwrap();
    assert_eq!(task22h_browser_input_oracle(project).unwrap(), g);
    assert_eq!(task22h_browser_oracle(project).unwrap(), h);
    (g, h)
}

fn assert_checkpoint(bytes: &[u8], magic: &[u8; 8], expected: Expected) -> Snapshot {
    assert_eq!(bytes.len(), expected.len);
    assert_eq!(sha256(bytes), expected.sha256);
    let snapshot = parse(bytes, magic);
    assert_eq!(snapshot.objects, 1);
    assert_eq!(snapshot.volumes, 1);
    assert_eq!(snapshot.layers.len(), 460);
    assert_eq!(snapshot.modes, expected.modes);
    assert_eq!(snapshot.contours, expected.contours);
    assert_eq!(snapshot.holes, expected.holes);
    assert_eq!(snapshot.points, expected.points);
    assert_eq!(snapshot.identity, (0, 0, 460, 0, 1, 0));
    snapshot
}

fn parse(bytes: &[u8], magic: &[u8; 8]) -> Snapshot {
    assert_eq!(bytes.get(..8), Some(magic.as_slice()));
    let mut cursor = 8;
    let objects = read_usize(bytes, &mut cursor);
    let mut snapshot = Snapshot {
        objects,
        volumes: 0,
        layers: Vec::new(),
        modes: [0; 4],
        contours: 0,
        holes: 0,
        points: 0,
        identity: (0, 0, 0, 0, 0, 0),
    };
    for object in 0..objects {
        let source_object = read_usize(bytes, &mut cursor);
        let transform = read_usize(bytes, &mut cursor);
        let planned_layers = read_usize(bytes, &mut cursor);
        let volumes = read_usize(bytes, &mut cursor);
        snapshot.volumes += volumes;
        for volume in 0..volumes {
            let source_volume = read_usize(bytes, &mut cursor);
            let ordinal = read_u32(bytes, &mut cursor);
            let volume_type = read_u8(bytes, &mut cursor);
            let layers = read_usize(bytes, &mut cursor);
            if object == 0 && volume == 0 {
                snapshot.identity = (
                    source_object,
                    transform,
                    planned_layers,
                    source_volume,
                    ordinal,
                    volume_type,
                );
            }
            for _ in 0..layers {
                parse_layer(bytes, &mut cursor, &mut snapshot);
            }
        }
    }
    assert_eq!(cursor, bytes.len(), "checkpoint must end at exact EOF");
    snapshot
}

fn parse_layer(bytes: &[u8], cursor: &mut usize, snapshot: &mut Snapshot) {
    let start = *cursor;
    let index = read_usize(bytes, cursor);
    let mode = read_u8(bytes, cursor);
    snapshot.modes[usize::from(mode)] += 1;
    let expolygons = read_usize(bytes, cursor);
    snapshot.contours += expolygons;
    for _ in 0..expolygons {
        snapshot.points += read_polygon(bytes, cursor);
        let holes = read_usize(bytes, cursor);
        snapshot.holes += holes;
        for _ in 0..holes {
            snapshot.points += read_polygon(bytes, cursor);
        }
    }
    snapshot.layers.push(LayerRecord {
        index,
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

fn record<'a>(bytes: &'a [u8], snapshot: &Snapshot, slot: usize) -> &'a [u8] {
    let layer = &snapshot.layers[slot];
    assert_eq!(layer.index, slot);
    &bytes[layer.start..layer.end]
}

fn assert_record(bytes: &[u8], snapshot: &Snapshot, slot: usize, len: usize, digest: &str) {
    let record = record(bytes, snapshot, slot);
    assert_eq!(record.len(), len);
    assert_eq!(sha256(record), digest);
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
