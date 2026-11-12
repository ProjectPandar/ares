use ares_core::{PlaterSession, Project, StlFile, StlImportOptions};

fn part(scale: [f32; 3], offset: [f32; 3]) -> Vec<u8> {
    let mut bytes = include_bytes!("../../../tests/parity/cube10.stl").to_vec();
    for facet in (84..bytes.len()).step_by(50) {
        for vertex in (facet + 12..facet + 48).step_by(12) {
            for axis in 0..3 {
                let at = vertex + axis * 4;
                let value = f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
                bytes[at..at + 4]
                    .copy_from_slice(&(value * scale[axis] + offset[axis]).to_le_bytes());
            }
        }
    }
    bytes
}

fn bounds(project: &Project) -> Vec<([f64; 3], [f64; 3])> {
    project
        .objects()
        .iter()
        .flat_map(|object| {
            object.volumes().iter().map(move |volume| {
                let mut min = [f64::INFINITY; 3];
                let mut max = [f64::NEG_INFINITY; 3];
                for point in volume.mesh().vertices() {
                    let point = object.instances()[0]
                        .transform()
                        .transform_point(volume.transform().transform_point(*point));
                    for (axis, coordinate) in [point.x, point.y, point.z].into_iter().enumerate() {
                        min[axis] = min[axis].min(coordinate);
                        max[axis] = max[axis].max(coordinate);
                    }
                }
                (min, max)
            })
        })
        .collect()
}

#[test]
fn four_choices_keep_all_parts_and_apply_the_requested_ground_policy() {
    let base = part([0.8, 0.3, 0.2], [0.0, 0.0, 3.0]);
    let upper = part([0.3, 0.8, 0.2], [12.0, 4.0, 7.0]);
    let files = [
        StlFile {
            name: "base.stl",
            bytes: &base,
        },
        StlFile {
            name: "upper.stl",
            bytes: &upper,
        },
    ];
    for as_multipart in [true, false] {
        for auto_drop in [true, false] {
            let session = PlaterSession::load_stl_files(
                &files,
                StlImportOptions {
                    as_multipart,
                    auto_drop,
                },
            )
            .unwrap();
            let project = session.project();
            assert_eq!(project.objects().len(), if as_multipart { 1 } else { 2 });
            let volumes: Vec<_> = project.objects().iter().flat_map(|o| o.volumes()).collect();
            assert_eq!(
                volumes.iter().map(|v| v.name()).collect::<Vec<_>>(),
                ["base.stl", "upper.stl"]
            );
            assert!(volumes.iter().all(|v| v.mesh().triangles().len() == 12));
            let boxes = bounds(project);
            assert_eq!(boxes.len(), 2);
            assert!((boxes[0].1[0] - boxes[0].0[0] - 8.0).abs() < 1e-5);
            let expected_z = if auto_drop {
                [0.0, if as_multipart { 4.0 } else { 0.0 }]
            } else {
                [2.0, 6.0]
            };
            for (actual, expected) in boxes.iter().zip(expected_z) {
                assert!(
                    (actual.0[2] - expected).abs() < 1e-5,
                    "{as_multipart}/{auto_drop}: {boxes:?}"
                );
            }
            if as_multipart || !auto_drop {
                assert!((boxes[1].0[0] - boxes[0].0[0] - 14.5).abs() < 1e-5);
                assert!((boxes[1].0[1] - boxes[0].0[1] - 1.5).abs() < 1e-5);
            }
        }
    }
}

#[test]
fn no_drop_lifts_negative_parts_together_before_splitting() {
    let base = part([0.8, 0.3, 0.2], [0.0, 0.0, -2.0]);
    let upper = part([0.3, 0.8, 0.2], [12.0, 4.0, 2.0]);
    let files = [
        StlFile {
            name: "base.stl",
            bytes: &base,
        },
        StlFile {
            name: "upper.stl",
            bytes: &upper,
        },
    ];
    let session = PlaterSession::load_stl_files(
        &files,
        StlImportOptions {
            as_multipart: false,
            auto_drop: false,
        },
    )
    .unwrap();
    let boxes = bounds(session.project());
    assert_eq!(boxes[0].0[2], 0.0);
    assert_eq!(boxes[1].0[2], 4.0);
    assert!(
        session
            .project()
            .objects()
            .iter()
            .all(|o| !o.instances()[0].auto_drop())
    );
}

#[test]
fn malformed_later_input_cannot_replace_the_loaded_session() {
    let bytes = part([0.8, 0.3, 0.2], [0.0; 3]);
    let files = [
        StlFile {
            name: "base.stl",
            bytes: &bytes,
        },
        StlFile {
            name: "upper.stl",
            bytes: &bytes,
        },
    ];
    let options = StlImportOptions {
        as_multipart: true,
        auto_drop: true,
    };
    let session = PlaterSession::load_stl_files(&files, options).unwrap();
    let invalid = [
        files[0],
        StlFile {
            name: "broken.stl",
            bytes: b"not an STL",
        },
    ];
    assert!(PlaterSession::load_stl_files(&invalid, options).is_err());
    assert_eq!(session.project().objects()[0].volumes().len(), 2);
}
