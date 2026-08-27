pub(in crate::project_slice::tests::prepare_infill) mod checksum;
pub(in crate::project_slice::tests::prepare_infill) mod totals;

pub(super) use checksum::checksum;
pub(in crate::project_slice::tests::prepare_infill) use checksum::unrelated_checksum;
pub(super) use totals::totals;

use crate::project_slice::{
    perimeters,
    prepare_infill::{fill_surfaces, surface_type_detection},
    tests::support::KsrArchive,
};

#[test]
fn task22o18_ksr_fill_surface_preparation_is_repeatable() {
    let first_detected = detected();

    let first = fill_surfaces::prepare(first_detected);
    let second = fill_surfaces::prepare(detected());
    assert_eq!(
        (
            checksum(&first.predecessor, &first.objects),
            totals(&first.objects)
        ),
        (
            checksum(&second.predecessor, &second.objects),
            totals(&second.objects)
        )
    );
}

fn detected() -> surface_type_detection::PreparedPostSurfaceTypeDetection {
    surface_type_detection::prepare(
        perimeters::prepare_post_layer_region_perimeters(&KsrArchive::new().bytes()).unwrap(),
    )
    .unwrap()
}
