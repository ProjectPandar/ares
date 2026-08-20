use crate::{
    SliceError,
    project_slice::{
        prepare_infill::horizontal_shell_promotion::{self, PromotionEvent},
        region_slices::RegionSurfaceKind,
        tests::support::KsrArchive,
    },
};

#[test]
fn task22o25_valid_typed_archive_schedule_promotes_internal_surfaces() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"extra_solid_infills\": \"\"",
        "\"extra_solid_infills\": \"1#\"",
    );
    horizontal_shell_promotion::reset_hooks();
    let input = super::fixture::prepare_o24(archive.bytes());
    let before = kind_count(&input.objects, RegionSurfaceKind::Internal);
    let pointers_before = pointers(&input.objects);
    let outer_before = [
        input.objects.as_ptr() as usize,
        input.caches.as_ptr() as usize,
        input.projections.as_ptr() as usize,
        input.trims.as_ptr() as usize,
        input.regularizations.as_ptr() as usize,
        input.filters.as_ptr() as usize,
    ];
    let output = horizontal_shell_promotion::prepare(input).unwrap();
    let after = kind_count(&output.objects, RegionSurfaceKind::Internal);
    assert!(before > 0);
    assert!(kind_count(&output.objects, RegionSurfaceKind::InternalSolid) >= before);
    assert_eq!(after, 0);
    assert!(horizontal_shell_promotion::commits() > 0);
    assert_eq!(
        horizontal_shell_promotion::events()
            .iter()
            .filter(|&&event| event == PromotionEvent::PromotedSurface)
            .count(),
        before
    );
    assert_eq!(pointers(&output.objects), pointers_before);
    assert_eq!(
        [
            output.objects.as_ptr() as usize,
            output.caches.as_ptr() as usize,
            output.projections.as_ptr() as usize,
            output.trims.as_ptr() as usize,
            output.regularizations.as_ptr() as usize,
            output.filters.as_ptr() as usize,
        ],
        outer_before
    );
    horizontal_shell_promotion::dispose(output);
}

#[test]
fn task22o25_malformed_typed_schedule_rolls_back_before_any_commit() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"extra_solid_infills\": \"\"",
        "\"extra_solid_infills\": \"2147483648\"",
    );
    horizontal_shell_promotion::reset_hooks();
    let input = super::fixture::prepare_o24(archive.bytes());
    assert!(matches!(
        horizontal_shell_promotion::prepare(input),
        Err(SliceError::InvalidInput(message))
            if message == "invalid extra_solid_infills pattern"
    ));
    assert_eq!(horizontal_shell_promotion::commits(), 0);
    assert_eq!(horizontal_shell_promotion::disposals(), 1);
}

#[test]
fn task22o25_promotion_has_no_sparse_density_gate_in_resolved_region_options() {
    for density in [0.0, 15.0, 100.0] {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            "Metadata/project_settings.config",
            "\"extra_solid_infills\": \"\"",
            "\"extra_solid_infills\": \"1#\"",
        );
        let mut input = super::fixture::prepare_o24(archive.bytes());
        let prelude = &mut input.predecessor.objects[0]
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        prelude.object.object.as_parts_mut().0.regions[0]
            .options
            .sparse_infill_density
            .0 = density;
        let before = kind_count(&input.objects, RegionSurfaceKind::Internal);
        horizontal_shell_promotion::reset_hooks();
        let output = horizontal_shell_promotion::prepare(input).unwrap();
        assert!(before > 0);
        assert_eq!(
            kind_count(&output.objects, RegionSurfaceKind::Internal),
            0,
            "density {density} must not gate extra-solid promotion"
        );
        assert!(horizontal_shell_promotion::commits() > 0);
        horizontal_shell_promotion::dispose(output);
    }
}

#[test]
fn task22o25_model_part_schedule_overrides_the_global_schedule() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"extra_solid_infills\": \"\"",
        "\"extra_solid_infills\": \"2147483647\"",
    );
    archive.replace(
        "Metadata/model_settings.config",
        "    <part id=\"1\" subtype=\"normal_part\">",
        "    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"extra_solid_infills\" value=\"1#\"/>",
    );
    horizontal_shell_promotion::reset_hooks();
    let output = super::fixture::prepare(archive.bytes());
    assert!(horizontal_shell_promotion::commits() > 0);
    assert!(horizontal_shell_promotion::events().contains(&PromotionEvent::PromotedSurface));
    horizontal_shell_promotion::dispose(output);
}

#[test]
fn task22o25_matching_uses_planned_array_index_not_stored_layer_id() {
    let mut input = super::fixture::prepare_o24(KsrArchive::new().bytes());
    let selected = input.objects[0]
        .records
        .iter()
        .position(|record| {
            record.as_ref().is_some_and(|record| {
                record
                    .fill_surfaces
                    .iter()
                    .any(|surface| surface.as_parts().0 == RegionSurfaceKind::Internal)
            })
        })
        .unwrap();
    let prelude = &mut input.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let post_regions = prelude.object.object.as_parts_mut().0;
    post_regions.regions[0].options.extra_solid_infills.0 = format!("{},2147483647", selected + 1);
    post_regions.plan.layers[selected].id = 900_000;
    prelude.object.records[selected].as_mut().unwrap().layer_id = 900_000;

    let output = horizontal_shell_promotion::prepare(input).unwrap();
    let record = output.objects[0].records[selected].as_ref().unwrap();
    assert!(
        record
            .fill_surfaces
            .iter()
            .all(|surface| surface.as_parts().0 != RegionSurfaceKind::Internal)
    );
    horizontal_shell_promotion::dispose(output);
}

fn pointers(
    objects: &[crate::project_slice::prepare_infill::surface_type_detection::PreparedSurfaceTypeObject],
) -> Vec<(usize, Vec<usize>)> {
    objects
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .map(|record| {
            (
                record.fill_surfaces.as_ptr() as usize,
                record
                    .fill_surfaces
                    .iter()
                    .map(|surface| surface.as_parts().1.contour().points().as_ptr() as usize)
                    .collect(),
            )
        })
        .collect()
}

fn kind_count(
    objects: &[crate::project_slice::prepare_infill::surface_type_detection::PreparedSurfaceTypeObject],
    selected: RegionSurfaceKind,
) -> usize {
    objects
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.fill_surfaces)
        .filter(|surface| surface.as_parts().0 == selected)
        .count()
}
