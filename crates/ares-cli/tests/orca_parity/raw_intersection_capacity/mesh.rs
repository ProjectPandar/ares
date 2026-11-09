use std::{
    collections::BTreeMap,
    fmt::Write as _,
    io::{Cursor, Read, Write},
};

const MEMBER: &str = "3D/Objects/cube10.stl_1.model";
const STRIPS: usize = 16_384;

#[derive(Default)]
struct Mesh {
    vertices: Vec<[f32; 3]>,
    triangles: Vec<[u32; 3]>,
    indices: BTreeMap<[u32; 3], u32>,
}

impl Mesh {
    fn triangle(&mut self, points: [[f64; 3]; 3]) {
        let indices = points.map(|point| {
            let point = point.map(|v| v as f32);
            *self
                .indices
                .entry(point.map(f32::to_bits))
                .or_insert_with(|| {
                    let index = self.vertices.len() as u32;
                    self.vertices.push(point);
                    index
                })
        });
        self.triangles.push(indices);
    }
}

pub(super) fn subdivide(control: &[u8]) -> (Vec<u8>, serde_json::Value) {
    let imported = ares_core::load_project(control).unwrap();
    let source = imported.objects()[0].volumes()[0].mesh();
    let vertices = source
        .vertices()
        .iter()
        .map(|p| [p.x, p.y, p.z])
        .collect::<Vec<_>>();
    let min_x = vertices.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
    let max_x = vertices
        .iter()
        .map(|p| p[0])
        .fold(f64::NEG_INFINITY, f64::max);
    let step = (max_x - min_x) / STRIPS as f64;
    let mut mesh = Mesh::default();
    for triangle in source.triangles() {
        let points = triangle.map(|v| vertices[v as usize]);
        let low = points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
        let high = points
            .iter()
            .map(|p| p[0])
            .fold(f64::NEG_INFINITY, f64::max);
        if low == high {
            mesh.triangle(points);
            continue;
        }
        let first = ((low - min_x) / step) as usize;
        let end = ((high - min_x) / step).ceil() as usize;
        for strip in first..end {
            let polygon = clip(&points, min_x + strip as f64 * step, true);
            let polygon = clip(&polygon, min_x + (strip + 1) as f64 * step, false);
            for i in 1..polygon.len() - 1 {
                mesh.triangle([polygon[0], polygon[i], polygon[i + 1]]);
            }
        }
    }
    // Stock cube local Z is [-5,5], translated to [0,10] by its saved transform.
    // Its unchanged 0.2 mm layer heights produce these fifty interior planes.
    let planes = (0..50).map(|i| -4.9 + i as f32 * 0.2).collect::<Vec<_>>();
    let intersections = mesh
        .triangles
        .iter()
        .map(|triangle| {
            let zs = triangle.map(|v| mesh.vertices[v as usize][2]);
            let min = zs.into_iter().fold(f32::INFINITY, f32::min);
            let max = zs.into_iter().fold(f32::NEG_INFINITY, f32::max);
            planes.iter().filter(|&&z| min < z && z < max).count()
        })
        .sum::<usize>();
    let mut changed = String::from("<vertices>");
    for [x, y, z] in &mesh.vertices {
        write!(changed, "<vertex x=\"{x}\" y=\"{y}\" z=\"{z}\"/>").unwrap();
    }
    changed.push_str("</vertices><triangles>");
    for [a, b, c] in &mesh.triangles {
        write!(changed, "<triangle v1=\"{a}\" v2=\"{b}\" v3=\"{c}\"/>").unwrap();
    }
    changed.push_str("</triangles>");
    let mut archive = zip::ZipArchive::new(Cursor::new(control)).unwrap();
    let mut model = String::new();
    archive
        .by_name(MEMBER)
        .unwrap()
        .read_to_string(&mut model)
        .unwrap();
    let first = model.find("<vertices>").unwrap();
    let end = model.find("</triangles>").unwrap() + "</triangles>".len();
    model.replace_range(first..end, &changed);
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == MEMBER {
            writer
                .start_file(MEMBER, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(model.as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    let project = writer.finish().unwrap().into_inner();
    let density = serde_json::json!({
        "stock_sha256": super::sha(control), "dense_sha256": super::sha(&project),
        "axis": "X", "strips": STRIPS, "source_triangles": source.triangles().len(),
        "vertices": mesh.vertices.len(), "triangles": mesh.triangles.len(),
        "local_planes": planes, "raw_intersections": intersections,
        "method": "sum of strict triangle Z-span crossings over stock interior slicing planes",
        "settings_changed": false, "geometry_boundary_changed": false,
    });
    (project, density)
}

fn clip(points: &[[f64; 3]], x: f64, greater: bool) -> Vec<[f64; 3]> {
    let mut result = Vec::new();
    let mut previous = *points.last().unwrap();
    let mut previous_inside = if greater {
        previous[0] >= x
    } else {
        previous[0] <= x
    };
    for &point in points {
        let inside = if greater {
            point[0] >= x
        } else {
            point[0] <= x
        };
        if previous_inside != inside {
            let t = (x - previous[0]) / (point[0] - previous[0]);
            let mut intersection =
                std::array::from_fn(|i| previous[i] + t * (point[i] - previous[i]));
            intersection[0] = x;
            result.push(intersection);
        }
        if inside {
            result.push(point);
        }
        previous = point;
        previous_inside = inside;
    }
    result.dedup();
    if result.first() == result.last() {
        result.pop();
    }
    result
}
