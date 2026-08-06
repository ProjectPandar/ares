use super::super::horizontal_shell_promotion::ksr::digest::surfaces_digest;
use crate::project_slice::prepare_infill::horizontal_shell_propagation::{self, PropagationEvent};

fn digest(
    objects: &[crate::project_slice::prepare_infill::surface_type_detection::PreparedSurfaceTypeObject],
) -> i128 {
    let mut digest = 0x004f_2645_5155_414c_5f44_4952_5459_5f54_i128;
    surfaces_digest(&mut digest, objects);
    digest
}

#[test]
fn task22o26_geometry_equal_production_rebuild_is_still_dirty_and_committed() {
    let input = super::fixture::controlled(false);
    let before_digest = digest(&input.objects);
    let before_pointer = input.objects[0].records[1]
        .as_ref()
        .unwrap()
        .fill_surfaces
        .as_ptr();

    horizontal_shell_propagation::reset_hooks();
    let output = horizontal_shell_propagation::prepare(input).unwrap();
    let after = output.objects[0].records[1].as_ref().unwrap();
    assert_eq!(
        digest(&output.objects),
        before_digest,
        "after={:?}",
        after
            .fill_surfaces
            .iter()
            .map(|surface| {
                let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
                (
                    kind,
                    expolygon
                        .contour()
                        .points()
                        .iter()
                        .map(|point| (point.x(), point.y()))
                        .collect::<Vec<_>>(),
                    thickness,
                    layers,
                    angle,
                    extra,
                )
            })
            .collect::<Vec<_>>()
    );
    assert_ne!(after.fill_surfaces.as_ptr(), before_pointer);
    assert_eq!(horizontal_shell_propagation::commits(), 1);
    assert!(matches!(
        horizontal_shell_propagation::events().last(),
        Some(PropagationEvent::DirtyCommit {
            object: 0,
            layer: 1
        })
    ));
    horizontal_shell_propagation::dispose(output);
}
