use plugin_spike_api::{Edit, Layer, Path, Point, ResolvedOption, Role, Value};

pub const PATHS: usize = 2_000;
pub const POINTS: usize = 200;
pub const LAYERS: u32 = 300;
pub const TRIALS: usize = 5;
pub const LAYER_HEIGHT: f64 = 0.25;
pub const GCODE: &[u8] = b"; plugin specimen\nG1 X0.125 Y1.25 E0.03125\n";
const HEADER: usize = 32;
const LAYER_INDEX: usize = 4;
const PATH_COUNT: usize = 8;
const POINT_COUNT: usize = 12;
const Z_BOTTOM: usize = 16;
const HEIGHT: usize = 24;
const METADATA: usize = 16;
const POINT_BYTES: usize = 16;
const EDIT_HEADER: usize = 4;
const EDIT: usize = 28;
const EDIT_Z: usize = 4;
const EDIT_HEIGHT: usize = 12;
const EDIT_FLOW: usize = 20;

// Little-endian v1: version, layer, path and point counts (u32), bottom Z
// and height (f64), then per-path inset, role, first point and point count
// (u32), then XY (f64 pairs). The option trailer uses length-prefixed keys
// and tagged values. Edits: count:u32, then [sourcePath:u32, z:f64,
// height:f64, flowMultiplier:f64] in print order. G-code has no framing.
pub fn encode_layer(layer: Layer<'_>) -> Vec<u8> {
    let points: usize = layer.paths.iter().map(|path| path.points.len()).sum();
    let mut bytes =
        Vec::with_capacity(HEADER + METADATA * layer.paths.len() + POINT_BYTES * points + 64);
    put_u32(&mut bytes, 1);
    put_u32(&mut bytes, layer.index as usize);
    put_u32(&mut bytes, layer.paths.len());
    put_u32(&mut bytes, points);
    put_float(&mut bytes, layer.z_bottom);
    put_float(&mut bytes, layer.height);
    let mut offset = 0;
    for path in layer.paths {
        put_u32(&mut bytes, path.inset_index as usize);
        put_u32(
            &mut bytes,
            usize::from(matches!(path.role, Role::InnerWall)),
        );
        put_u32(&mut bytes, offset);
        put_u32(&mut bytes, path.points.len());
        offset += path.points.len();
    }
    for path in layer.paths {
        bytes.extend_from_slice(bytemuck::cast_slice(path.points));
    }
    put_u32(&mut bytes, layer.options.len());
    for option in layer.options {
        put_text(&mut bytes, option.key);
        match option.value {
            Value::Boolean(value) => bytes.extend_from_slice(&[0, u8::from(value)]),
            Value::Number(value) => {
                bytes.push(1);
                put_float(&mut bytes, value);
            }
            Value::Choice(value) => {
                bytes.push(2);
                put_text(&mut bytes, value);
            }
        }
    }
    bytes
}

pub fn with_layer<T>(bytes: &[u8], run: impl FnOnce(Layer<'_>) -> T) -> T {
    let path_count = read_u32(bytes, PATH_COUNT) as usize;
    let points_start = HEADER + METADATA * path_count;
    let points_end = points_start + POINT_BYTES * read_u32(bytes, POINT_COUNT) as usize;
    let raw_points = &bytes[points_start..points_end];
    let copied;
    let points: &[Point] = if let Ok(points) = bytemuck::try_cast_slice(raw_points) {
        points
    } else {
        copied = raw_points
            .as_chunks::<POINT_BYTES>()
            .0
            .iter()
            .map(|xy| [read_f64(xy, 0), read_f64(xy, 8)])
            .collect::<Vec<_>>();
        &copied
    };
    let paths = bytes[HEADER..points_start]
        .as_chunks::<METADATA>()
        .0
        .iter()
        .map(|meta| {
            let offset = read_u32(meta, 8) as usize;
            let count = read_u32(meta, 12) as usize;
            let role = match read_u32(meta, 4) {
                0 => Role::OuterWall,
                1 => Role::InnerWall,
                _ => panic!("invalid perimeter role"),
            };
            Path {
                inset_index: read_u32(meta, 0),
                role,
                points: &points[offset..offset + count],
            }
        })
        .collect::<Vec<_>>();
    let mut cursor = points_end + 4;
    let options = (0..read_u32(bytes, points_end))
        .map(|_| {
            let key = read_text(bytes, &mut cursor);
            let tag = bytes[cursor];
            cursor += 1;
            let value = match tag {
                0 => {
                    cursor += 1;
                    Value::Boolean(bytes[cursor - 1] != 0)
                }
                1 => {
                    cursor += 8;
                    Value::Number(read_f64(bytes, cursor - 8))
                }
                2 => Value::Choice(read_text(bytes, &mut cursor)),
                _ => panic!("invalid option tag"),
            };
            ResolvedOption { key, value }
        })
        .collect::<Vec<_>>();
    run(Layer {
        index: read_u32(bytes, LAYER_INDEX),
        z_bottom: read_f64(bytes, Z_BOTTOM),
        height: read_f64(bytes, HEIGHT),
        paths: &paths,
        options: &options,
    })
}

fn read_text<'a>(data: &'a [u8], cursor: &mut usize) -> &'a str {
    let count = read_u32(data, *cursor) as usize;
    *cursor += 4 + count;
    std::str::from_utf8(&data[*cursor - count..*cursor]).unwrap()
}
fn put_text(data: &mut Vec<u8>, text: &str) {
    put_u32(data, text.len());
    data.extend_from_slice(text.as_bytes());
}
fn put_u32(data: &mut Vec<u8>, value: usize) {
    data.extend_from_slice(&(value as u32).to_le_bytes());
}
fn put_float(data: &mut Vec<u8>, value: f64) {
    data.extend_from_slice(&canonical(value).to_le_bytes());
}
fn canonical(value: f64) -> f64 {
    if value.is_nan() { f64::NAN } else { value }
}
fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}
fn read_f64(data: &[u8], offset: usize) -> f64 {
    f64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
}

