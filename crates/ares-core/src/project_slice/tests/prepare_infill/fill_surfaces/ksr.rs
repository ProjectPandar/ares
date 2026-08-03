mod checksum;
mod totals;

use checksum::checksum;
pub(super) use checksum::unrelated_checksum;
use totals::totals;

use crate::project_slice::{
    perimeters,
    prepare_infill::{fill_surfaces, surface_type_detection},
    tests::support::KsrArchive,
};

const O17_CHECKSUM: i128 = -126_362_407_653_399_901_571_400_348_049_652_748_978;
const O17_TOTALS: [usize; 24] = [
    1, 460, 460, 2_881, 5_243, 2_285, 1_112, 1_112, 5_388, 519, 6, 666, 4_197, 1_294, 113, 6, 48,
    1_127, 5_388, 517, 85_886, 1_294, 168, 46_011,
];
const EXPECTED_CHECKSUM: i128 = O17_CHECKSUM;
const EXPECTED_TOTALS: [usize; 26] = [
    1, 460, 460, 2_881, 5_243, 2_285, 1_112, 1_112, 5_388, 519, 6, 666, 4_197, 1_294, 113, 6, 48,
    1_127, 5_388, 517, 85_886, 1_294, 168, 46_011, 0, 0,
];

#[test]
fn task22o18_ksr_fill_surface_preparation_is_literal_and_repeatable() {
    let first_detected = detected();
    assert_eq!(
        checksum(&first_detected.predecessor, &first_detected.objects),
        O17_CHECKSUM
    );
    assert_eq!(totals(&first_detected.objects)[..24], O17_TOTALS);

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
    assert_eq!(
        (
            checksum(&first.predecessor, &first.objects),
            totals(&first.objects)
        ),
        (EXPECTED_CHECKSUM, EXPECTED_TOTALS)
    );
}

fn detected() -> surface_type_detection::PreparedPostSurfaceTypeDetection {
    surface_type_detection::prepare(
        perimeters::prepare_post_layer_region_perimeters(KsrArchive::new().bytes()).unwrap(),
    )
    .unwrap()
}
