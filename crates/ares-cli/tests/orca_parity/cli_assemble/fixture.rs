use std::{fs, io::Write, path::Path};

const SIDES: [[usize; 3]; 12] = [
    [0, 2, 1],
    [0, 3, 2],
    [4, 5, 6],
    [4, 6, 7],
    [0, 1, 5],
    [0, 5, 4],
    [1, 2, 6],
    [1, 6, 5],
    [2, 3, 7],
    [2, 7, 6],
    [3, 0, 4],
    [3, 4, 7],
];

fn vertices(x: f32, y: f32) -> [[f32; 3]; 8] {
    [
        [x, y, 0.0],
        [x + 5.0, y, 0.0],
        [x + 5.0, y + 7.0, 0.0],
        [x, y + 7.0, 0.0],
        [x, y, 2.0],
        [x + 5.0, y, 2.0],
        [x + 5.0, y + 7.0, 2.0],
        [x, y + 7.0, 2.0],
    ]
}

pub(super) fn stl(x: f32, y: f32) -> Vec<u8> {
    let mut bytes = vec![0; 80];
    bytes.extend(12u32.to_le_bytes());
    let vertices = vertices(x, y);
    for triangle in SIDES {
        bytes.extend([0; 12]);
        for index in triangle {
            for point in vertices[index] {
                bytes.extend(point.to_le_bytes());
            }
        }
        bytes.extend([0; 2]);
    }
    bytes
}

pub(super) fn project(path: &Path) {
    use std::fmt::Write as _;
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
<metadata name="Application">OrcaSlicer-2.5.0</metadata><resources>"#,
    );
    for id in 1..=4 {
        write!(xml, r#"<object id="{id}" type="model"><mesh><vertices>"#).unwrap();
        for [x, y, z] in vertices(0.0, 0.0) {
            write!(xml, r#"<vertex x="{x}" y="{y}" z="{z}"/>"#).unwrap();
        }
        xml.push_str("</vertices><triangles>");
        for [a, b, c] in SIDES {
            write!(xml, r#"<triangle v1="{a}" v2="{b}" v3="{c}"/>"#).unwrap();
        }
        xml.push_str("</triangles></mesh></object>");
    }
    xml.push_str(
        r#"<object id="5" type="model"><components>
<component objectid="1" transform="1 0 0 0 1 0 0 0 1 60 70 0"/>
<component objectid="2" transform="1 0 0 0 1 0 0 0 1 75 72 0"/>
</components></object>
<object id="6" type="model"><components>
<component objectid="3" transform="0 1 0 -1 0 0 0 0 1 90 82 0"/>
<component objectid="4" transform="1 0 0 0 1 0 0 0 1 106 85 0"/>
</components></object>
</resources><build>
<item objectid="5" transform="1 0 0 0 1 0 0 0 1 18 19 0"/>
<item objectid="6" transform="1 0 0 0 1 0 0 0 1 33 41 0"/>
</build></model>"#,
    );
    let mut zip = zip::ZipWriter::new(fs::File::create(path).unwrap());
    for (name, bytes) in [
        (
            "[Content_Types].xml",
            r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/><Default Extension="png" ContentType="image/png"/></Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/3dmodel.model" Id="rel-1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#,
        ),
        ("3D/3dmodel.model", &xml),
    ] {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(bytes.as_bytes()).unwrap();
    }
    zip.finish().unwrap();
}

/// Extents of deposited outer walls, in G-code's 0.001 mm coordinate units.
pub(super) fn outer_wall_extents(gcode: &str) -> Vec<Vec<[i32; 4]>> {
    let (layer_marker, feature_marker) = if gcode.lines().any(|line| line == "; CHANGE_LAYER") {
        ("; CHANGE_LAYER\n", "; FEATURE: ")
    } else {
        (";LAYER_CHANGE\n", ";TYPE:")
    };
    gcode
        .split(layer_marker)
        .skip(1)
        .map(|layer| layer_outer_wall_extents(layer, feature_marker))
        .collect()
}

fn layer_outer_wall_extents(layer: &str, feature_marker: &str) -> Vec<[i32; 4]> {
    let mut paths = Vec::new();
    let mut points = Vec::new();
    let mut outer = false;
    let mut xy = [0; 2];
    for line in layer.lines() {
        if let Some(feature) = line.strip_prefix(feature_marker) {
            add_extents(&mut paths, &mut points);
            outer = feature == "Outer wall";
            continue;
        }
        let Some(move_args) = line
            .strip_prefix("G1 ")
            .or_else(|| line.strip_prefix("G0 "))
        else {
            continue;
        };
        let last = xy;
        let extrude = parse_move(move_args, &mut xy);
        if last != xy {
            if outer && extrude {
                points.extend([last, xy]);
            } else {
                add_extents(&mut paths, &mut points);
            }
        }
    }
    add_extents(&mut paths, &mut points);
    paths.sort_unstable();
    paths
}

fn parse_move(move_args: &str, xy: &mut [i32; 2]) -> bool {
    let mut extrude = false;
    for word in move_args
        .split(';')
        .next()
        .unwrap()
        .split_ascii_whitespace()
    {
        if let Some(e) = word.strip_prefix('E') {
            extrude = e.parse::<f64>().unwrap() > 0.0;
        }
        for (axis, letter) in ['X', 'Y'].into_iter().enumerate() {
            if let Some(value) = word.strip_prefix(letter) {
                xy[axis] = (value.parse::<f64>().unwrap() * 1000.0).round() as i32;
            }
        }
    }
    extrude
}

fn add_extents(paths: &mut Vec<[i32; 4]>, points: &mut Vec<[i32; 2]>) {
    if let Some([x, y]) = points.first().copied() {
        let mut bounds = [x, y, x, y];
        for [x, y] in points.drain(..) {
            bounds = [
                bounds[0].min(x),
                bounds[1].min(y),
                bounds[2].max(x),
                bounds[3].max(y),
            ];
        }
        paths.push(bounds);
    }
}
