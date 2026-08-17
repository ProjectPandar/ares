use super::{
    Placement,
    mesh::{Triangle, TriangleMesh, Vec3},
    place_loop,
    sampling::sample_uniform,
    split_at,
    visibility::GlobalVisibility,
};
use crate::{
    geometry::CoordinateScale,
    project_slice::{
        perimeters::classic::{
            chained_loops::{ExtrusionLoop, ExtrusionLoopRole},
            materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
        },
        seam_candidates::{SeamCandidate, SeamCandidatePosition},
    },
};

fn upward_triangle(z: f32) -> Triangle {
    Triangle::new(
        Vec3::new(-10.0, -10.0, z),
        Vec3::new(10.0, -10.0, z),
        Vec3::new(0.0, 10.0, z),
    )
}

#[test]
fn task22o128_uniform_samples_are_deterministic_and_area_weighted() {
    let mesh = TriangleMesh::new(vec![
        upward_triangle(0.0),
        Triangle::new(
            Vec3::new(20.0, -20.0, 0.0),
            Vec3::new(60.0, -20.0, 0.0),
            Vec3::new(20.0, 20.0, 0.0),
        ),
    ]);

    let first = sample_uniform(&mesh, 4_000);
    let second = sample_uniform(&mesh, 4_000);

    assert_eq!(first, second);
    let larger_triangle_samples = first
        .positions
        .iter()
        .filter(|point| point.x >= 20.0)
        .count();
    assert!((3_150..=3_250).contains(&larger_triangle_samples));
}

#[test]
fn task22o128_occluding_surface_lowers_visibility() {
    let open = GlobalVisibility::from_mesh(TriangleMesh::new(vec![upward_triangle(0.0)]), 2_000);
    let occluded = GlobalVisibility::from_mesh(
        TriangleMesh::new(vec![
            upward_triangle(0.0),
            Triangle::new(
                Vec3::new(-10.0, -10.0, 2.0),
                Vec3::new(0.0, 10.0, 2.0),
                Vec3::new(10.0, -10.0, 2.0),
            ),
        ]),
        2_000,
    );
    let point = Vec3::new(0.0, 0.0, 0.0);

    let open_visibility = open.at(point);
    let occluded_visibility = occluded.at(point);

    assert!(open_visibility > 0.99, "{open_visibility}");
    assert!(
        occluded_visibility < open_visibility,
        "{occluded_visibility}"
    );
}

#[test]
fn task22o128_split_at_existing_vertex_does_not_emit_zero_length_segment() {
    let mut loop_ = ExtrusionLoop {
        paths: vec![ExtrusionPath {
            polyline: Polyline3 {
                points: [(0, 0), (1_000_000, 0), (1_000_000, 1_000_000), (0, 0)]
                    .into_iter()
                    .map(|(x, y)| Point3 { x, y, z: 200_000 })
                    .collect(),
            },
            role: ExtrusionRole::ExternalPerimeter,
            mm3_per_mm: 0.04,
            width: 0.4,
            height: 0.2,
        }],
        role: ExtrusionLoopRole::Default,
    };

    split_at(&mut loop_, (1.0, 0.0), CoordinateScale::Normal);

    assert_eq!(
        loop_.paths[0].polyline.points[..2],
        [
            Point3 {
                x: 1_000_000,
                y: 0,
                z: 200_000,
            },
            Point3 {
                x: 1_000_000,
                y: 1_000_000,
                z: 200_000,
            },
        ]
    );
}

#[test]
fn task22o130_internal_corner_projection_starts_at_selected_candidate() {
    let candidate = |x, y| SeamCandidate {
        position: SeamCandidatePosition { x, y, z: 0.2 },
        perimeter_index: 0,
        local_ccw_angle: -std::f32::consts::FRAC_PI_2,
    };
    let selected = candidate(0.0, 0.0);
    let previous = candidate(-1.0, 0.0);
    let next = candidate(0.0, -1.0);
    let mut loop_ = ExtrusionLoop {
        paths: vec![ExtrusionPath {
            polyline: Polyline3 {
                points: [(1, 1), (5, 1), (5, 5), (1, 5), (1, 1)]
                    .into_iter()
                    .map(|(x, y)| Point3 {
                        x: x * 1_000_000,
                        y: y * 1_000_000,
                        z: 200_000,
                    })
                    .collect(),
            },
            role: ExtrusionRole::Perimeter,
            mm3_per_mm: 0.04,
            width: 0.4,
            height: 0.2,
        }],
        role: ExtrusionLoopRole::Internal,
    };

    place_loop(
        &mut loop_,
        Placement {
            selected: &selected,
            previous: &previous,
            next: &next,
            position: Vec3::new(0.2, 0.1, 0.2),
        },
        CoordinateScale::Normal,
    );

    assert_eq!(
        loop_.paths[0].polyline.points[0],
        Point3 {
            x: 1_204_148,
            y: 1_000_000,
            z: 200_000,
        }
    );
}

#[tokio::test]
async fn task22o142_inner_path_role_projects_aligned_seam() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let label = lines
        .iter()
        .position(|line| *line == "M624 AQAAAAAAAAA=")
        .unwrap();
    let first_outer = lines
        .iter()
        .position(|line| *line == "; FEATURE: Outer wall")
        .unwrap();
    assert_eq!(lines[label + 1], "G1 X140.158 Y102.797 F60000");
    assert_eq!(lines[first_outer - 2], "G1 X140.625 Y102.983 F60000");
}
