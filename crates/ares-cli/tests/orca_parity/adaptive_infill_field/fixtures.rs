//! Procedural 3MF fixture builders for adaptive infill field parity cases.

use std::io::{Cursor, Read, Write};

use serde_json::{Value, json};

const BASE_3MF: &[u8] = include_bytes!("../../../../../tests/parity/orca_cli_ender3.3mf");

pub(super) struct MeshSpec {
    pub(super) vertices: Vec<[f64; 3]>,
    pub(super) triangles: Vec<[usize; 3]>,
    pub(super) build_z: f64,
}

pub(super) fn box_mesh(sx: f64, sy: f64, sz: f64) -> MeshSpec {
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    push_box(
        &mut vertices,
        &mut triangles,
        [-sx / 2.0, -sy / 2.0, -sz / 2.0],
        [sx / 2.0, sy / 2.0, sz / 2.0],
    );
    MeshSpec {
        vertices,
        triangles,
        build_z: sz / 2.0,
    }
}

pub(super) fn two_bodies_mesh() -> MeshSpec {
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    push_box(
        &mut vertices,
        &mut triangles,
        [-14.0, -5.0, -5.0],
        [-4.0, 5.0, 5.0],
    );
    push_box(
        &mut vertices,
        &mut triangles,
        [4.0, -5.0, -5.0],
        [14.0, 5.0, 5.0],
    );
    MeshSpec {
        vertices,
        triangles,
        build_z: 5.0,
    }
}

pub(super) fn necked_dumbbell_mesh() -> MeshSpec {
    // 12-vertex CCW footprint: two 12x12 mm lobes linked by a 6x4 mm neck,
    // extruded 12 mm along Z. Top and bottom caps share all perimeter vertices
    // with zero T-junctions.
    let ring = [
        [-15.0, -6.0],
        [-3.0, -6.0],
        [-3.0, -2.0],
        [3.0, -2.0],
        [3.0, -6.0],
        [15.0, -6.0],
        [15.0, 6.0],
        [3.0, 6.0],
        [3.0, 2.0],
        [-3.0, 2.0],
        [-3.0, 6.0],
        [-15.0, 6.0],
    ];
    let top_tris = [
        [0, 1, 2],
        [0, 2, 9],
        [0, 9, 11],
        [11, 9, 10],
        [2, 3, 8],
        [2, 8, 9],
        [3, 4, 5],
        [3, 5, 6],
        [3, 6, 8],
        [8, 6, 7],
    ];
    extrude_xy_prism(&ring, &top_tris, -6.0, 6.0)
}

pub(super) fn stepped_bridge_mesh() -> MeshSpec {
    // 6-vertex L-profile in XZ extruded along Y in [-10, 10]:
    // 20x20x8 mm lower base + 10x20x8 mm upper step creating an internal
    // bridge shelf over sparse infill at Z = 8 mm.
    let profile_xz = [
        [-10.0, -8.0],
        [10.0, -8.0],
        [10.0, 0.0],
        [0.0, 0.0],
        [0.0, 8.0],
        [-10.0, 8.0],
    ];
    let cap_tris = [[0, 1, 2], [0, 2, 3], [0, 3, 5], [3, 4, 5]];
    extrude_xz_prism(&profile_xz, &cap_tris, -10.0, 10.0, 8.0)
}