pub fn encode_edits(edits: &[Edit]) -> Vec<u8> {
    let mut data = Vec::with_capacity(EDIT_HEADER + EDIT * edits.len());
    put_u32(&mut data, edits.len());
    for edit in edits {
        put_u32(&mut data, edit.source);
        put_float(&mut data, edit.z);
        put_float(&mut data, edit.height);
        put_float(&mut data, edit.flow_multiplier);
    }
    data
}

pub fn decode_edits(data: &[u8]) -> Option<Vec<Edit>> {
    let records = data.get(EDIT_HEADER..)?;
    if !records.len().is_multiple_of(EDIT) || records.len() / EDIT != read_u32(data, 0) as usize {
        return None;
    }
    Some(
        records
            .as_chunks::<EDIT>()
            .0
            .iter()
            .map(|edit| Edit {
                source: read_u32(edit, 0) as usize,
                z: canonical(read_f64(edit, EDIT_Z)),
                height: canonical(read_f64(edit, EDIT_HEIGHT)),
                flow_multiplier: canonical(read_f64(edit, EDIT_FLOW)),
            })
            .collect(),
    )
}

pub fn geometry() -> Vec<u8> {
    let points = (0..PATHS)
        .flat_map(|path| {
            (0..POINTS).map(move |point| {
                let p = point % (POINTS - 1);
                let d = (p % 50) as f64 * 0.03125;
                let (x, y) = match p / 50 {
                    0 => (d, 0.0),
                    1 => (1.5625, d),
                    2 => (1.5625 - d, 1.5625),
                    _ => (0.0, 1.5625 - d),
                };
                [x + (path % 50) as f64 * 2.0, y + (path / 50) as f64 * 2.0]
            })
        })
        .collect::<Vec<Point>>();
    let paths = points
        .as_chunks::<POINTS>()
        .0
        .iter()
        .map(|points| Path {
            inset_index: 0,
            points,
            role: Role::OuterWall,
        })
        .collect::<Vec<_>>();
    encode_layer(Layer {
        index: 0,
        z_bottom: 0.0,
        height: LAYER_HEIGHT,
        paths: &paths,
        options: &[
            ResolvedOption {
                key: "odd_offset",
                value: Value::Number(0.5),
            },
            ResolvedOption {
                key: "lower_fraction",
                value: Value::Number(0.5),
            },
        ],
    })
}

pub fn set_layer(data: &mut [u8], layer: u32) {
    data[LAYER_INDEX..LAYER_INDEX + 4].copy_from_slice(&layer.to_le_bytes());
    data[Z_BOTTOM..Z_BOTTOM + 8].copy_from_slice(&(f64::from(layer) * LAYER_HEIGHT).to_le_bytes());
    for (path, meta) in data[HEADER..HEADER + METADATA * PATHS]
        .as_chunks_mut::<METADATA>()
        .0
        .iter_mut()
        .enumerate()
    {
        let inset = (path as u32 + layer) % 4;
        meta[..4].copy_from_slice(&inset.to_le_bytes());
        meta[4..8].copy_from_slice(&u32::from(inset != 0).to_le_bytes());
    }
}

pub fn js_layout() -> String {
    format!(
        "const HEADER={HEADER}, METADATA={METADATA}, EDIT_HEADER={EDIT_HEADER}, EDIT={EDIT}, \
        EDIT_Z={EDIT_Z}, EDIT_HEIGHT={EDIT_HEIGHT}, EDIT_FLOW={EDIT_FLOW}, PATH_COUNT={PATH_COUNT}, \
        POINT_COUNT={POINT_COUNT}, POINT_BYTES={POINT_BYTES}, LAYER_INDEX={LAYER_INDEX}, \
        Z_BOTTOM={Z_BOTTOM}, HEIGHT={HEIGHT}, LAYER_HEIGHT={LAYER_HEIGHT};"
    )
}
