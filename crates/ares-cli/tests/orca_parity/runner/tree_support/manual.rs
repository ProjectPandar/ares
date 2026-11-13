use super::{OrcaRunner, has_support_extrusion, overhang_case, parity};
use crate::runner::ExportedCase;
use regex::Regex;
use std::io::{Cursor, Read, Write};

#[path = "manual/blocker.rs"]
mod blocker;
#[path = "manual/split.rs"]
mod split;
#[path = "manual/split_sides.rs"]
mod split_sides;
#[path = "manual/styles.rs"]
mod styles;
#[path = "manual/vertical.rs"]
mod vertical;

// OrcaSlicer 2.5.0 TreeSupport.cpp:3359-3371 includes XY clearance in node classification.
#[test]
fn manual_slim_painted_overhang_collision_clearance_preserves_complete_output() {
    let label = "tree-support/manual/slim/painted-overhang";
    let Some(baseline) = overhang_case(label, "tree_slim", &[("support_type", "tree(manual)")])
    else {
        return;
    };
    assert!(!has_support_extrusion(
        std::str::from_utf8(&baseline.reference).unwrap()
    ));
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let project = paint_canopy(&baseline.project, &["4", "4"]);
    let sliced = slice_painted(&runner, project, "painted-manual");
    assert!(has_support_extrusion(
        std::str::from_utf8(&sliced.reference).unwrap()
    ));
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

fn slice_painted(runner: &OrcaRunner, project: Vec<u8>, label: &str) -> crate::runner::ParityCase {
    let project_path = runner.work.join(format!("{label}.3mf"));
    let output_dir = runner.work.join(format!("{label}-native"));
    std::fs::write(&project_path, &project).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();
    runner
        .slice_case(&ExportedCase {
            label: label.into(),
            project,
            project_path,
            output_dir,
        })
        .unwrap()
}

fn paint_canopy(project: &[u8], codes: &[&str]) -> Vec<u8> {
    paint_faces(project, codes, |[a, b, c]| {
        let normal_z = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        normal_z < 0.0 && [a, b, c].iter().all(|point| point[2] > 0.0)
    })
}

fn paint_faces(project: &[u8], codes: &[&str], select: impl Fn([[f64; 3]; 3]) -> bool) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut painted = 0;
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name().starts_with("3D/Objects/") && file.name().ends_with(".model") {
            let xml = String::from_utf8(bytes).unwrap();
            bytes = paint_model(&xml, codes, &mut painted, &select).into_bytes();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    assert!(painted > 0);
    assert_eq!(painted, codes.len());
    output.finish().unwrap().into_inner()
}

fn paint_model(
    xml: &str,
    codes: &[&str],
    painted: &mut usize,
    select: &impl Fn([[f64; 3]; 3]) -> bool,
) -> String {
    let vertices = Regex::new(r#"<vertex\s+x="([^"]+)"\s+y="([^"]+)"\s+z="([^"]+)"\s*/>"#).unwrap();
    let triangles = Regex::new(r#"<triangle\s+v1="(\d+)"\s+v2="(\d+)"\s+v3="(\d+)"\s*/>"#).unwrap();
    let points: Vec<[f64; 3]> = vertices
        .captures_iter(xml)
        .map(|vertex| std::array::from_fn(|axis| vertex[axis + 1].parse().unwrap()))
        .collect();
    triangles
        .replace_all(xml, |triangle: &regex::Captures<'_>| {
            let points = std::array::from_fn(|corner| {
                points[triangle[corner + 1].parse::<usize>().unwrap()]
            });
            if select(points) {
                let code = codes[*painted];
                *painted += 1;
                triangle[0].replace("/>", &format!(" paint_supports=\"{code}\"/>"))
            } else {
                triangle[0].to_owned()
            }
        })
        .into_owned()
}
