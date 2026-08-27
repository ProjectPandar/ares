use crate::project_slice::{
    perimeters,
    prepare_infill::{fill_surfaces, surface_type_detection},
    tests::support::KsrArchive,
};

fn detected(active: bool) -> surface_type_detection::PreparedPostSurfaceTypeDetection {
    let mut archive = KsrArchive::new();
    if active {
        archive.replace_unique(
            "Metadata/project_settings.config",
            "\"top_shell_layers\": \"5\"",
            "\"top_shell_layers\": \"0\"",
        );
    }
    surface_type_detection::prepare(
        perimeters::prepare_post_layer_region_perimeters(&archive.bytes()).unwrap(),
    )
    .unwrap()
}

#[test]
fn task22o18_prepare_validates_every_alignment_class_before_any_retag() {
    for corrupt in 0..5 {
        let mut prepared = detected(true);
        match corrupt {
            0 => {
                prepared.objects.pop();
            }
            1 => {
                prepared.objects[0].records.pop();
            }
            2 => {
                prepared.objects[0].records[0] = None;
            }
            3 => {
                prepared.predecessor.objects[0]
                    .predecessor
                    .predecessor
                    .predecessor
                    .predecessor
                    .object
                    .records[0]
                    .as_mut()
                    .unwrap()
                    .compatible_region_ids = [usize::MAX];
            }
            4 => {
                prepared.predecessor.objects[0]
                    .predecessor
                    .predecessor
                    .predecessor
                    .predecessor
                    .object
                    .records[0]
                    .as_mut()
                    .unwrap()
                    .source_object_index += 1;
            }
            _ => unreachable!(),
        }
        fill_surfaces::reset_retags();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                fill_surfaces::prepare(prepared)
            }))
            .is_err(),
            "alignment corruption {corrupt} must fail"
        );
        assert_eq!(fill_surfaces::retags(), 0);
    }
}

#[test]
fn task22o18_aligned_none_slots_remain_none() {
    let mut prepared = detected(false);
    prepared.objects[0].records[0] = None;
    prepared.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .records[0] = None;
    let output = fill_surfaces::prepare(prepared);
    assert!(output.objects[0].records[0].is_none());
}
