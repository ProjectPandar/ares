//! Symmetric-input checks assert Ares determinism only, never Native parity.
//! Failure modes: mutating the borrowed mesh, unstable equal-score selection,
//! worker-dependent results, losing batch association, and missing cooling context.
use ares_core::plater::{OrientMesh, OrientParams, auto_orient, auto_orient_mesh};

const CUBE: &[u8] =
    include_bytes!("../../../tests/parity/machine-acceleration-cache-rounding/stock-control.3mf");

#[test]
fn symmetric_mesh_keeps_upright_without_mutating_input() {
    let project = ares_core::load_project(CUBE).unwrap();
    let mesh = project.objects()[0].volumes()[0].mesh();
    let before = mesh.clone();
    let result = auto_orient_mesh(mesh).unwrap();
    assert_eq!(mesh, &before);
    assert_eq!(result.direction, [0., 0., 1.]);
    assert!(result.angle.abs() < 1e-12);
    assert_eq!(result, auto_orient_mesh(mesh).unwrap());
}

#[test]
fn symmetric_batch_is_independent_of_worker_count_in_both_scoring_modes() {
    let project = ares_core::load_project(CUBE).unwrap();
    let mesh = project.objects()[0].volumes()[0].mesh();
    let item = OrientMesh {
        mesh,
        overhang_angle: 30.,
        cooling_direction: Some([1., 0., 0.]),
    };
    for min_volume in [false, true] {
        let params = OrientParams {
            min_volume,
            ..OrientParams::default()
        };
        let serial = auto_orient(&[item; 4], &params).unwrap();
        let parallel = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let params = &params;
                    scope.spawn(move || auto_orient(&[item], params).unwrap().remove(0))
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(serial, parallel);
        for result in serial {
            assert!(result.cooling.is_some());
            assert_eq!(result.direction, [0., 0., 1.]);
            assert!(
                result
                    .rotation_matrix
                    .iter()
                    .flatten()
                    .all(|value| value.is_finite())
            );
        }
    }
}