pub(super) fn build_3mf(
    mesh: &MeshSpec,
    settings_patch: &[(&str, Option<Value>)],
    object_metadata: &[(&str, &str)],
) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(BASE_3MF)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut member = input.by_index(index).unwrap();
        let options =
            zip::write::SimpleFileOptions::default().compression_method(member.compression());
        if member.is_dir() {
            output.add_directory(member.name(), options).unwrap();
            continue;
        }
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        match member.name() {
            "3D/Objects/cube10.stl_1.model" => {
                bytes = render_model_xml(mesh).into_bytes();
            }
            "3D/3dmodel.model" => {
                let text = String::from_utf8(bytes).unwrap();
                let target = "transform=\"1 0 0 0 1 0 0 0 1 110 110 5\"";
                let replacement =
                    format!("transform=\"1 0 0 0 1 0 0 0 1 110 110 {}\"", mesh.build_z);
                assert!(text.contains(target));
                bytes = text.replace(target, &replacement).into_bytes();
            }
            "Metadata/project_settings.config" => {
                let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
                let map = settings.as_object_mut().unwrap();
                map.insert("layer_height".into(), json!("0.2"));
                map.insert("top_shell_layers".into(), json!("2"));
                map.insert("bottom_shell_layers".into(), json!("2"));
                map.insert("wall_loops".into(), json!("2"));
                map.insert("sparse_infill_pattern".into(), json!("gyroid"));
                map.insert("sparse_infill_density".into(), json!("30%"));
                map.insert("adaptive_infill_mode".into(), json!("disabled"));
                map.insert("adaptive_infill_interior_density".into(), json!("5%"));
                map.insert("adaptive_infill_gradient".into(), json!("linear"));
                for (key, value) in settings_patch {
                    match value {
                        Some(v) => {
                            map.insert((*key).into(), v.clone());
                        }
                        None => {
                            map.remove(*key);
                        }
                    }
                }
                bytes = serde_json::to_vec_pretty(&settings).unwrap();
            }
            "Metadata/model_settings.config" if !object_metadata.is_empty() => {
                let text = String::from_utf8(bytes).unwrap();
                let anchor = "<metadata key=\"extruder\" value=\"1\"/>";
                assert!(text.contains(anchor));
                let mut extra = String::from(anchor);
                for (key, value) in object_metadata {
                    extra.push_str(&format!(
                        "\n    <metadata key=\"{key}\" value=\"{value}\"/>"
                    ));
                }
                bytes = text.replacen(anchor, &extra, 1).into_bytes();
            }
            _ => {}
        }
        output.start_file(member.name(), options).unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn push_box(
    vertices: &mut Vec<[f64; 3]>,
    triangles: &mut Vec<[usize; 3]>,
    min: [f64; 3],
    max: [f64; 3],
) {
    let base = vertices.len();
    vertices.extend_from_slice(&[
        [max[0], min[1], min[2]],
        [min[0], min[1], min[2]],
        [max[0], max[1], min[2]],
        [min[0], max[1], min[2]],
        [max[0], max[1], max[2]],
        [min[0], min[1], max[2]],
        [max[0], min[1], max[2]],
        [min[0], max[1], max[2]],
    ]);
    let faces = [
        [0, 1, 2],
        [2, 1, 3],
        [4, 5, 6],
        [7, 5, 4],
        [5, 1, 6],
        [6, 1, 0],
        [2, 3, 4],
        [4, 3, 7],
        [3, 1, 7],
        [7, 1, 5],
        [6, 0, 4],
        [4, 0, 2],
    ];
    for [a, b, c] in faces {
        triangles.push([base + a, base + b, base + c]);
    }
}

fn extrude_xy_prism(ring: &[[f64; 2]], top_tris: &[[usize; 3]], z0: f64, z1: f64) -> MeshSpec {
    let n = ring.len();
    let mut vertices = Vec::with_capacity(2 * n);
    for &[x, y] in ring {
        vertices.push([x, y, z0]);
    }
    for &[x, y] in ring {
        vertices.push([x, y, z1]);
    }
    let mut triangles = Vec::with_capacity(2 * top_tris.len() + 2 * n);
    for &[a, b, c] in top_tris {
        triangles.push([a, c, b]);
        triangles.push([n + a, n + b, n + c]);
    }
    for i in 0..n {
        let j = (i + 1) % n;
        triangles.push([i, j, n + j]);
        triangles.push([i, n + j, n + i]);
    }
    MeshSpec {
        vertices,
        triangles,
        build_z: -z0,
    }
}

fn extrude_xz_prism(
    profile: &[[f64; 2]],
    cap_tris: &[[usize; 3]],
    y0: f64,
    y1: f64,
    build_z: f64,
) -> MeshSpec {
    let n = profile.len();
    let mut vertices = Vec::with_capacity(2 * n);
    for &[x, z] in profile {
        vertices.push([x, y0, z]);
    }
    for &[x, z] in profile {
        vertices.push([x, y1, z]);
    }
    let mut triangles = Vec::with_capacity(2 * cap_tris.len() + 2 * n);
    // In XZ viewed from -Y (outward -Y normal), CCW (x,z) gives outward -Y.
    for &[a, b, c] in cap_tris {
        triangles.push([a, b, c]);
        triangles.push([n + a, n + c, n + b]);
    }
    for i in 0..n {
        let j = (i + 1) % n;
        triangles.push([i, n + j, j]);
        triangles.push([i, n + i, n + j]);
    }
    MeshSpec {
        vertices,
        triangles,
        build_z,
    }
}

fn render_model_xml(mesh: &MeshSpec) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <model unit=\"millimeter\" xml:lang=\"en-US\" \
         xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\" \
         xmlns:BambuStudio=\"http://schemas.bambulab.com/package/2021\" \
         xmlns:p=\"http://schemas.microsoft.com/3dmanufacturing/production/2015/06\" \
         requiredextensions=\"p\">\n\
          <metadata name=\"BambuStudio:3mfVersion\">1</metadata>\n\
          <resources>\n\
           <object id=\"1\" p:UUID=\"00010000-81cb-4c03-9d28-80fed5dfa1dc\" type=\"model\">\n\
            <mesh>\n\
             <vertices>\n",
    );
    for [x, y, z] in &mesh.vertices {
        xml.push_str(&format!("      <vertex x=\"{x}\" y=\"{y}\" z=\"{z}\"/>\n"));
    }
    xml.push_str("     </vertices>\n     <triangles>\n");
    for [a, b, c] in &mesh.triangles {
        xml.push_str(&format!(
            "      <triangle v1=\"{a}\" v2=\"{b}\" v3=\"{c}\"/>\n"
        ));
    }
    xml.push_str(
        "     </triangles>\n    </mesh>\n   </object>\n  </resources>\n  <build/>\n</model>\n",
    );
    xml
}
