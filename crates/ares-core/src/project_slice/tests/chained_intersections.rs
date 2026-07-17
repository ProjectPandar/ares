use crate::{ProjectVolumeType, geometry::CoordinateScale};

use super::{
    super::chained_intersections::chain_project_intersections,
    raw_support::{bfs_restart_request, intersections, ordinal_gap_object, planned_layers},
    support::identity_resolved,
};

#[test]
fn task22c_project_wrapper_preserves_object_volume_and_layer_ownership() {
    use ProjectVolumeType::{ModelPart, NegativeVolume, ParameterModifier};

    let (mut restart_objects, _, _) = bfs_restart_request();
    let source_objects = vec![ordinal_gap_object(), restart_objects.remove(0)];
    let plans = vec![
        planned_layers(0, 0, &[(100.0, 0.5), (101.0, 3.5)]),
        planned_layers(1, 0, &[(100.0, 0.5), (101.0, 3.5)]),
    ];
    let raw = intersections(
        &source_objects,
        &[identity_resolved(0), identity_resolved(1)],
        plans.clone(),
    )
    .unwrap();
    let chained = chain_project_intersections(raw);

    assert_eq!(chained.len(), 2);
    for (object, plan) in chained.iter().zip(&plans) {
        assert_eq!(object.plan(), plan);
    }
    assert_eq!(
        chained[0]
            .volumes()
            .iter()
            .map(|volume| (volume.ordinal(), volume.volume_type()))
            .collect::<Vec<_>>(),
        [(2, ModelPart), (3, ParameterModifier), (5, NegativeVolume)]
    );
    assert_eq!(
        chained[1]
            .volumes()
            .iter()
            .map(|volume| (volume.ordinal(), volume.volume_type()))
            .collect::<Vec<_>>(),
        [(1, ModelPart), (2, ModelPart), (3, ModelPart)]
    );

    for volume in chained.iter().flat_map(|object| object.volumes()) {
        assert_eq!(volume.layers().len(), 2);
        assert!(volume.layers()[0].polygons().is_empty());
        assert_eq!(volume.layers()[0].open_polylines().len(), 1);
        assert_eq!(volume.layers()[0].open_polylines()[0].points().len(), 2);
        assert_eq!(
            volume.layers()[0].open_polylines()[0].length(),
            0.5 / CoordinateScale::Normal.factor()
        );
        assert!(volume.layers()[1].polygons().is_empty());
        assert!(volume.layers()[1].open_polylines().is_empty());
    }
}
